//! STEP model import (#917): the kernel reads every solid of the file
//! (`camcraft_kernel::solidcore_step::import_step`) and meshes each one with
//! the machining mesh (watertight, outward, deviation measured) at the
//! importer's chord cap; a model over the facet budget is meshed again at a
//! coarser chord, and the chord it was made at is handed out (#864, #1054).
//! Every kernel refusal keeps its reason.
//! Provenance: our own glue over camcraft-kernel's public API (solidcore-step
//! reads ISO 10303-21/-42 from the public standards, Apache-2.0); no
//! HSMWorks material consulted.

use crate::geometry::Mesh;

/// Why a STEP file gave no mesh.
#[derive(Debug, thiserror::Error)]
pub enum StepError {
    /// The model does not fit the facet budget at any chord tried.
    #[error("model exceeds the import limit")]
    TooLarge,
    /// The reader, the kernel or the mesher refused the file; the reason
    /// names what failed.
    #[error("{0}")]
    Refused(String),
}

/// A STEP model meshed in millimetres.
#[derive(Debug)]
pub struct StepMesh {
    /// Every solid of the file, each a watertight outward mesh, with one
    /// facet range for each solid ([`Mesh::solid_ranges`]).
    pub mesh: Mesh,
    /// The chord tolerance the mesh was made at (mm): the chord cap, or
    /// coarser when the cap did not fit the facet budget.
    pub chord_mm: f64,
    /// The largest certified distance (mm) from a facet to its solid's
    /// surface, over every solid: the kernel's measured bound, never more
    /// than `chord_mm` (a solid over its chord is refused).
    pub deviation_mm: f64,
    /// What the reader and the mesher noted, one line each.
    pub notes: Vec<String>,
}

/// How many times a model over the facet budget is meshed again, coarser,
/// before it is refused as too large.
pub const COARSEN_TRIES: u32 = 6;

/// The most one coarsening multiplies the chord by.
const COARSEN_MAX_STEP: f64 = 16.0;

/// The advice a mesh refusal ends with (the wording of the earlier STEP
/// importer's tessellation refusals).
pub const ADVICE: &str = "re-export the model, or import an STL";

/// The solids of a STEP file as one watertight mesh in mm, at most `budget`
/// facets, at the chord `chord_cap` (mm) or coarser when that does not fit.
#[cfg(not(target_arch = "wasm32"))]
pub fn parse(data: &[u8], budget: usize, chord_cap: f64) -> Result<StepMesh, StepError> {
    use camcraft_kernel::machining_mesh::{MachiningMesh, machining_mesh};
    use camcraft_kernel::solidcore_impl::SolidcoreKernel;
    use camcraft_kernel::solidcore_step::import_step;

    let stop = camcraft_kernel::CancelToken::new();
    let _watch = crate::kernel_import::Watchdog::start(stop.clone(), "STEP import");
    let import = import_step(&mut SolidcoreKernel::default(), data, &stop)
        .map_err(|e| StepError::Refused(locate(data, e.to_string())))?;
    let mut chord = chord_cap;
    for _ in 0..=COARSEN_TRIES {
        let mut meshes: Vec<MachiningMesh> = Vec::with_capacity(import.solids.len());
        let mut total = 0usize;
        for solid in &import.solids {
            let mesh = machining_mesh(solid, chord, &stop).map_err(|e| {
                StepError::Refused(format!("{} could not be tessellated: {e} - {ADVICE}", solid_label(solid.item, &solid.name)))
            })?;
            total = total.saturating_add(mesh.triangles.len());
            if total > budget {
                break;
            }
            meshes.push(mesh);
        }
        if total <= budget {
            // One facet range for each solid: solids that share a wall stay
            // apart for the slicer and the winding check (#917).
            let mut facets = Vec::new();
            let mut ranges = Vec::with_capacity(meshes.len());
            for mesh in &meshes {
                let start = facets.len();
                crate::kernel_import::push_facets(mesh, &mut facets).map_err(StepError::Refused)?;
                ranges.push(start..facets.len());
            }
            let mut notes: Vec<String> = import.notes.iter().map(|n| format!("STEP: {n}")).collect();
            for solid in &import.solids {
                let label = solid_label(solid.item, &solid.name);
                notes.extend(solid.notes.iter().map(|n| format!("STEP {label}: {n}")));
            }
            for (solid, mesh) in import.solids.iter().zip(&meshes) {
                let label = solid_label(solid.item, &solid.name);
                notes.extend(mesh.report.notes.iter().map(|n| format!("STEP {label}: {n}")));
            }
            let mesh = Mesh::from_solids(facets, ranges).map_err(|e| StepError::Refused(e.to_string()))?;
            let deviation_mm = meshes.iter().map(|m| m.report.deviation.largest()).fold(0.0, f64::max);
            return Ok(StepMesh { mesh, chord_mm: chord, deviation_mm, notes });
        }
        // Facets fall about as the chord rises; `total` may be a lower bound
        // (the count stops at the budget), so the chord at least doubles.
        let ratio = total as f64 / budget.max(1) as f64;
        chord *= ratio.clamp(2.0, COARSEN_MAX_STEP);
        tracing::warn!("STEP model over its {budget}-facet budget; meshing again at {chord:.4} mm");
    }
    Err(StepError::TooLarge)
}

