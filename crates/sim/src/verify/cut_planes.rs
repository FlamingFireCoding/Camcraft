//! The cut planes at which law (c) samples the model's island walls (#3615):
//! the cut LEVELS plus the deepest cut, not one plane per feed chord - our own
//! conservative filing rule; the section/tangency model it feeds is stated in
//! `verify`'s module law (rest material, machine-tool handbooks).
//!
//! `Rest::new` keys a cut per feed chord on its `to.z`. A level ladder then
//! holds one plane per level, which is exactly the set the law needs. An
//! interpolated ramp (a dead-end entry, a helix) emits one feed move per
//! thinned vertex, each with its own Z, so the per-chord key turned 3 cut
//! levels into about 1,700 planes: the island sample set and its verdict
//! walk grew with the vertex count, and a program took minutes.
//!
//! The law reads no sample's plane: a wall sliced at any plane can still
//! convict a band, and every reading is a pure function of the sample's XY and
//! the column surface. For a prismatic (vertical-walled) part every plane in a
//! ramp's Z span slices the same walls, so the per-chord planes repeated one
//! XY sample set hundreds of times. Sampling the levels plus the deepest cut
//! reads that same set once.
//!
//! HELD (#3615): this does NOT yet keep the verdicts. The equivalence pin in
//! `reach_tests` is RED: a real wall ramp also gets triangulation-tie wall
//! points from the per-chord planes, and the surviving planes lose them, so
//! part-true `ShankHit` verdicts move. The change is a draft demonstrator and
//! does not merge; `docs/decisions/3615-verify-cut-levels.md` has the numbers.

use crate::cl::{MoveKind, Toolpath};

/// The planes law (c) slices the model at: one per cut level (a feed move
/// whose Z does not change, `|from.z - to.z| <= z_tol`) and the deepest cut
/// the path makes. Sorted and deduplicated; a path with no cutting move gives
/// an empty set.
///
/// `z_tol` may be zero (`VerifyContext::default`): a level move built with
/// the same Z at both ends is still one plane, because an unchanged Z differs
/// by exactly zero. A non-finite `z_tol` reads as zero. Non-finite moves are
/// skipped - they have no plane to slice, and the old per-chord set added no
/// samples for them either (`islands_at` fails closed).
pub(super) fn sampled_cut_planes(path: &Toolpath, z_tol: f64) -> Vec<f64> {
    let tol = if z_tol.is_finite() { z_tol.max(0.0) } else { 0.0 };
    let mut planes: Vec<f64> = Vec::new();
    let mut deepest = f64::INFINITY;
    for m in &path.moves {
        if m.kind == MoveKind::Rapid || !(m.from.is_finite() && m.to.is_finite()) {
            continue;
        }
        deepest = deepest.min(m.from.z.min(m.to.z));
        if (m.to.z - m.from.z).abs() <= tol {
            planes.push(m.to.z);
        }
    }
    // The deepest cut is read whatever its level: a ramp or helix bottoms out
    // below every level move's Z, and a drill's only plane is its floor.
    if deepest.is_finite() {
        planes.push(deepest);
    }
    planes.sort_by(|a, b| a.total_cmp(b));
    planes.dedup();
    planes
}

#[cfg(test)]
mod tests {
    use super::sampled_cut_planes;
    use crate::cl::{Move, MoveKind, Toolpath};
    use crate::geometry::Point3;

    fn feed(a: (f64, f64, f64), b: (f64, f64, f64)) -> Move {
        Move::new(MoveKind::Feed, Point3::new(a.0, a.1, a.2), Point3::new(b.0, b.1, b.2), Some(500.0))
    }

    #[test]
    fn cut_levels_collapse_a_ramp_to_its_deepest_plane() {
        // A ramp of 500 feed moves down Z 9 to 1: no level move, so only the
        // deepest plane (1) is sampled - the ~500 per-chord planes are gone.
        let moves: Vec<Move> = (0..500)
            .map(|k| {
                let (a, b) = (k as f64, (k + 1) as f64);
                feed((10.0 + a * 0.16, 34.0, 9.0 - a * 0.016), (10.0 + b * 0.16, 34.0, 9.0 - b * 0.016))
            })
            .collect();
        let path = Toolpath { moves, ..Toolpath::default() };
        assert_eq!(sampled_cut_planes(&path, 1e-6), vec![1.0]);
    }

    #[test]
    fn cut_levels_keep_every_level_plus_the_deepest() {
        // Four level rings at Z 8, 6, 4, 2 and a plunge to Z 1: the set is the
        // four levels plus the deepest cut.
        let mut moves: Vec<Move> = [8.0, 6.0, 4.0, 2.0]
            .iter()
            .flat_map(|&z| {
                vec![
                    feed((10.0, 40.0, z), (90.0, 40.0, z)),
                    feed((90.0, 40.0, z), (90.0, 60.0, z)),
                ]
            })
            .collect();
        moves.push(Move::new(MoveKind::Plunge, Point3::new(50.0, 50.0, 10.0), Point3::new(50.0, 50.0, 1.0), Some(100.0)));
        let path = Toolpath { moves, ..Toolpath::default() };
        assert_eq!(sampled_cut_planes(&path, 1e-6), vec![1.0, 2.0, 4.0, 6.0, 8.0]);
        // An exact level is still one plane when `z_tol` is zero.
        let level: Vec<Move> = vec![feed((10.0, 40.0, 5.0), (90.0, 40.0, 5.0))];
        let p = Toolpath { moves: level, ..Toolpath::default() };
        assert_eq!(sampled_cut_planes(&p, 0.0), vec![5.0]);
    }

    #[test]
    fn an_empty_path_samples_no_plane() {
        assert_eq!(sampled_cut_planes(&Toolpath::default(), 1e-6), Vec::<f64>::new());
    }
}
