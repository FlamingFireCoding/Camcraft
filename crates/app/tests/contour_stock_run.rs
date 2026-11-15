//! A Contour on an open run picked on the STOCK entity (#1928): the tool
//! never rides the picked line silently. The run obeys the Side setting -
//! Outside stands the tool one radius plus the stock to leave off the
//! stock, Inside one radius into it, Auto away from the part - and a run
//! with no side to read, or a tool that would enter the part, is refused
//! by name. Before the fix the tool centre rode the picked line.
//!
//! Gate map (#1928): covers app::operation (generate), motion
//! machining2d (contour_run_stock, contour_run_side), app::posting.

mod area_support;

use area_support::*;
use camcraft_app::commands::GenerateOperation;
use camcraft_app::posting;

/// The open chain of `entity`'s top rim at `top` whose points all
/// satisfy `pick`.
fn rim_of(world: &World, entity: &str, top: f64, pick: impl Fn(&[f64; 3]) -> bool) -> FeatureSel {
    let e = model::find_by_name(world, entity).expect("model");
    let f = model::features_of(world, &e).expect("features");
    feature::chains(&f)
        .into_iter()
        .find(|c| !c.closed && {
            let p = feature::chain_points(&f, c);
            p.len() >= 2 && p.iter().all(|q| (q[2] - top).abs() < 1e-9 && pick(q))
        })
        .map(|c| FeatureSel::Chain(c.id))
        .expect("the rim edge is its own open chain")
}

fn stock_name(world: &World) -> String {
    camcraft_app::setup::stock_entity_name(world, None)
}

/// The stock's north top edge (y = the stock's max y) and that y.
fn stock_north(world: &World) -> (Vec<GeoSel>, f64) {
    let (lo, hi) = stock_box(world);
    let name = stock_name(world);
    let f = rim_of(world, &name, hi.z, |q| (q[1] - hi.y).abs() < 1e-9 && q[0] > lo.x - 1e-9);
    (pick(&name, f), hi.y)
}

/// Generate "C1" from `picks` with `side`, `leave` stock to leave.
fn generate(world: &World, picks: Vec<GeoSel>, side: Side, leave: f64) -> Result<(Toolpath, usize), String> {
    let mut p = contour(10.0, 0.0, side);
    if let OpParams::Contour { stock_to_leave, .. } = &mut p {
        *stock_to_leave = leave;
    }
    let o = op(SrcRef::Selection(picks), p, AreaControls::default());
    run(world, &o)
}

/// The cutting feeds below the top (Z 10): where the tool works.
fn cutting(t: &Toolpath) -> Vec<Point3> {
    feeds(t).into_iter().filter(|p| p.z < 10.0 - EPS).collect()
}

/// The XY distances (min, max) of `pts` to the line y = `y`.
fn span(pts: &[Point3], y: f64) -> (f64, f64) {
    let d: Vec<f64> = pts.iter().map(|p| (p.y - y).abs()).collect();
    (d.iter().copied().fold(f64::INFINITY, f64::min), d.iter().copied().fold(f64::NEG_INFINITY, f64::max))
}

#[test]
fn a_stock_run_is_cut_one_radius_off_the_line_by_its_side() {
    let w = block_world();
    let (picks, y) = stock_north(&w);
    // Outside and Auto (the part lies south): north of the line.
    // Inside: south of it, into the stock.
    for (side, sign) in [(Side::Auto, 1.0), (Side::Outside, 1.0), (Side::Inside, -1.0)] {
        let (t, warnings) = generate(&w, picks.clone(), side, 0.0).unwrap_or_else(|e| panic!("{side:?}: {e}"));
        let f = cutting(&t);
        assert!(!f.is_empty(), "{side:?}: the run is cut");
        let (lo, hi) = span(&f, y);
        assert!((lo - R).abs() < EPS && (hi - R).abs() < EPS, "{side:?}: min {lo:.4} max {hi:.4} off the line, want {R}");
        assert!(f.iter().all(|p| (p.y - y) * sign > 0.0), "{side:?}: on the wrong side: {f:?}");
        assert_eq!(warnings, 0, "{side:?}: nothing gouges");
        assert!(t.notes.iter().any(|n| n.contains("open run on the stock")), "{side:?}: {:?}", t.notes);
    }
}

#[test]
fn stock_to_leave_moves_the_stock_run_further_off() {
    let w = block_world();
    let (picks, y) = stock_north(&w);
    let (t, _) = generate(&w, picks, Side::Outside, 0.5).expect("outside");
    let (lo, hi) = span(&cutting(&t), y);
    assert!((lo - (R + 0.5)).abs() < EPS && (hi - (R + 0.5)).abs() < EPS, "min {lo:.4} max {hi:.4}");
}

#[test]
fn a_stock_run_posts_clean() {
    let w = block_world();
    let (picks, _) = stock_north(&w);
    let mut h = History::default();
    let p = contour(10.0, 0.0, Side::Inside);
    h.execute(&w, CreateOperation::new(&w, "C1", TOOL, SrcRef::Selection(picks), p).expect("create"));
    h.execute(&w, GenerateOperation::new(&w, "C1").expect("generate"));
    let e = model::find_by_name(&w, "C1").expect("op");
    assert!(model::toolpath_of(&w, &e).is_some(), "a toolpath");
    assert!(posting::post_setup_unwritten(&w).is_ok(), "{:?}", posting::post_setup_unwritten(&w).err());
}

#[test]
fn auto_on_a_stock_run_with_no_part_is_refused_by_name() {
    let w = world_with(None, 0.0, 10.0);
    let (picks, _) = stock_north(&w);
    let e = generate(&w, picks.clone(), Side::Auto, 0.0).expect_err("no part to cut away from");
    assert!(e.contains("Inside") && e.contains("Outside"), "{e}");
    // An explicit side is enough.
    let (t, _) = generate(&w, picks, Side::Outside, 0.0).expect("outside");
    assert!(!cutting(&t).is_empty());
}

#[test]
fn a_stock_run_whose_tool_enters_the_part_is_refused_by_name() {
    // A 44 x 44 stock round the 40 x 40 part: one radius into the stock
    // from its north edge (y 22) reaches y 12 - inside the part.
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("part", cube(Point3::new(-20.0, -20.0, 0.0), Point3::new(20.0, 20.0, 10.0))));
    let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), ..Default::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 44.0, dy: 44.0, dz: 10.0 }, loc).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    let (picks, _) = stock_north(&world);
    let e = generate(&world, picks.clone(), Side::Inside, 0.0).expect_err("into the part");
    assert!(e.contains("cuts the part"), "{e}");
    // Away from the part stays legal.
    assert!(generate(&world, picks, Side::Auto, 0.0).is_ok());
}