/// The browser build keeps solidcore out of its wasm: a STEP file is
/// refused by name there.
#[cfg(target_arch = "wasm32")]
pub fn parse(_data: &[u8], _budget: usize, _chord_cap: f64) -> Result<StepMesh, StepError> {
    Err(StepError::Refused("STEP models open in the desktop app only, not in the browser build".into()))
}

/// A syntax refusal ("... at line 9: ...") with the entity that holds the
/// line named after it, and the string or comment that never closes (#912:
/// the user finds the bad record by its id and its first line). Any other
/// refusal (a solid the kernel could not read) ends with [`ADVICE`].
#[cfg(not(target_arch = "wasm32"))]
fn locate(data: &[u8], reason: String) -> String {
    let Some(line) = reason
        .split_once("syntax error at line ")
        .and_then(|(_, rest)| rest.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse::<usize>().ok())
    else {
        return format!("{reason} - {ADVICE}");
    };
    let Ok(text) = std::str::from_utf8(data) else {
        return reason;
    };
    let at = scan(text, line);
    let mut out = reason;
    if let Some((id, first)) = at.entity {
        out.push_str(&format!(" (in entity #{id}, which starts at line {first})"));
    }
    if let Some((what, first)) = at.open {
        out.push_str(&format!("; the {what} opened at line {first} never closes"));
    }
    out
}

/// Where a line of a STEP file stands: the last entity begun on or before
/// it, and a string or comment still open at its end.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Spot {
    entity: Option<(u64, usize)>,
    open: Option<(&'static str, usize)>,
}

/// [`Spot`] at the end of `line` (counted from 1): a scan that skips
/// strings (`'...'`, `''` inside) and comments (`/* ... */`).
#[cfg(not(target_arch = "wasm32"))]
fn scan(text: &str, line: usize) -> Spot {
    let mut spot = Spot::default();
    let (mut at, mut fresh) = (1usize, true);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match spot.open {
            Some(("string", _)) => {
                if c == '\'' && chars.next_if_eq(&'\'').is_none() {
                    spot.open = None;
                }
            }
            Some(_) => {
                if c == '*' && chars.next_if_eq(&'/').is_some() {
                    spot.open = None;
                }
            }
            None if c == '\'' => spot.open = Some(("string", at)),
            None if c == '/' && chars.next_if_eq(&'*').is_some() => spot.open = Some(("comment", at)),
            None if c == '#' && fresh => {
                let mut id = String::new();
                while let Some(d) = chars.next_if(char::is_ascii_digit) {
                    id.push(d);
                }
                while chars.next_if(|w| *w == ' ' || *w == '\t').is_some() {}
                if chars.peek() == Some(&'=') {
                    if let Ok(id) = id.parse() {
                        spot.entity = Some((id, at));
                    }
                }
            }
            None => {}
        }
        if c == '\n' {
            if at >= line {
                break;
            }
            at += 1;
            fresh = true;
        } else if !c.is_whitespace() {
            fresh = false;
        }
    }
    spot
}

/// `solid #12 'bracket'`, or `solid #12` for a solid without a name.
#[cfg(not(target_arch = "wasm32"))]
fn solid_label(item: u64, name: &str) -> String {
    if name.is_empty() {
        format!("solid #{item}")
    } else {
        format!("solid #{item} '{name}'")
    }
}
