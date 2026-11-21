//! #3153 motion pins for the 2D Chamfer cone-band law.
//!
//! Gate map (#3153): covers motion::machining2d::chamfer,
//! motion::machining2d::chamfer_side,
//! motion::machining2d::contour_run_side, and geometry::geo2d::Region.

use camcraft_motion::cl::MoveKind;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::machining2d::{ChamferParams, GenError, Side, chamfer, chamfer_open_runs};

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Region {
    Region::new(vec![
        Loop::new(vec![
            Point2::new(x0, y0),
            Point2::new(x1, y0),
            Point2::new(x1, y1),
            Point2::new(x0, y1),
        ])
        .expect("loop"),
    ])
    .expect("region")
}

fn params(minimum_cone_radius: f64, side: Side) -> ChamferParams {
    ChamferParams {
        width: 1.0,
        angle_deg: 45.0,
        minimum_cone_radius,
        clearance_z: 25.0,
        retract_z: None,
        top_z: 20.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        inside: side == Side::Inside,
    }
}

fn refusal<T>(result: Result<T, GenError>) -> String {
    match result {
        Ok(_) => panic!("the minimum-radius door should refuse"),
        Err(error) => error.to_string(),
    }
}

fn feed_xy(path: &camcraft_motion::cl::Toolpath) -> Vec<Point2> {
    path.moves
        .iter()
        .filter(|m| m.kind == MoveKind::Feed)
        .map(|m| Point2::new(m.to.x, m.to.y))
        .collect()
}

#[test]
fn radius_two_uses_the_two_to_three_cone_band_at_z_seventeen() {
    let p = params(2.0, Side::Outside);
    let path = chamfer(&rect(0.0, 0.0, 60.0, 60.0), &p).expect("radius band");
    let feeds: Vec<_> = path
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Feed)
        .collect();
    assert!(!feeds.is_empty());
    assert!(
        feeds.iter().all(|m| (m.to.z - 17.0).abs() < 1e-12),
        "the 45-degree cone tip is 3 mm below the top: {feeds:?}"
    );
    let tan = p.angle_deg.to_radians().tan();
    let inner = (p.corner_z() - p.tip_z()) / tan;
    let outer = (p.top_z - p.tip_z()) / tan;
    assert!((inner - 2.0).abs() < 1e-12 && (outer - 3.0).abs() < 1e-12);
    let xy = feed_xy(&path);
    assert!(
        (xy.iter().map(|q| q.x).fold(f64::INFINITY, f64::min) + 2.0).abs() < 1e-6,
        "the cone axis stands 2 mm off the original west wall: {:?}",
        xy
    );
    let b = path.breaks.first().expect("authored edge break");
    assert!((b.depth - 1.0).abs() < 1e-12);
    assert!((b.width() - 1.0).abs() < 1e-12);
    assert!(
        b.edge
            .iter()
            .any(|q| q.x == 0.0 && q.y == 0.0 && q.z == 20.0),
        "the break stays on the original selected loop: {:?}",
        b.edge
    );
}

#[test]
fn a_closed_ring_offsets_toward_the_named_side() {
    let outside = params(2.0, Side::Outside);
    let out = feed_xy(&chamfer(&rect(0.0, 0.0, 20.0, 20.0), &outside).expect("outside"));
    assert!(
        (out.iter().map(|q| q.x).fold(f64::INFINITY, f64::min) + 2.0).abs() < 1e-6,
        "Outside moves the centerline past the west wall: {out:?}"
    );
    let inside = params(2.0, Side::Inside);
    let into = feed_xy(&chamfer(&rect(0.0, 0.0, 20.0, 20.0), &inside).expect("inside"));
    assert!(
        (into.iter().map(|q| q.x).fold(f64::INFINITY, f64::min) - 2.0).abs() < 1e-6,
        "Inside moves the centerline into the selected region: {into:?}"
    );
}

