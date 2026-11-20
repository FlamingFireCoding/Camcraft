//! Quantize the nominal polygon and find an exterior perpendicular Agie punch entry (planar geometry).
use super::AgieJob;

type Point = [i64; 2];
const UNITS: f64 = 100_000.0; // Five decimal places in the native ISO geometry file.
const MAX_VERTICES: usize = 4096;
// CCSEP20's 20 mm QN2 rough pass stands 0.245 mm outside each rectangular
// face. Include the 0.125 mm wire radius and round the guard out to 0.4 mm.
const STOCK_MARGIN: i64 = 40_000;

fn quantize(v: f64) -> Result<i64, String> {
    if !v.is_finite() || v.abs() > 9999.9999 {
        return Err("Agie coordinates must be finite and within ±9999.9999 mm".into());
    }
    Ok((v * UNITS).round() as i64)
}

fn cross(a: Point, b: Point, p: Point) -> i128 {
    i128::from(b[0] - a[0]) * i128::from(p[1] - a[1])
        - i128::from(b[1] - a[1]) * i128::from(p[0] - a[0])
}

fn between(a: Point, b: Point, p: Point) -> bool {
    (0..2).all(|d| (a[d].min(b[d])..=a[d].max(b[d])).contains(&p[d]))
}

fn intersects(a: Point, b: Point, c: Point, d: Point) -> bool {
    let (u, v, w, z) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (u.signum() * v.signum() == -1 && w.signum() * z.signum() == -1)
        || (u == 0 && between(a, b, c))
        || (v == 0 && between(a, b, d))
        || (w == 0 && between(c, d, a))
        || (z == 0 && between(c, d, b))
}

fn check_stock(job: &AgieJob) -> Result<(), String> {
    for &v in job.stock_min.iter().chain(&job.stock_max).chain(&job.start) {
        quantize(v)?;
        if (v - (v * 1000.0).round() / 1000.0).abs() > 1e-9 {
            return Err(
                "Agie stock and start-hole coordinates must be representable to 0.001 mm".into(),
            );
        }
    }
    let [width, depth, height] = std::array::from_fn(|i| job.stock_max[i] - job.stock_min[i]);
    if !(width > 0.0 && width <= 350.0 && depth > 0.0 && depth <= 250.0) {
        return Err("Agie stock must fit the stated 350 × 250 mm working span".into());
    }
    if job.stock_max[2] != 0.0 || height != 20.0 {
        return Err(
            "The Agie Berco Fine Ra0.30 profile requires stock top Z0 and thickness 20 mm".into(),
        );
    }
    if !(0..2)
        .all(|i| job.start[i] - 1.0 >= job.stock_min[i] && job.start[i] + 1.0 <= job.stock_max[i])
    {
        return Err("The complete 2 mm Agie start hole must lie inside the stock".into());
    }
    Ok(())
}

pub(super) fn prepare(job: &AgieJob) -> Result<Vec<[f64; 2]>, String> {
    check_stock(job)?;
    let input = &job.contour;
    if input.len() < 3 || input.len() > MAX_VERTICES + 1 {
        return Err(format!(
            "Agie needs one closed nominal polygon with 3–{MAX_VERTICES} vertices"
        ));
    }
    let mut ring = Vec::with_capacity(input.len());
    for &[x, y] in input {
        let p = [quantize(x)?, quantize(y)?];
        if !(0..2).all(|i| {
            let value = p[i] as f64 / UNITS;
            value >= job.stock_min[i] && value <= job.stock_max[i]
        }) {
            return Err("Agie nominal contour extends outside the stock".into());
        }
        ring.push(p);
    }
    if input.first() == input.last() {
        ring.pop();
    }
    let n = ring.len();
    if !(3..=MAX_VERTICES).contains(&n) {
        return Err("Agie contour vertex count exceeds the supported range".into());
    }
    // Validate quantized segments, including the closing edge, before choosing a start point.
    // Keep collinear intermediate vertices out of the search for the perpendicular entry edge.
    let mut outline = Vec::with_capacity(n);
    for i in 0..n {
        let a = ring[(i + n - 1) % n];
        let b = ring[i];
        let c = ring[(i + 1) % n];
        if a == b || (cross(a, b, c) == 0 && !between(a, c, b)) {
            return Err("Agie nominal contour contains a collapsed or reversed edge".into());
        }
        if cross(a, b, c) != 0 {
            outline.push(b);
        }
    }
    let n = outline.len();
    let mut area = 0_i128;
    for i in 0..n {
        let (a, b) = (outline[i], outline[(i + 1) % n]);
        area += cross([0, 0], a, b);
        for j in (i + 2)..n {
            if i == 0 && j + 1 == n {
                continue;
            }
            if intersects(a, b, outline[j], outline[(j + 1) % n]) {
                return Err("Agie nominal contour crosses or touches itself".into());
            }
        }
    }
    if area == 0 {
        return Err("Agie nominal contour has no enclosed area".into());
    }
    if area < 0 {
        outline.reverse();
    }
    check_punch_envelope(&outline, job)?;
    let hole = [quantize(job.start[0])?, quantize(job.start[1])?];
    let (edge, foot) = entry_face(&outline, hole).ok_or(
        "Agie needs the start hole at least 3 mm outside a straight face, with at least 3 mm of straight face on both sides of the perpendicular entry for the lead and separation"
    )?;
    // Begin the nominal program at the middle of the chosen edge, facing along that edge. The
    // hole and the full three-millimetre lead share its exterior half-plane. Compensation is
    // deliberately left to the Agie technology. The closing move returns to the inserted point.
    outline.rotate_left((edge + 1) % n);
    let mut vertices = vec![foot];
    vertices.extend(outline);
    Ok(vertices
        .into_iter()
        .map(|p| [p[0] as f64 / UNITS, p[1] as f64 / UNITS])
        .collect())
}

