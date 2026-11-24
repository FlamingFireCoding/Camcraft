//! Native job export of two captured punches on one plate, from the Agie Processor work format
//! and CamCraft's offline control checks of #2961.

use std::path::Path;

use super::{
    AgieJob, AgieSide, ExportedJob, contour, export_with_review, iso, publish, valid_name,
};
use crate::agie_setup::{GROUP_HEIGHT_UM, GroupPunch, PunchGroup20, Quality};

/// The 3 mm lead from the threading hole to the contour (`ID_COMMPOINTENTRY`).
const LEAD_MM: f64 = 3.0;
/// The radius of the 0.25 mm Berco Cut wire.
const WIRE_RADIUS_MM: f64 = 0.125;

/// The nominal ISO of one work and the XY box that its complete cutting area
/// cannot leave.
struct Work {
    geometry: String,
    points: Vec<[f64; 2]>,
    start_um: [i32; 2],
    min: [f64; 2],
    max: [f64; 2],
}

/// Publish one work as [`export_with_review`] does, or a group of two
/// independently captured punches on one 20 mm plate.
///
/// A group writes one ISO for each work (`<program>1.ISO`, `<program>2.ISO`),
/// one SBR with a resource line for each ISO, and one SBL with the works
/// `CUT1` and `CUT2` in list order. A single work keeps its file layout.
///
/// # Errors
/// Refuses unsupported groups, and cutting areas that are not separated by a
/// clear band along X or Y, before it publishes the directory.
pub fn export_group_with_review(
    jobs: &[AgieJob],
    destination: &Path,
    program: &str,
    separation_length_mm: f64,
) -> Result<ExportedJob, String> {
    let first = match jobs {
        [] => return Err("The job requires an active Wire Contour".into()),
        [job] => return export_with_review(job, destination, program, separation_length_mm),
        [first, ..] => first,
    };
    if separation_length_mm != 1.0 {
        return Err("Set the separation cut length (separationLengthMm) to 1.0 mm for this closed contour. A zero or unspecified separation cannot pass the control formal check".into());
    }
    if !valid_name(program) {
        return Err("The job program name must be a non-reserved 1–8 character DOS name starting with a letter and using uppercase letters or digits".into());
    }
    let height = f64::from(GROUP_HEIGHT_UM) / 1000.0;
    if jobs.len() != 2
        || jobs.iter().any(|job| {
            job.side != AgieSide::Punch
                || job.stock_min != first.stock_min
                || job.stock_max != first.stock_max
                || job.stock_max[2] != 0.0
                || job.stock_min[2] != -height
        })
    {
        return Err(
            "The punch group requires two independent closed punches on the same 20 mm stock"
                .into(),
        );
    }
    if program.len() > 7 {
        return Err("Choose a punch group name of up to seven characters, so that each work has a distinct geometry file name".into());
    }
    let names: Vec<String> = (1..=jobs.len()).map(|n| format!("{program}{n}")).collect();
    if !names.iter().all(|name| valid_name(name)) {
        return Err("Choose a punch group name that gives non-reserved geometry file names".into());
    }
    let mut works = Vec::with_capacity(jobs.len());
    let mut stock = None;
    for (job, geometry) in jobs.iter().zip(&names) {
        let (mut points, setup, reach) =
            contour::prepare_with_reach(job, geometry, Quality::Ccb030)?;
        let seam = *points.first().ok_or("The job requires a closed contour")?;
        points.push(seam);
        let entry = job
            .start
            .ok_or("The closed contour requires a predrilled 2 mm start hole")?;
        // The box holds the retained punch, its hole and lead, the longest
        // rough miter (at least the rough offset of this thickness) and the
        // wire radius. Every pass of the work stays inside it.
        let margin = reach + LEAD_MM + WIRE_RADIUS_MM;
        let min = std::array::from_fn(|axis| {
            points.iter().fold(entry[axis], |v, p| v.min(p[axis])) - margin
        });
        let max = std::array::from_fn(|axis| {
            points.iter().fold(entry[axis], |v, p| v.max(p[axis])) + margin
        });
        stock = Some((setup.stock_min_um, setup.stock_max_um, setup.height_um));
        works.push(Work {
            geometry: geometry.clone(),
            points,
            start_um: setup.start_hole_um,
            min,
            max,
        });
    }
    for (i, a) in works.iter().enumerate() {
        for b in &works[i + 1..] {
            if !(0..2).any(|axis| a.max[axis] < b.min[axis] || b.max[axis] < a.min[axis]) {
                return Err("The punch group requires a clear band along X or Y between the complete cutting areas of the punches: each solid, threading hole, lead, rough corner and wire".into());
            }
        }
    }
    let (stock_min_um, stock_max_um, height_um) =
        stock.ok_or("The job requires an active Wire Contour")?;
    let punches: Vec<GroupPunch<'_>> = works
        .iter()
        .map(|work| GroupPunch {
            iso_stem: &work.geometry,
            start_hole_um: work.start_um,
        })
        .collect();
    let setup = PunchGroup20 {
        stem: program,
        stock_min_um,
        stock_max_um,
        height_um,
        punches: &punches,
    }
    .encode()
    .map_err(|error| error.to_string())?;
    let mut files: Vec<(String, String)> = works
        .iter()
        .map(|work| {
            let name = format!("{}.ISO", work.geometry);
            let text = iso(&name, &work.points);
            (name, text)
        })
        .collect();
    files.push((format!("{program}.SBR"), setup.sbr));
    files.push((format!("{program}.SBL"), setup.sbl));
    Ok(ExportedJob {
        files: publish(destination, &files)?,
        controller_review: setup.controller_review,
    })
}
