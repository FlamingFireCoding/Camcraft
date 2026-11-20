//! Native ARC records for a capsule proved from DXF primitives; the control offsets its nominal arcs.
use super::{AgieJob, AgieSide, Work, contour};
use camcraft_geometry::{geo2d::Point2, wire_racetrack::Racetrack};

pub(super) fn prepare(job: &AgieJob, shape: Racetrack, geometry: String) -> Result<Work, String> {
    let rough = f64::from(contour::stock_offset_um(job)?) / 1000.;
    if job.side != AgieSide::Punch || job.stock_min[2] != -20. || !job.contour.is_empty() {
        return Err("The Agie DXF racetrack requires one nominal punch on 20 mm stock".into());
    }
    let grid = |n: f64| {
        let value = (n * 100000.).round() / 100000.;
        if n.is_finite() && n.abs() < 10000. && (n - value).abs() < 1e-9 {
            Ok(value)
        } else {
            Err("The placed Agie DXF coordinates must fit the native five-decimal grid".to_owned())
        }
    };
    let point = |p: Point2| -> Result<[f64; 2], String> { Ok([grid(p.x)?, grid(p.y)?]) };
    let [a, b] = shape.centers;
    let [a, b] = [point(a)?, point(b)?];
    let radius = grid(shape.radius)?;
    if !(0.5..=20.).contains(&radius) || (a[0] != b[0] && a[1] != b[1]) || a == b {
        return Err("The native DXF racetrack requires horizontal or vertical straights and a radius from 0.5 to 20 mm".into());
    }
    let entry = job
        .start
        .ok_or("The DXF punch requires a predrilled 2 mm threading hole")?;
    let min: [f64; 2] = std::array::from_fn(|i| a[i].min(b[i]) - radius);
    let max: [f64; 2] = std::array::from_fn(|i| a[i].max(b[i]) + radius);
    if (0..2).any(|i| {
        min[i] - 0.4 < job.stock_min[i]
            || max[i] + 0.4 > job.stock_max[i]
            || entry[i] - 1. < job.stock_min[i]
            || entry[i] + 1. > job.stock_max[i]
    }) {
        return Err(
            "The native DXF punch, complete start hole and rough wire must clear the stock edges"
                .into(),
        );
    }
    let [p, q, r, s] = shape.corners();
    let [p, q, r, s] = [point(p)?, point(q)?, point(r)?, point(s)?];
    for (first, last, around, other, far, near) in [(p, q, r, s, b, a), (r, s, p, q, a, b)] {
        let dx = last[0] - first[0];
        let dy = last[1] - first[1];
        let length = dx.hypot(dy);
        let along = ((entry[0] - first[0]) * dx + (entry[1] - first[1]) * dy) / length;
        let outside = ((entry[0] - first[0]) * dy - (entry[1] - first[1]) * dx) / length;
        if along < 3.0 || length - along < 3.0 || outside < 3.0 {
            continue;
        }
        let seam = if dx == 0. {
            [first[0], entry[1]]
        } else {
            [entry[0], first[1]]
        };
        // The approach is perpendicular to a supporting line of this convex
        // capsule. Its other material lies strictly on the inward side.
        let points = vec![seam, last, around, other, first, seam];
        let arc_centers = vec![None, None, Some(far), None, Some(near), None];
        let margin = rough + 3. + 0.125;
        return Ok(Work {
            side: job.side,
            geometry,
            points,
            arc_centers,
            entry,
            exit: None,
            min: std::array::from_fn(|i| min[i].min(entry[i]) - margin),
            max: std::array::from_fn(|i| max[i].max(entry[i]) + margin),
        });
    }
    Err("The DXF punch needs a perpendicular waste-side start at least 3 mm from a straight and its two cap joints".into())
}
