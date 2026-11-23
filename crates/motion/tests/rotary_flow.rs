//! Issue #2824, the Rotary Flow kernel: flow lines between picked
//! helical guide chains on a rotor, unwrapped about a STATED axis
//! (the #2772 seam - the axis is a parameter, never fitted), aligned
//! by the nearest whole turn, pushed out radially until the
//! closest-point oracle clears, and linked on the retract cylinders.
//!
//! The rotor fixture is a helical groove in a cylinder (a rotor's
//! flute): the groove's width runs AXIALLY - its two root edge
//! helices answer for `v = k*phi` and `v = k*phi + W_v` at the same
//! azimuth - with the steep helical flanks rising radially at the
//! two v-edges. That is the ground a radially posed ball can ride:
//! a groove whose edges were screw-shifted copies of one helix
//! would be one ramp, not a band.
//!
//! Gate map (M4.4): covers camcraft_motion::multiaxis (rotary_flow,
//! the #2824 kernel) on this crate's own fixtures.
//! Lane shape (M4.4): the default tests run in seconds; the rotor
//! envelope scan rides the heavy lane.

use camcraft_motion::cl::{Move, MoveKind, Toolpath};
use camcraft_motion::geometry::{Point3, Triangle};
use camcraft_motion::multiaxis::axis::Axis;
use camcraft_motion::multiaxis::{rotary_flow, AxisControl, RotaryFlowParams};
use camcraft_motion::tools::{Tool, ToolKind};
use camcraft_sim::stock_sim::dist_seg_tri;

const TAU: f64 = std::f64::consts::TAU;
/// The groove's rim radius.
const R_OUT: f64 = 10.0;
/// The groove's floor radius.
const R_ROOT: f64 = 8.0;
/// The advance of one full turn.
const PITCH: f64 = 12.0;
/// The turns the default groove winds.
const TURNS: usize = 3;
/// Floor vertices per turn of the default groove.
const N_CIRC: usize = 72;
/// The groove's width, in axial stations (the developed width).
const W_V: f64 = 4.0;

