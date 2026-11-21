//! Build and validate imported solids with the public solidcore-convert and solidcore B-rep APIs.

use solidcore::mass::integrator::properties::brep_area;
use solidcore::topology::Brep;
use solidcore::topology::brep::validation::{CheckLevel, check_brep};
use solidcore_convert::brep::to_kernel_body;
use solidcore_record::brep::Brep as Record;

use super::StepSolid;
use crate::solidcore_impl::{SolidcoreKernel, VOLUME_ACCURACY, checked, failed};
use crate::solidcore_import::{HEAL_GAP_LIMIT, vertex_gaps};
use crate::{Kernel, Measure};

type Refusal = (&'static str, String);

pub(crate) enum RecordSource {
    Step(u64),
    Solidcraft,
}

struct MeasuredBody {
    brep: Brep,
    measure: Measure,
    area: f64,
    gap: f64,
}

/// Both candidate bodies must pass the same checks. The converter leaves the
/// face-shell cache behind the face-use links; refresh it before checking (#916).
fn measure_body(k: &mut SolidcoreKernel, mut brep: Brep) -> Result<MeasuredBody, Refusal> {
    brep.refresh_cached_topology();
    let mut brep = checked("read_step", brep).map_err(|e| ("check", e.to_string()))?;
    let measure = k
        .measure(&mut brep)
        .map_err(|e| ("measure", e.to_string()))?;
    let area = brep_area(&k.k, &mut brep, VOLUME_ACCURACY)
        .map_err(|e| ("measure", failed("read_step", &e).to_string()))?;
    if !(area.is_finite() && area > 0.0) {
        return Err(("measure", format!("not a positive finite area {area}")));
    }
    let gaps = vertex_gaps(k, &brep).map_err(|e| ("check", e.to_string()))?;
    if gaps.unjudged_edges > 0 {
        return Err((
            "check",
            format!(
                "{} edge(s) without a 3D curve: gaps cannot be judged",
                gaps.unjudged_edges
            ),
        ));
    }
    Ok(MeasuredBody {
        brep,
        measure,
        area,
        gap: gaps.gap,
    })
}

/// Inspect the original topology only after healing removed edges and failed
/// the endpoint limit. Skipping the converter's healer here only constructs
/// the candidate. Its full kernel validity report gates its use (#3252).
fn check_source_topology(
    k: &mut SolidcoreKernel,
    record: &Record,
    healed: MeasuredBody,
) -> Result<(MeasuredBody, String), Refusal> {
    let healed_summary = format!(
        "after healing: {} edges, maximum gap {:.3e} mm",
        healed.brep.edge_list().len(),
        healed.gap
    );
    // The rejected body can be large. Keep only its measurements for the retry.
    drop(healed);
    let source_error = |(stage, why): Refusal| {
        (
            stage,
            format!(
                "{healed_summary} (limit {HEAL_GAP_LIMIT:e} mm); could not check the source topology: {why}"
            ),
        )
    };
    let brep = to_kernel_body(&mut k.k, record, true)
        .map_err(|e| source_error(("build", e.to_string())))?;
    let mut source = measure_body(k, brep).map_err(source_error)?;
    let comparison = format!(
        "before healing: {} edges, maximum vertex-to-endpoint gap {:.3e} mm; {healed_summary}",
        source.brep.edge_list().len(),
        source.gap
    );
    if source.gap > HEAL_GAP_LIMIT {
        return Err((
            "check",
            format!("{comparison} (limit {HEAL_GAP_LIMIT:e} mm)"),
        ));
    }
    let validity = check_brep(&mut k.k, &mut source.brep, CheckLevel::Level2).map_err(|e| {
        (
            "check",
            format!("{comparison}; source validation failed: {e}"),
        )
    })?;
    if let Some(reason) = validity.lines().first() {
        return Err((
            "check",
            format!("{comparison}; source validation failed: {reason}"),
        ));
    }
    Ok((
        source,
        format!("Kept the original STEP edges. {comparison}."),
    ))
}

/// A reader record in mm, measured and checked before the mesher gets it.
pub(crate) fn build_record(
    k: &mut SolidcoreKernel,
    source: RecordSource,
    name: String,
    record: Record,
    mut notes: Vec<String>,
) -> Result<StepSolid, Refusal> {
    let brep = to_kernel_body(&mut k.k, &record, false).map_err(|e| ("build", e.to_string()))?;
    let mut body = measure_body(k, brep)?;
    if body.gap > HEAL_GAP_LIMIT {
        if !matches!(source, RecordSource::Step(_))
            || body.brep.edge_list().len() >= record.edges.len()
        {
            return Err((
                "check",
                format!(
                    "a vertex sits {:.3e} mm off its edges (limit {HEAL_GAP_LIMIT:e})",
                    body.gap
                ),
            ));
        }
        let (original, note) = check_source_topology(k, &record, body)?;
        body = original;
        notes.push(note);
    }
    Ok(StepSolid {
        item: match source {
            RecordSource::Step(item) => item,
            RecordSource::Solidcraft => 0,
        },
        name,
        brep: body.brep,
        record,
        measure: body.measure,
        area: body.area,
        vertex_gap: body.gap,
        notes,
    })
}
