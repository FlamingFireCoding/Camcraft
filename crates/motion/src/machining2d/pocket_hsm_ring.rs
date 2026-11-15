//! Held probe for issue #1950 (not a pin): the user's D plate
//! (`Milling-test.step`: disc R30 cut flat at y 19.719, bore R12.5) in a
//! 70 x 70 stock, one level, its center area built as the 3-D ladder
//! builds it (`open_sides` with the stock outline, less the part, eroded
//! by r). Prints what the HSM plan does with each piece and with the
//! whole level. Run: `cargo test --release -p camcraft-motion --lib
//! probe_ring -- --ignored --nocapture`. Ours.

use super::*;
use crate::entry::Entry3D;
use crate::geo2d::{Loop, Point2, Region};
use crate::load::DEFAULT_OPTIMAL_LOAD;

fn circle(r: f64, n: usize) -> Vec<Point2> {
    (0..n).map(|k| {
        let a = std::f64::consts::TAU * k as f64 / n as f64;
        Point2::new(r * a.cos(), r * a.sin())
    }).collect()
}

fn part_d() -> Region {
    // Disc R30 cut by the flat y = 19.719 (the STEP's chord).
    let Ok(l) = Loop::new(circle(30.0, 2000)) else { panic!("disc") };
    let Ok(disc) = Region::new(vec![l]) else { panic!("disc") };
    let Ok(h) = Loop::new(vec![Point2::new(-40.0, -40.0), Point2::new(40.0, -40.0), Point2::new(40.0, 19.719_008_6), Point2::new(-40.0, 19.719_008_6)]) else { panic!("half") };
    let Ok(h) = Region::new(vec![h]) else { panic!("half") };
    let Ok(d) = disc.intersect(&h) else { panic!("d") };
    d
}

fn params() -> PocketParams {
    PocketParams {
        blend_radius: None,
        tool_diameter: 10.0,
        stepover: 4.0,
        stock_to_leave: 0.0,
        clearance_z: 40.0,
        retract_z: Some(22.0),
        top_z: 10.0,
        bottom_z: 10.0,
        stepdown: 5.0,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        strategy: PocketStrategy::Adaptive,
        entry: Entry3D::Helix,
        helix_radius: 2.0,
        optimal_load: DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        links: crate::links::Links::ROUGH3D,
    }
}

fn region(pts: Vec<Point2>) -> Region {
    let Ok(l) = Loop::new(pts) else { panic!("loop") };
    let Ok(r) = Region::new(vec![l]) else { panic!("region") };
    r
}

/// The level's center area as the 3-D ladder builds it.
fn level_center() -> Region {
    let r = 5.0;
    let s = 35.0;
    let stock = region(vec![Point2::new(-s, -s), Point2::new(s, -s), Point2::new(s, s), Point2::new(-s, s)]);
    let d = part_d();
    let Ok(foot) = crate::machining3d::open_sides(&d, r, &Region::empty(), Some(&stock)) else { panic!("foot") };
    let bore = region(circle(12.5, 2000));
    let Ok(mat) = d.difference(&bore) else { panic!("mat") };
    let Ok(target) = foot.difference(&mat) else { panic!("target") };
    let Ok(c) = target.offset(-r) else { panic!("center") };
    c
}

#[test]
#[ignore = "probe for issue #1950: prints readings, asserts nothing (~30 s release)"]
fn probe_ring() {
    let ring = level_center();
    for l in ring.loops() {
        let (lo, hi) = l.points().iter().fold((Point2::new(1e9, 1e9), Point2::new(-1e9, -1e9)), |(a, b), q| (Point2::new(a.x.min(q.x), a.y.min(q.y)), Point2::new(b.x.max(q.x), b.y.max(q.y))));
        eprintln!("center loop: {} pts, area {:.1}, box ({:.2},{:.2})-({:.2},{:.2})", l.points().len(), l.area(), lo.x, lo.y, hi.x, hi.y);
    }
    if let Ok(comps) = components(&ring) {
        for (i, c) in comps.iter().enumerate() {
            let t = std::time::Instant::now();
            let Ok((ps, _, rest, _, _, tr, why)) = hsm_passes(c, &params()) else { panic!("piece") };
            eprintln!("piece {i} area {:.0}: {:.1} s, pts {}, rest {rest:.3}, {tr:?}, refused {why:?}", c.area(), t.elapsed().as_secs_f64(), ps.iter().map(|p| p.len()).sum::<usize>());
        }
    }
    let t0 = std::time::Instant::now();
    let Ok(h) = hsm_passes(&ring, &params()) else { panic!("plan") };
    let (passes, notes, rest, _, _, travel, refused) = h;
    let pts: usize = passes.iter().map(|p| p.len()).sum();
    eprintln!("level: {:.1} s, passes {} pts {pts}, rest {rest:.3}, travel {travel:?}, refused {refused:?}", t0.elapsed().as_secs_f64(), passes.len());
    for n in notes {
        eprintln!("  note {n}");
    }
}