fn p3(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// The groove's own law: the near edge rides `v = PITCH * th / TAU`.
fn law(th: f64) -> f64 {
    PITCH * th / TAU
}

/// The rotor's stated axis: world Z through the origin - a
/// parameter, never fitted from the mesh.
fn stated_axis() -> Axis {
    Axis {
        origin: Point3::ZERO,
        dir: p3(0.0, 0.0, 1.0),
    }
}

/// A point of the groove at radius `r`, base azimuth `th`, axial
/// station `v` - the groove's own law is `v = PITCH * th / TAU` at
/// the near edge, `+ W_V` at the far edge.
fn gp(r: f64, th: f64, v: f64) -> Point3 {
    p3(r * th.cos(), r * th.sin(), v)
}

/// The helical groove: a floor strip on the `R_ROOT` cylinder
/// between the two root edge helices (`W_V` of axial width,
/// subdivided across it), and a radial wall on each v-edge rising
/// to the rim - a helicoid, the steep flank the issue's gouge law
/// exists for. An open channel by design: the kernel reads facets
/// as the gouge oracle and the chain ground, never as a wrap.
fn groove(n_circ: usize, turns: usize, phase: f64) -> Vec<Triangle> {
    const FLOOR_COLS: usize = 8;
    let step = TAU / n_circ as f64;
    let mut t = Vec::new();
    for j in 0..n_circ * turns {
        let (th, thn) = (phase + step * j as f64, phase + step * (j + 1) as f64);
        // the floor: root edge A to root edge B, `FLOOR_COLS` v-strips wide
        for jc in 0..FLOOR_COLS {
            let (v0, v1) = (
                law(th) + W_V * jc as f64 / FLOOR_COLS as f64,
                law(th) + W_V * (jc + 1) as f64 / FLOOR_COLS as f64,
            );
            let (v0n, v1n) = (law(thn) + v0 - law(th), law(thn) + v1 - law(th));
            let (a0, b0) = (gp(R_ROOT, th, v0), gp(R_ROOT, th, v1));
            let (a1, b1) = (gp(R_ROOT, thn, v0n), gp(R_ROOT, thn, v1n));
            t.push(Triangle::new(a0, b0, b1));
            t.push(Triangle::new(a0, b1, a1));
        }
        // the walls: root edge up to the rim, at each v-edge
        for base in [0.0, W_V] {
            let (v, vn) = (law(th) + base, law(thn) + base);
            let (r0, r1) = (gp(R_ROOT, th, v), gp(R_OUT, th, v));
            let (r0n, r1n) = (gp(R_ROOT, thn, vn), gp(R_OUT, thn, vn));
            t.push(Triangle::new(r0, r1, r1n));
            t.push(Triangle::new(r0, r1n, r0n));
        }
    }
    t
}

/// The groove's two root edge helices - the picked guide chains,
/// exact mesh vertices (points ON the model by construction).
fn root_pair(n_circ: usize, turns: usize, phase: f64) -> Vec<Vec<Point3>> {
    let step = TAU / n_circ as f64;
    let mut a = Vec::with_capacity(n_circ * turns + 1);
    let mut b = Vec::with_capacity(n_circ * turns + 1);
    for j in 0..=n_circ * turns {
        let th = phase + step * j as f64;
        a.push(gp(R_ROOT, th, law(th)));
        b.push(gp(R_ROOT, th, law(th) + W_V));
    }
    vec![a, b]
}

/// An oblique band for the alignment law: two helices at the same
/// radius, separated `wb` in azimuth and `PITCH * wb / TAU + W_V`
/// in station, spanned by one ruled strip - a legal (s, v) band
/// whose pair DIFFERS in azimuth, so the whole-turn ambiguity is
/// real. No walls: the band is its own ground.
fn ruled_band(wb: f64, n: usize, phase: f64) -> Vec<Triangle> {
    let step = TAU / n as f64;
    let a = |th: f64| gp(R_ROOT, th, law(th));
    let b = |th: f64| gp(R_ROOT, th + wb, law(th + wb) + W_V);
    let mut t = Vec::new();
    for j in 0..n {
        let (th, thn) = (phase + step * j as f64, phase + step * (j + 1) as f64);
        t.push(Triangle::new(a(th), b(th), b(thn)));
        t.push(Triangle::new(a(th), b(thn), a(thn)));
    }
    t
}

/// The ruled band's two edge chains.
fn band_pair(wb: f64, n: usize, phase: f64) -> Vec<Vec<Point3>> {
    let step = TAU / n as f64;
    let mut a = Vec::with_capacity(n + 1);
    let mut b = Vec::with_capacity(n + 1);
    for j in 0..=n {
        let th = phase + step * j as f64;
        a.push(gp(R_ROOT, th, law(th)));
        b.push(gp(R_ROOT, th + wb, law(th + wb) + W_V));
    }
    vec![a, b]
}

fn ball(d: f64) -> Tool {
    Tool {
        kind: ToolKind::BallNose,
        diameter: d,
        corner_r: d * 0.5,
        tip_angle: 0.0,
        pitch: 0.0,
        pitch_min: 0.0,
        thread_hand: Default::default(),
        nose_radius: 0.0,
        width: 0.0,
        flute_len: 30.0,
        shank_d: d,
        shank_len: 30.0,
        holder_d: 20.0,
        holder_len: 20.0,
        material_group: "aluminum".into(),
        center_cutting: true,
        rpm: 10_000.0,
        feed: 400.0,
        tool_material: None,
        flutes: None,
        orientation: None,
        catalog: camcraft_tools::catalog_fields::ToolCatalog::default(),
        cutting: Vec::new(),
        outline: Vec::new(),
        turning: None,
    }
}

fn params<'a>(tool: &'a Tool, stepover: Option<f64>) -> RotaryFlowParams<'a> {
    RotaryFlowParams {
        tool,
        stepover,
        tolerance: 0.05,
        clearance: 14.0,
        retract: None,
        feed: 400.0,
        plunge_feed: 100.0,
        climb: true,
    }
}

/// The ball center a cut move's tip and pose state (the pivot law:
/// the tip is the center backed off `r` along its own pose).
fn center_of(m: &Move, r: f64) -> Option<Point3> {
    let a = m.axis?;
    Some(p3(m.to.x + a[0] * r, m.to.y + a[1] * r, m.to.z + a[2] * r))
}

