//! Provenance: analytic 3D break caps over a one-column sharp block (#1891).
//! Gate map: covers camcraft_sim::stock_sim (edge_break, deviate).

use camcraft_sim::cl::{BreakFrame, BreakProfile, EdgeBreak, EdgeBreakVolume};
use camcraft_sim::geometry::{cube, Point3, Triangle};
use camcraft_sim::stock_sim::{credit_breaks, deviate, Dexel};

fn p(x: f64, y: f64, z: f64) -> Point3 { Point3::new(x, y, z) }

fn record(profile: BreakProfile) -> EdgeBreak {
    EdgeBreak {
        edge: vec![p(-5.0, 0.0, 10.0), p(5.0, 0.0, 10.0)], depth: 0.5, slope: 0.0,
        volume: Some(EdgeBreakVolume {
            width: 1.0,
            frames: vec![BreakFrame { outward: [0.0, 0.0, 1.0], across: [0.0, 1.0, 0.0] }], profile,
        }),
    }
}

fn part() -> Vec<Triangle> {
    cube(p(-2.0, -2.0, 0.0), p(2.0, 2.0, 10.0)).facets.to_vec()
}

fn cut(x: f64, y: f64, z: f64) -> Dexel {
    let mut d = Dexel::block(x - 0.0625, y - 0.0625, x + 0.0625, y + 0.0625, 10.0, 0.125).expect("stock");
    d.cut_chord(p(x - 0.1, y, z), p(x + 0.1, y, z), 0.1);
    d
}

#[test]
fn flat_ball_and_multiple_rounded_passes_clear_only_their_declared_profile() {
    for profile in [BreakProfile::Flat, BreakProfile::ConcaveBall { radius: 0.75 }, BreakProfile::ConvexRound { radius: 0.75 }] {
        let b = record(profile);
        for y in [0.0, 0.2, 0.4] {
            let floor = b.floor_below(0.0, y, 10.0).expect("declared cap");
            let mut d = cut(0.0, y, 10.0 - 0.05);
            // Several passes remove the same rounded envelope: credit
            // concerns the final metal, not the pass that reached it.
            for fraction in [0.25, 0.5, 0.75, 1.0] {
                let z = 10.0 + (floor - 10.0) * fraction;
                d.cut_chord(p(-0.1, y, z), p(0.1, y, z), 0.1);
            }
            let facets = part();
            let mut dev = deviate(&d, &facets, 0.01).expect("sharp verdict");
            assert!(dev.gouge_cells > 0);
            assert_eq!(credit_breaks(&mut dev, &d, &facets, std::slice::from_ref(&b)), 1);
            assert!(dev.is_clean(), "profile {:?} at y={y}: {:?}", b.volume, dev);
            assert_eq!(dev.gouge_volume, 0.0);
            assert_eq!(dev.max_gouge_at, None);
            let d = cut(0.0, y, floor - 0.125);
            let mut dev = deviate(&d, &facets, 0.01).expect("deeper verdict");
            credit_breaks(&mut dev, &d, &facets, std::slice::from_ref(&b));
            assert_eq!(dev.gouge_cells, 1, "the profile forgave a real gouge");
            assert!((dev.max_gouge - 0.125).abs() < 1e-9, "{:?}", dev);
        }
    }
}

#[test]
fn vertical_and_oblique_breaks_clear_without_xy_projection() {
    let mut vertical = record(BreakProfile::Flat);
    vertical.edge = vec![p(0.0, 0.0, 8.0), p(0.0, 0.0, 10.0)];
    vertical.volume.as_mut().expect("volume").frames[0].outward = [1.0, 0.0, 0.0];
    let s = 0.5_f64.sqrt();
    let mut oblique = record(BreakProfile::Flat);
    oblique.edge = vec![p(-1.0, 0.0, 9.0), p(1.0, 0.0, 11.0)];
    oblique.volume.as_mut().expect("volume").frames[0].outward = [-s, 0.0, s];
    for (b, x, expected) in [(vertical, -0.25, 8.0), (oblique, 0.0, 10.0 - 0.5 / s)] {
        let d = cut(x, 0.0, expected);
        let facets = part();
        let mut dev = deviate(&d, &facets, 0.01).expect("verdict");
        assert_eq!(dev.gouge_cells, 1);
        assert_eq!(credit_breaks(&mut dev, &d, &facets, &[b]), 1);
        assert!(dev.is_clean(), "{:?}", dev);
    }
}

#[test]
fn undeclared_columns_bosses_and_detached_caps_keep_their_gouges() {
    let b = record(BreakProfile::Flat);
    for (x, y, z) in [(0.0, 0.6, 9.5), (0.0, 0.0, 9.5)] {
        let mut b = b.clone();
        if y == 0.0 {
            for q in &mut b.edge { q.z -= 0.1; }
        }
        let facets = part();
        let d = cut(x, y, z);
        let mut dev = deviate(&d, &facets, 0.01).expect("verdict");
        assert_eq!(credit_breaks(&mut dev, &d, &facets, &[b]), 0);
        assert_eq!(dev.gouge_cells, 1);
        assert!((dev.max_gouge - 0.5).abs() < 1e-9);
    }
    let facets = cube(p(-2.0, -2.0, 0.0), p(2.0, 2.0, 11.0)).facets.to_vec();
    let d = cut(0.0, 0.0, 9.5);
    let mut dev = deviate(&d, &facets, 0.01).expect("boss verdict");
    assert_eq!(credit_breaks(&mut dev, &d, &facets, &[b]), 0);
    assert!((dev.max_gouge - 1.5).abs() < 1e-9);
}

#[test]
fn malformed_volume_and_mismatched_lattice_cannot_credit() {
    let d = cut(0.0, 0.0, 9.5);
    let facets = part();
    let mut b = record(BreakProfile::Flat);
    b.depth = 2.0;
    let mut dev = deviate(&d, &facets, 0.01).expect("verdict");
    assert_eq!(credit_breaks(&mut dev, &d, &facets, &[b]), 0);
    assert_eq!(dev.gouge_cells, 1);
    dev.cell *= 2.0;
    assert_eq!(credit_breaks(&mut dev, &d, &facets, &[record(BreakProfile::Flat)]), 0);
    assert_eq!(dev.gouge_cells, 1);
}
