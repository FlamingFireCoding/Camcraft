//! Provenance: our own swept orthonormal caps and analytic line/circle intersections, from elementary Euclidean geometry.

use crate::geometry::Point3;

/// Local outward bisector and cross-edge direction, one frame per segment.
/// Together with the segment tangent these must be orthonormal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreakFrame {
    pub outward: [f64; 3],
    pub across: [f64; 3],
}

/// Finished cross-section in bisector coordinates. Width is the full
/// cross-edge span; depth is the maximum inward distance from the edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BreakProfile {
    Flat,
    ConcaveBall { radius: f64 },
    ConvexRound { radius: f64 },
}

/// A finite cap swept along a polyline. The actual part clips the cap:
/// at across coordinate s, only 0 <= inward <= profile_depth(s) is
/// declared. End planes stop at the selected segment endpoints.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeBreakVolume {
    pub width: f64,
    pub frames: Vec<BreakFrame>,
    pub profile: BreakProfile,
}

impl EdgeBreakVolume {
    pub(super) fn rotate(&mut self, frame: &super::TiltedFrame) {
        let rotate = |v: [f64; 3]| std::array::from_fn(|i| {
            v[0] * frame.u[i] + v[1] * frame.v[i] + v[2] * frame.axis[i]
        });
        for f in &mut self.frames {
            f.outward = rotate(f.outward);
            f.across = rotate(f.across);
        }
    }

    pub(super) fn is_sane(&self, edge: &[Point3], depth: f64) -> bool {
        if !self.width.is_finite() || self.width <= 0.0 || depth > self.width
            || edge.len() < 2 || self.frames.len() != edge.len() - 1
        {
            return false;
        }
        if let Some(r) = self.radius()
            && (!r.is_finite() || r < self.width / 2.0 || r <= 0.0 || sag(self.width / 2.0, r) > depth)
        {
            return false;
        }
        edge.windows(2).zip(&self.frames).all(|(seg, f)| {
            let delta = seg[1] - seg[0];
            let len = delta.x.hypot(delta.y).hypot(delta.z);
            if !len.is_finite() || len <= 0.0 || !(len + self.width + depth).is_finite() {
                return false;
            }
            let tangent = [delta.x / len, delta.y / len, delta.z / len];
            unit(f.outward) && unit(f.across)
                && dot(f.outward, f.across).abs() <= 1e-9
                && dot(f.outward, tangent).abs() <= 1e-9
                && dot(f.across, tangent).abs() <= 1e-9
        })
    }

    fn radius(&self) -> Option<f64> {
        match self.profile {
            BreakProfile::Flat => None,
            BreakProfile::ConcaveBall { radius } | BreakProfile::ConvexRound { radius } => Some(radius),
        }
    }

    fn profile_depth(&self, s: f64, depth: f64) -> f64 {
        match self.profile {
            BreakProfile::Flat => depth,
            BreakProfile::ConcaveBall { radius } => depth - sag(s, radius),
            BreakProfile::ConvexRound { radius } => depth - sag(self.width / 2.0, radius) + sag(s, radius),
        }
    }