/// The (azimuth, radius) of a world point about the stated axis.
fn cyl(q: Point3) -> (f64, f64) {
    (q.y.atan2(q.x), (q.x * q.x + q.y * q.y).sqrt())
}

/// The smallest distance from `q` to the mesh - the sim's own exact
/// point-to-triangle law, independent of the kernel's oracle.
fn mesh_dist(q: Point3, facets: &[Triangle]) -> f64 {
    facets
        .iter()
        .map(|f| dist_seg_tri(q, q, f))
        .fold(f64::INFINITY, f64::min)
}

/// The pin: every cut station's ball keeps its radius off the part,
/// and the path is chained and finite.
fn no_gouge(path: &Toolpath, facets: &[Triangle], r: f64) {
    assert!(path.is_chained());
    assert!(path.all_finite());
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        let Some(c) = center_of(m, r) else {
            panic!("a cut move states no pose: {m:?}");
        };
        assert!(
            mesh_dist(c, facets) >= r - 1e-6,
            "a station's ball at {c:?} stands {:e} inside the part",
            r - mesh_dist(c, facets)
        );
    }
}

/// The first pass's ball centers: the plunge's landing station and
/// every feed station after it, up to the retract onto the `clear`
/// cylinder.
fn first_pass_centers(path: &Toolpath, clear: f64, r: f64) -> Vec<Point3> {
    let mut out = Vec::new();
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        let Some(c) = center_of(m, r) else { continue };
        if cyl(c).1 > clear - 2.0 {
            break; // the retract onto the link cylinder
        }
        out.push(c);
    }
    out
}

#[test]
fn a_rotor_floor_band_flows_multi_turn_without_gouge() {
    let facets = groove(N_CIRC, TURNS, 0.0);
    let chains = root_pair(N_CIRC, TURNS, 0.0);
    let tool = ball(2.0);
    // 0.9 mm keeps the count clear of 1-ulp ceiling dust at a whole
    // number of spans (4.0 / 0.9 = 4.44 spans: 6 passes).
    let path = rotary_flow(
        &chains,
        &facets,
        &stated_axis(),
        &params(&tool, Some(0.9)),
        &AxisControl::default(),
    )
    .unwrap();
    no_gouge(&path, &facets, 1.0);
    // The developed band is the groove's 4.0 mm axial width: 6
    // passes at 0.9 mm.
    let plunges = path
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Plunge)
        .count();
    assert_eq!(plunges, 6, "the band did not divide into 6 passes: {:?}", path.notes);
    assert!(
        path.notes.iter().any(|n| n.contains("6 flow passes")),
        "the notes do not state the band: {:?}",
        path.notes
    );
    // The passes run MULTI-TURN: the first pass's centers wind the
    // groove's whole three turns about the stated axis.
    let centers = first_pass_centers(&path, 14.0, 1.0);
    assert!(
        centers.len() > 10,
        "the first pass states {} stations",
        centers.len()
    );
    let mut prev: Option<f64> = None;
    let mut span = 0.0f64;
    for c in &centers {
        let th = cyl(*c).0;
        if let Some(p) = prev {
            let d = th - p;
            span += if d.abs() > std::f64::consts::PI {
                d - d.signum() * TAU
            } else {
                d
            };
        }
        prev = Some(th);
    }
    assert!(
        (span - TAU * TURNS as f64).abs() < 0.3,
        "the first pass winds {span:.3} rad, not the groove's {:.3}",
        TAU * TURNS as f64
    );
    // The first pass is the WALL-ADJACENT one: its picked edge lies
    // IN a wall plane, so the radial push-out carries it up the
    // steep flank (the issue's own law). An interior pass rides the
    // floor a ball radius off it, untouched by any wall.
    let (_, rho0) = cyl(centers[0]);
    assert!(
        rho0 > R_ROOT + 0.5,
        "the wall-adjacent pass rides at {rho0:.3}, not pushed out"
    );
    let floors = path
        .moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid)
        .filter_map(|m| center_of(m, 1.0))
        .filter(|c| {
            let (_, rho) = cyl(*c);
            rho > R_ROOT + 0.5 && rho < R_ROOT + 1.2
        })
        .count();
    assert!(floors > 10, "no pass rides the floor: {floors} stations");
}

