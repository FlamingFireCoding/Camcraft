//! Match a selected rim to one through-opening of a retained die plate for the native wire EDM
//! job: one opening (#2957), or one of the two openings of a die group (#2977).
//! Provenance: CAMCraft's Wire Contour geometry and its closed vertical prism.

use super::job;
use crate::geo2d::{Loop, Region};

/// The through-opening of the retained die plate that the selected rim
/// matches. The plate must have exactly `openings` through-openings, one for
/// each work of the job. Reading only the picked loop could post a cut
/// through a blind floor or on an unrelated face, so compare it with the
/// full-height holes of the prism.
pub(super) fn die_opening<'r>(
    region: &'r Region,
    job: &job::GenJob,
    lo: [f64; 2],
    hi: [f64; 2],
    openings: usize,
) -> Result<&'r Loop, String> {
    let loops = region.loops();
    if loops.len() != openings + 1 || loops.iter().filter(|r| r.area() > 0.0).count() != 1 {
        return Err(match openings {
            1 => "The native wire EDM job requires one retained die plate with exactly one through-opening".into(),
            n => format!("The die group requires one retained die plate with exactly {n} through-openings, one for each work"),
        });
    }
    if loops
        .iter()
        .flat_map(|r| r.points())
        .any(|p| p.x < lo[0] || p.x > hi[0] || p.y < lo[1] || p.y > hi[1])
    {
        return Err("The retained die plate must fit inside the stock".into());
    }
    let (picked, open, _) = job
        .contour_sel
        .as_ref()
        .ok_or("Select the complete rim of the die opening")?
        .as_ref()
        .map_err(ToString::to_string)?;
    let [picked] = picked.as_slice() else {
        return Err("Select the edge chains of exactly one closed die opening rim".into());
    };
    let footprint = |r: &Loop| Region::new(vec![r.clone()]).map_err(|e| e.to_string());
    let selection = footprint(picked)?;
    let mut matched = None;
    for hole in loops.iter().filter(|r| r.area() < 0.0) {
        let candidate = footprint(hole)?;
        if open.is_empty()
            && candidate.difference(&selection).is_ok_and(|r| r.is_empty())
            && selection.difference(&candidate).is_ok_and(|r| r.is_empty())
        {
            if matched.is_some() {
                return Err(
                    "The selected rim matches more than one opening of the die plate".into(),
                );
            }
            matched = Some(hole);
        }
    }
    matched.ok_or_else(|| {
        "The selected chains must match the full through-opening of the retained die plate".into()
    })
}