    pub(super) fn floor_below(&self, edge: &[Point3], depth: f64, top: Point3) -> Option<f64> {
        let mut spans = Vec::new();
        for (seg, f) in edge.windows(2).zip(&self.frames) {
            self.column_spans(seg[0], seg[1], *f, depth, top, &mut spans);
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut bottom: f64 = 0.0;
        for (lo, hi) in spans {
            // Only roundoff in vertical coordinates and distances may
            // join spans. Include world Z for translated/rotated caps;
            // unrelated width and depth must not bridge a real gap.
            let slack = 64.0 * f64::EPSILON * top.z.abs().max(lo.abs()).max(bottom);
            if lo > bottom + slack {
                break;
            }
            bottom = bottom.max(hi);
        }
        let floor = top.z - bottom;
        (bottom > 0.0 && floor.is_finite()).then_some(floor)
    }

    fn column_spans(&self, a: Point3, b: Point3, f: BreakFrame, depth: f64, top: Point3, spans: &mut Vec<(f64, f64)>) {
        let delta = b - a;
        let len = delta.x.hypot(delta.y).hypot(delta.z);
        let tangent = [delta.x / len, delta.y / len, delta.z / len];
        let q = top - a;
        let q = [q.x, q.y, q.z];
        let (s, ds) = (dot(q, f.across), -f.across[2]);
        let (d, dd) = (-dot(q, f.outward), f.outward[2]);
        let half = self.width / 2.0;
        let mut bounds = (0.0, len + self.width + depth);
        if !clip(&mut bounds, dot(q, tangent), -tangent[2], 0.0, len)
            || !clip(&mut bounds, s, ds, -half, half)
            || !clip(&mut bounds, d, dd, 0.0, depth)
        {
            return;
        }
        let mut cuts = vec![bounds.0, bounds.1];
        let speed = ds.hypot(dd);
        if let Some(radius) = self.radius().filter(|_| speed > 0.0) {
            let (base, sign) = match self.profile {
                BreakProfile::ConcaveBall { .. } => (depth, 1.0),
                _ => (depth - sag(half, radius), -1.0),
            };
            // Expand the circle about its surface, not its distant
            // centre: subtracting R from a shallow sag loses the cut.
            // Solve in cross-section distance, not column height:
            // squaring tiny world-Z rates can underflow to zero.
            let (rs, rd) = (ds / speed, dd / speed);
            let e = d - base;
            let aa = (rs * rs + rd * rd) / radius;
            let bb = 2.0 * (s / radius * rs + e / radius * rd + sign * rd);
            let cc = e * (e / radius + 2.0 * sign) + s * (s / radius);
            if !(aa.is_finite() && bb.is_finite() && cc.is_finite()) {
                return; // ill-conditioned records grant no credit
            }
            let mut roots = Vec::new();
            if !circle_roots(aa, bb, cc, (bounds.0 * speed, bounds.1 * speed), &mut roots) {
                return;
            }
            cuts.extend(roots.into_iter().map(|distance| distance / speed));
        }
        cuts.sort_by(f64::total_cmp);
        for pair in cuts.windows(2) {
            let (lo, hi) = (pair[0], pair[1]);
            let mid = lo + (hi - lo) / 2.0;
            if hi > lo && d + dd * mid <= self.profile_depth(s + ds * mid, depth) {
                spans.push((lo, hi));
            }
        }
    }
}

fn unit(v: [f64; 3]) -> bool {
    v.iter().all(|x| x.is_finite()) && (dot(v, v) - 1.0).abs() <= 1e-9
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Stable circle sag, including R much larger than the break width.
fn sag(s: f64, radius: f64) -> f64 {
    let t = (s.abs() / radius).min(1.0);
    s.abs() * t / (1.0 + ((1.0 - t) * (1.0 + t)).sqrt())
}

fn clip(bounds: &mut (f64, f64), value: f64, rate: f64, min: f64, max: f64) -> bool {
    if !value.is_finite() {
        return false;
    }
    if rate == 0.0 {
        return value >= min && value <= max;
    }
    let (a, b) = ((min - value) / rate, (max - value) / rate);
    bounds.0 = bounds.0.max(a.min(b));
    bounds.1 = bounds.1.min(a.max(b));
    bounds.0.is_finite() && bounds.1.is_finite() && bounds.1 > bounds.0
}

fn circle_roots(a: f64, b: f64, c: f64, bounds: (f64, f64), cuts: &mut Vec<f64>) -> bool {
    // Roots are unchanged by a common scale. Normalize before products:
    // finite, tiny coefficients can otherwise lose a positive discriminant.
    let scale = a.abs().max(b.abs()).max(c.abs());
    if !scale.is_finite() {
        return false;
    }
    if scale == 0.0 {
        return true;
    }
    let (aa, bb, cc) = (a / scale, b / scale, c / scale);
    if (a != 0.0 && aa == 0.0) || (b != 0.0 && bb == 0.0) || (c != 0.0 && cc == 0.0) {
        return false; // lost coefficients cannot safely grant credit
    }
    let (a, b, c) = (aa, bb, cc);
    let mut take = |h: f64| {
        if h.is_finite() && h > bounds.0 && h < bounds.1 {
            cuts.push(h);
        }
    };
    if a == 0.0 {
        if b != 0.0 {
            take(-c / b);
        }
        return true;
    }
    let square = b * b;
    let product = 4.0 * a * c;
    if (b != 0.0 && square == 0.0) || (a != 0.0 && c != 0.0 && product == 0.0) {
        return false;
    }
    let disc = square - product;
    if !disc.is_finite() {
        return false;
    }
    if disc < 0.0 {
        return true;
    }
    let q = -0.5 * (b + disc.sqrt().copysign(b));
    if q != 0.0 {
        take(q / a);
        take(c / q);
    } else {
        take(-b / (2.0 * a));
    }
    true
}