/// Check the swept wire against the stock before Agie chooses its corner joins.
/// A round join lies within the rough offset of its nominal vertex. A miter
/// reaches the intersection of the two parallel, outward-offset face lines.
/// The segments connecting adjacent joins stay inside their stock bounds.
fn check_punch_envelope(ring: &[Point], job: &AgieJob) -> Result<(), String> {
    const ROUGH_OFFSET: f64 = 24_500.0; // 0.245 mm in the ISO lattice.
    let wire_and_allowance = STOCK_MARGIN as f64 - ROUGH_OFFSET;
    let lo = [quantize(job.stock_min[0])?, quantize(job.stock_min[1])?];
    let hi = [quantize(job.stock_max[0])?, quantize(job.stock_max[1])?];
    let n = ring.len();
    let outward = |a: Point, b: Point| {
        let dx = (b[0] - a[0]) as f64;
        let dy = (b[1] - a[1]) as f64;
        let length = dx.hypot(dy);
        [dy / length, -dx / length]
    };
    for (i, &corner) in ring.iter().enumerate() {
        let previous = ring[(i + n - 1) % n];
        let next = ring[(i + 1) % n];
        if cross(previous, corner, next) <= 0 {
            return Err(
                "Agie requires a convex punch contour; a reentrant corner is unsupported".into(),
            );
        }
        let a = outward(previous, corner);
        let b = outward(corner, next);
        // Use the squared bisector length instead of 1 + dot(a, b);
        // the latter loses precision when the punch corner is very sharp.
        let bisector = [a[0] + b[0], a[1] + b[1]];
        let norm2 = bisector[0] * bisector[0] + bisector[1] * bisector[1];
        if norm2 <= f64::EPSILON {
            return Err(
                "Agie cannot bound the rough pass at a nearly reversed punch corner".into(),
            );
        }
        for axis in 0..2 {
            if corner[axis] - STOCK_MARGIN < lo[axis] || corner[axis] + STOCK_MARGIN > hi[axis] {
                return Err(
                    "The Agie rough pass and wire need at least 0.4 mm of stock beyond the punch"
                        .into(),
                );
            }
            let miter = 2.0 * ROUGH_OFFSET * bisector[axis] / norm2;
            let left = (lo[axis] - corner[axis]) as f64;
            let right = (hi[axis] - corner[axis]) as f64;
            if miter - wire_and_allowance < left || miter + wire_and_allowance > right {
                return Err(
                    "The Agie rough corner and wire can reach outside the stock at a punch vertex"
                        .into(),
                );
            }
        }
    }
    Ok(())
}

fn entry_face(ring: &[Point], hole: Point) -> Option<(usize, Point)> {
    let mut best = None;
    let mut nearest = f64::INFINITY;
    for (i, &a) in ring.iter().enumerate() {
        let b = ring[(i + 1) % ring.len()];
        let side = cross(a, b, hole);
        if side >= 0 || ring.iter().any(|&p| cross(a, b, p) < 0) {
            continue;
        }
        let dx = (b[0] - a[0]) as f64;
        let dy = (b[1] - a[1]) as f64;
        let length = dx.hypot(dy);
        let distance = -(side as f64) / length;
        if distance < 3.0 * UNITS || distance >= nearest {
            continue;
        }
        let t =
            (((hole[0] - a[0]) as f64) * dx + ((hole[1] - a[1]) as f64) * dy) / (length * length);
        if !(0.0 < t && t < 1.0) {
            continue;
        }
        let foot = [
            (a[0] as f64 + t * dx).round() as i64,
            (a[1] as f64 + t * dy).round() as i64,
        ];
        // Room on each side for the control's 3 mm transition and 1 mm
        // separation, independent of which way the resident pass runs.
        let min_face = i128::from(300_000).pow(2);
        if cross(a, b, foot) != 0
            || [a, b].iter().any(|p| {
                let dx = i128::from(foot[0] - p[0]);
                let dy = i128::from(foot[1] - p[1]);
                dx * dx + dy * dy < min_face
            })
        {
            continue;
        }
        nearest = distance;
        best = Some((i, foot));
    }
    best
}
