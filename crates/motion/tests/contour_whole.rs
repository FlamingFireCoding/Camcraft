//! Direct kernel pins for whole-model multi-band 2D Contour (#2436):
//! two islands above a common base plate (no double cutting on the deeper
//! island), a counterbore that narrows below Top Z, and fail-closed refusal
//! when a deeper slice cannot be read.
//!
//! Provenance: our own extruded manifold test meshes; no vendor material.

use camcraft_motion::cl::MoveKind;
use camcraft_motion::geo2d::Region;
use camcraft_motion::geometry::{Point3, Triangle, slice};
use camcraft_motion::machining2d::{ContourParams, Lead, Side, contour_whole_bodies};
use camcraft_motion::machining3d::Entry3D;
use camcraft_motion::verify::{Mode, ToolGeom, VerifyContext, Warning, verify};

fn quad(f: &mut Vec<Triangle>, a: Point3, b: Point3, c: Point3, d: Point3) {
    f.push(Triangle::new(a, b, c));
    f.push(Triangle::new(a, c, d));
}

fn cap_poly(f: &mut Vec<Triangle>, outer: &[(f64, f64)], inner: Option<&[(f64, f64)]>, z: f64, up: bool) {
    let p = |q: (f64, f64)| Point3::new(q.0, q.1, z);
    let mut push = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        if up {
            f.push(Triangle::new(p(a), p(b), p(c)));
        } else {
            f.push(Triangle::new(p(a), p(c), p(b)));
        }
    };
    match inner {
        None => {
            for k in 1..outer.len() - 1 {
                push(outer[0], outer[k], outer[k + 1]);
            }
        }
        Some(inn) => {
            assert_eq!(outer.len(), inn.len());
            for k in 0..outer.len() {
                let n = (k + 1) % outer.len();
                push(outer[k], outer[n], inn[n]);
                push(outer[k], inn[n], inn[k]);
            }
        }
    }
}