#[test]
fn an_open_chain_offsets_on_its_named_air_side_and_keeps_its_break() {
    let run = vec![Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)];
    let section = |_| Ok(Some(rect(-20.0, -20.0, 20.0, 20.0)));
    let p = params(2.0, Side::Outside);
    let path =
        chamfer_open_runs(std::slice::from_ref(&run), &p, &section).expect("open air offset");
    let cut: Vec<_> = path
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Feed && (m.to.z - 17.0).abs() < 1e-12)
        .collect();
    assert_eq!(cut.len(), 3);
    assert!(
        cut.iter().any(|m| (m.to.y - 22.0).abs() < 1e-6),
        "the selected run stands 2 mm into air: {cut:?}"
    );
    assert_eq!(
        path.breaks[0].edge,
        run.iter()
            .map(|q| camcraft_motion::geometry::Point3::new(q.x, q.y, 20.0))
            .collect::<Vec<_>>(),
        "the offset does not grant credit beyond the picked chain"
    );
    let wrong = refusal(chamfer_open_runs(
        std::slice::from_ref(&run),
        &params(2.0, Side::Inside),
        &section,
    ));
    assert!(
        wrong.contains("wrong class") && wrong.contains("Side Inside"),
        "{wrong}"
    );

    let blocked = Region::new(vec![
        rect(-20.0, -20.0, 20.0, 20.0).loops()[0].clone(),
        rect(-10.0, 21.0, 10.0, 25.0).loops()[0].clone(),
    ])
    .expect("wall and air-side material");
    let blocked = refusal(chamfer_open_runs(std::slice::from_ref(&run), &p, &|_| {
        Ok(Some(blocked.clone()))
    }));
    assert!(
        blocked.contains("air side") && blocked.contains("smaller offset"),
        "{blocked}"
    );

    let hole = Region::new(vec![
        Loop::new(vec![
            Point2::new(-30.0, -30.0),
            Point2::new(30.0, -30.0),
            Point2::new(30.0, 30.0),
            Point2::new(-30.0, 30.0),
        ])
        .expect("outer"),
        Loop::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 20.0),
            Point2::new(20.0, 20.0),
            Point2::new(20.0, 0.0),
        ])
        .expect("hole"),
    ])
    .expect("holed section");
    let hole_run = vec![Point2::new(5.0, 20.0), Point2::new(15.0, 20.0)];
    let hole_path = chamfer_open_runs(
        std::slice::from_ref(&hole_run),
        &params(2.0, Side::Inside),
        &|_| Ok(Some(hole.clone())),
    )
    .expect("an inside wall");
    assert!(
        feed_xy(&hole_path)
            .iter()
            .any(|q| (q.y - 18.0).abs() < 1e-6),
        "Inside offsets a clear interior span toward the bounded air of the hole: {:?}",
        feed_xy(&hole_path)
    );
}

#[test]
fn zero_radius_keeps_the_exact_legacy_closed_and_open_paths() {
    let p = ChamferParams {
        width: 2.0,
        minimum_cone_radius: 0.0,
        inside: false,
        ..params(0.0, Side::Auto)
    };
    let closed = chamfer(&rect(0.0, 0.0, 4.0, 4.0), &p).expect("legacy closed");
    assert_eq!(
        feed_xy(&closed),
        vec![
            Point2::new(4.0, 0.0),
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 4.0),
            Point2::new(4.0, 4.0),
        ]
    );
    assert!(
        closed
            .moves
            .iter()
            .filter(|m| m.kind == MoveKind::Feed)
            .all(|m| (m.to.z - 18.0).abs() < 1e-12)
    );
    let run = vec![Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)];
    let open = chamfer_open_runs(std::slice::from_ref(&run), &p, &|_| {
        Ok(Some(rect(-20.0, -20.0, 20.0, 20.0)))
    })
    .expect("legacy open");
    assert_eq!(
        feed_xy(&open),
        vec![
            Point2::new(-20.0, 20.0),
            Point2::new(20.0, 20.0),
            Point2::new(20.0, 22.0),
        ]
    );
}

#[test]
fn minimum_radius_refuses_invalid_and_collapsed_offsets_by_name() {
    for (value, word) in [
        (-0.1, "non-negative"),
        (f64::NAN, "finite"),
        (f64::INFINITY, "finite"),
    ] {
        let why = refusal(chamfer(
            &rect(0.0, 0.0, 20.0, 20.0),
            &params(value, Side::Outside),
        ));
        assert!(
            why.contains("tip offset") && why.contains(word),
            "{why}"
        );
    }
    let why = refusal(chamfer(
        &rect(0.0, 0.0, 4.0, 4.0),
        &params(3.0, Side::Inside),
    ));
    assert!(
        why.contains("disappears") && why.contains("tip offset"),
        "{why}"
    );
}