#[test]
fn a_pair_straddling_the_branch_aligns_by_a_whole_turn() {
    // The oblique band, phased so the second chain's raw azimuth
    // straddles the kernel basis' own branch: without the
    // whole-turn alignment the pair would read its width plus a
    // full lap. The pass count is the unshifted band's own.
    let (wb, n) = (0.4f64, 72usize);
    let width = (R_ROOT * wb).hypot(PITCH * wb / TAU + W_V);
    let step = 1.0;
    let want = (width / step).ceil() as usize + 1;
    let straight = {
        let facets = ruled_band(wb, n, 0.0);
        let chains = band_pair(wb, n, 0.0);
        let tool = ball(2.0);
        rotary_flow(&chains, &facets, &stated_axis(), &params(&tool, Some(step)), &AxisControl::default())
            .unwrap()
    };
    let straddle = {
        let phase = 4.5; // B's raw azimuth crosses the kernel basis' own branch
        let facets = ruled_band(wb, n, phase);
        let chains = band_pair(wb, n, phase);
        let tool = ball(2.0);
        let path = rotary_flow(&chains, &facets, &stated_axis(), &params(&tool, Some(step)), &AxisControl::default())
            .unwrap();
        no_gouge(&path, &facets, 1.0);
        path
    };
    let count = |p: &Toolpath| p.moves.iter().filter(|m| m.kind == MoveKind::Plunge).count();
    assert_eq!(count(&straight), want, "the straight band divided wrong");
    assert_eq!(
        count(&straddle),
        want,
        "the straddling pair did not align: {} passes, not {want}",
        count(&straddle)
    );
    assert!(
        straddle.notes.iter().any(|n| n.contains("of whole-turn alignment")),
        "the notes do not state the alignment: {:?}",
        straddle.notes
    );
}

#[test]
fn climb_reverses_each_pass() {
    // The same band walked backwards: the same stations, the same
    // clearance, the travel reversed - a reversed pass opens at the
    // forward pass's end.
    let facets = groove(N_CIRC, TURNS, 0.0);
    let chains = root_pair(N_CIRC, TURNS, 0.0);
    let tool = ball(2.0);
    let mut q = params(&tool, Some(1.0));
    q.climb = false;
    let back = rotary_flow(&chains, &facets, &stated_axis(), &q, &AxisControl::default()).unwrap();
    let fwd = rotary_flow(
        &chains,
        &facets,
        &stated_axis(),
        &params(&tool, Some(1.0)),
        &AxisControl::default(),
    )
    .unwrap();
    no_gouge(&back, &facets, 1.0);
    let f = first_pass_centers(&fwd, 14.0, 1.0);
    let b = first_pass_centers(&back, 14.0, 1.0);
    let (Some(x), Some(y)) = (b.first().copied(), f.last().copied()) else {
        panic!("a pass stated no station");
    };
    assert!(
        (x.x - y.x).abs() < 0.2 && (x.y - y.y).abs() < 0.2,
        "the reversed pass opens at {x:?}, not the forward pass's end {y:?}"
    );
}

/// The full-envelope scan: a finer, longer rotor at the scallop
/// law's own stepover, every station judged against every facet.
#[test]
#[ignore = "heavy lane: release --ignored (the 4-turn fine rotor envelope scanned station by station, minutes in debug)"]
fn the_rotor_envelope_stays_gouge_free_at_the_scallop_law() {
    let (n_circ, turns) = (120usize, 4usize);
    let facets = groove(n_circ, turns, 0.0);
    let chains = root_pair(n_circ, turns, 0.0);
    let tool = ball(2.0);
    let path = rotary_flow(
        &chains,
        &facets,
        &stated_axis(),
        &params(&tool, None),
        &AxisControl::default(),
    )
    .unwrap();
    no_gouge(&path, &facets, 1.0);
    // The scallop law's own count: sqrt(8 * r * tol) = 0.63 mm over
    // the 4.0 mm band, ceilinged to whole spans.
    let plunges = path
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Plunge)
        .count();
    let step = (8.0 * 1.0 * 0.05f64).sqrt();
    let want = (W_V / step).ceil() as usize + 1;
    assert_eq!(plunges, want, "the scallop law did not divide the band");
}