fn walls(f: &mut Vec<Triangle>, loop_xy: &[(f64, f64)], z0: f64, z1: f64, out: bool) {
    for k in 0..loop_xy.len() {
        let (a, b) = (loop_xy[k], loop_xy[(k + 1) % loop_xy.len()]);
        let (p0, p1) = if out { (a, b) } else { (b, a) };
        quad(
            f,
            Point3::new(p0.0, p0.1, z0),
            Point3::new(p1.0, p1.1, z0),
            Point3::new(p1.0, p1.1, z1),
            Point3::new(p0.0, p0.1, z1),
        );
    }
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> [(f64, f64); 4] {
    [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

/// Plate `[-40..0] x [-20..20]` z `[0..10]` topped by Island A `[-30..-10] x [-10..10]`
/// z `[10..15]`, beside standalone Island B `[20..40] x [-10..10]` z `[0..15]`.
fn two_islands_part() -> Vec<Triangle> {
    let mut f = Vec::new();
    let plate = rect(-40.0, -20.0, 0.0, 20.0);
    let boss_a = rect(-30.0, -10.0, -10.0, 10.0);
    let boss_b = rect(20.0, -10.0, 40.0, 10.0);
    cap_poly(&mut f, &plate, None, 0.0, false);
    walls(&mut f, &plate, 0.0, 10.0, true);
    cap_poly(&mut f, &plate, Some(&boss_a), 10.0, true);
    walls(&mut f, &boss_a, 10.0, 15.0, true);
    cap_poly(&mut f, &boss_a, None, 15.0, true);
    cap_poly(&mut f, &boss_b, None, 0.0, false);
    walls(&mut f, &boss_b, 0.0, 15.0, true);
    cap_poly(&mut f, &boss_b, None, 15.0, true);
    f
}

/// Outer `[-40..40]^2` block z `[0..20]` with counterbore: `[-15..15]^2` z `[10..20]`
/// narrowing to `[-10..10]^2` z `[0..10]`.
fn counterbore_part() -> Vec<Triangle> {
    let mut f = Vec::new();
    let outer = rect(-40.0, -40.0, 40.0, 40.0);
    let upper = rect(-15.0, -15.0, 15.0, 15.0);
    let lower = rect(-10.0, -10.0, 10.0, 10.0);
    walls(&mut f, &outer, 0.0, 20.0, true);
    cap_poly(&mut f, &outer, Some(&upper), 20.0, true);
    walls(&mut f, &upper, 10.0, 20.0, false);
    cap_poly(&mut f, &upper, Some(&lower), 10.0, true);
    walls(&mut f, &lower, 0.0, 10.0, false);
    cap_poly(&mut f, &outer, Some(&lower), 0.0, false);
    f
}

fn params(top: f64, bottom: f64, stepdown: f64) -> ContourParams {
    ContourParams {
        tool_diameter: 6.0,
        stock_to_leave: 0.0,
        clearance_z: top + 10.0,
        retract_z: top + 5.0,
        top_z: top,
        bottom_z: bottom,
        stepdown,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb: true,
        side: Side::Auto,
        lead: Lead::None,
        tolerance: 0.01,
        tab_count: 0,
        tab_width: 5.0,
        tab_height: 1.0,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        links: camcraft_motion::links::Links::PLANAR,
    }
}

fn depth_verify(path: &camcraft_motion::cl::Toolpath, region: &Region, f: &[Triangle], top: f64) -> Vec<Warning> {
    let tool = ToolGeom {
        radius: 3.0,
        nose_radius: 0.0,
        flute_len: 30.0,
        shank_radius: 3.0,
        shank_len: 40.0,
        holder_radius: 20.0,
        holder_len: 60.0,
    };
    verify(
        path,
        &VerifyContext {
            region,
            stock: None,
            model: Some(f),
            tool: &tool,
            cutter: None,
            mode: Mode::Boundary,
            stock_top_z: top,
            clearance_z: top + 10.0,
            stock_to_leave: 0.0,
            gouge_tol: 0.01,
            z_tol: 1e-6,
            finish_tol: None,
            fixtures: &[],
            spindle: None,
            ..Default::default()
        },
    )
}

#[test]
fn two_islands_above_a_plate_continue_without_double_cutting_the_deeper_island() {
    let f = two_islands_part();
    let top_reg = slice::slice(&f, 15.0 - 1e-3).expect("top slice");
    let path = contour_whole_bodies(&top_reg, &f, &[], &params(15.0, 0.0, 2.0)).expect("whole contour");
    assert!(path.all_finite() && path.is_chained());
    assert_eq!(depth_verify(&path, &top_reg, &f, 15.0), Vec::new(), "{:?}", path.notes);

    // Every level 13, 11, 9, 7, 5, 3, 1, 0 must walk Island B (x > 10) exactly once.
    for z in [13.0, 11.0, 9.0, 7.0, 5.0, 3.0, 1.0, 0.0] {
        let len_b: f64 = path
            .moves
            .iter()
            .filter(|m| {
                m.kind == MoveKind::Feed
                    && (m.from.z - z).abs() < 1e-6
                    && (m.to.z - z).abs() < 1e-6
                    && m.from.x > 10.0
                    && m.to.x > 10.0
            })
            .map(|m| (m.to.x - m.from.x).hypot(m.to.y - m.from.y))
            .sum();
        // Offset 20x20 square with r=3 round corners is 4*20 + 4*3*sqrt(2) ~= 97 mm (never 2x).
        assert!(
            (94.0..=102.0).contains(&len_b),
            "z {z}: expected one lap around B (~97 mm), got {len_b:.3}"
        );
    }
    // Lower levels must trace the [-40..0] x [-20..20] plate (x around -43).
    assert!(
        path.moves.iter().any(|m| m.kind == MoveKind::Feed && m.to.z < 1e-6 && m.to.x < -42.5),
        "must finish plate down to Z 0"
    );
}

#[test]
fn counterbore_narrowing_below_top_z_walks_both_upper_and_lower_bores() {
    let f = counterbore_part();
    let top_reg = slice::slice(&f, 20.0 - 1e-3).expect("top slice");
    let path = contour_whole_bodies(&top_reg, &f, &[], &params(20.0, 0.0, 4.0)).expect("whole contour");
    assert!(path.all_finite() && path.is_chained());
    assert_eq!(depth_verify(&path, &top_reg, &f, 20.0), Vec::new(), "{:?}", path.notes);

    let bore_at = |z: f64| -> Option<f64> {
        path.moves
            .iter()
            .filter(|m| {
                m.kind == MoveKind::Feed
                    && (m.from.z - z).abs() < 1e-6
                    && (m.to.z - z).abs() < 1e-6
                    && m.to.x.abs() < 20.0
                    && m.to.y.abs() < 20.0
            })
            .map(|m| m.to.x.abs().max(m.to.y.abs()))
            .reduce(f64::max)
    };
    assert!((bore_at(12.0).expect("upper bore") - 12.0).abs() < 0.05);
    assert!((bore_at(8.0).expect("lower bore") - 7.0).abs() < 0.05);
    assert!((bore_at(0.0).expect("lower bore bottom") - 7.0).abs() < 0.05);
}

#[test]
fn broken_lower_plane_refuses_rather_than_stopping_silently() {
    let mut f = two_islands_part();
    // Dangling sliver at z [2, 6] breaks slicing inside band 1 without touching band 0.
    f.push(Triangle::new(Point3::new(0.0, 0.0, 2.0), Point3::new(10.0, 0.0, 2.0), Point3::new(5.0, 5.0, 6.0)));
    let top_reg = slice::slice(&f, 15.0 - 1e-3).expect("top slice");
    let err = contour_whole_bodies(&top_reg, &f, &[], &params(15.0, 0.0, 4.0))
        .expect_err("must refuse broken lower plane")
        .to_string();
    assert!(err.contains("3.000") || err.contains("7.000"), "{err}");
}

#[test]
fn broken_vertical_sliver_between_levels_refuses_via_check_unread() {
    // A vertical open triangle strictly between cut levels 13 and 11 (z 11.8..12.2):
    // material_overhead_of(11) ignores vertical walls and clean planes, so only
    // ladder's intermediate vertex flank check + check_unread catches it.
    let mut f = two_islands_part();
    f.push(Triangle::new(
        Point3::new(0.0, 0.0, 11.8),
        Point3::new(2.0, 0.0, 11.8),
        Point3::new(1.0, 0.0, 12.2),
    ));
    let top_reg = slice::slice(&f, 15.0 - 1e-3).expect("top slice");
    let err = contour_whole_bodies(&top_reg, &f, &[], &params(15.0, 0.0, 2.0))
        .expect_err("must refuse unread intermediate vertex flank")
        .to_string();
    assert!(err.contains("11.000") && err.contains("cannot section deeper"), "{err}");
}
