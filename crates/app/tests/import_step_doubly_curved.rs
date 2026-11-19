//! #1602 (PR #1638 review item 5): a face curved in both directions is not
//! under-divided by the per-direction split. A saddle patch is imported
//! through `step::parse_with_chord`, and every vertex, edge midpoint and
//! facet centre of its mesh is measured against the ideal surface. #917:
//! the patch is the top of a closed block (solidcore reads solids only).
//!
//! Fixture: generated here, our own geometry - the saddle z = a (x^2 - y^2)
//! over a square, written exactly as one biquadratic Bezier patch (the
//! Bernstein form of a quadratic in a linear parameter, Piegl & Tiller,
//! "The NURBS Book", 2nd ed., ch. 1-2), bounded by its four quadratic
//! boundary curves, as ISO 10303-21 entities. No user file.
//!
//! Gate map (M4.4): covers app::geometry::import::step,
//! camcraft_geometry::step.

use std::fmt::Write as _;

use camcraft_app::geometry::import::step;
use camcraft_app::geometry::Point3;

/// Half the side of the square, in mm.
const HALF: f64 = 10.0;
/// z = A (x^2 - y^2): principal radii of 1 / (2 A) = 10 mm at the centre.
const A: f64 = 0.05;
/// The block's floor, under the saddle's lowest point (-A HALF^2 = -5 mm).
const FLOOR: f64 = -10.0;

fn real(x: f64) -> String {
    format!("{x:.17E}")
}

/// The saddle's height over (x, y).
fn ideal_z(x: f64, y: f64) -> f64 {
    A * (x * x - y * y)
}

/// The patch's control net: x and y linear in u and v, and the Bernstein
/// coefficients of x^2 over [-HALF, HALF] are (HALF^2, -HALF^2, HALF^2).
fn net() -> [[[f64; 3]; 3]; 3] {
    let at = [-HALF, 0.0, HALF];
    let sq = [HALF * HALF, -HALF * HALF, HALF * HALF];
    std::array::from_fn(|i| std::array::from_fn(|j| [at[i], at[j], A * (sq[i] - sq[j])]))
}

/// The saddle as a STEP file in millimetres: the top face of a block
/// whose floor is at [`FLOOR`].
fn saddle_step() -> String {
    let n = net();
    let mut out = String::new();
    let mut id = 100;
    let mut e = |out: &mut String, body: String| -> String {
        id += 1;
        let _ = writeln!(out, "#{id} = {body};");
        format!("#{id}")
    };
    let mut p: Vec<Vec<String>> = Vec::new();
    for row in &n {
        let ids = row.iter().map(|&[x, y, z]| e(&mut out, format!("CARTESIAN_POINT('', ({}, {}, {}))", real(x), real(y), real(z))));
        p.push(ids.collect());
    }
    let rows: Vec<String> = (0..3).map(|i| format!("({}, {}, {})", p[i][0], p[i][1], p[i][2])).collect();
    let surface = e(
        &mut out,
        format!(
            "B_SPLINE_SURFACE_WITH_KNOTS('', 2, 2, ({}), .UNSPECIFIED., .F., .F., .F., (3, 3), (3, 3), (0.0, 1.0), (0.0, 1.0), .UNSPECIFIED.)",
            rows.join(", ")
        ),
    );
    let mut curve = |out: &mut String, a: &str, b: &str, c: &str| {
        e(out, format!("B_SPLINE_CURVE_WITH_KNOTS('', 2, ({a}, {b}, {c}), .UNSPECIFIED., .F., .F., (3, 3), (0.0, 1.0), .UNSPECIFIED.)"))
    };
    let v0 = curve(&mut out, &p[0][0], &p[1][0], &p[2][0]);
    let u1 = curve(&mut out, &p[2][0], &p[2][1], &p[2][2]);
    let v1 = curve(&mut out, &p[0][2], &p[1][2], &p[2][2]);
    let u0 = curve(&mut out, &p[0][0], &p[0][1], &p[0][2]);
    let mut e = |body: String| e(&mut out, body);
    let vx: Vec<String> = [&p[0][0], &p[2][0], &p[2][2], &p[0][2]].iter().map(|c| e(format!("VERTEX_POINT('', {c})"))).collect();
    // The floor under the saddle (#917: solidcore reads closed solids only,
    // so the patch is the top of a block): corners B00, B20, B22, B02.
    let floor = [(-HALF, -HALF), (HALF, -HALF), (HALF, HALF), (-HALF, HALF)];
    let bx: Vec<String> = floor
        .iter()
        .map(|&(x, y)| {
            let pt = e(format!("CARTESIAN_POINT('', ({}, {}, {}))", real(x), real(y), real(FLOOR)));
            e(format!("VERTEX_POINT('', {pt})"))
        })
        .collect();
    let mut dir = |d: [f64; 3]| e(format!("DIRECTION('', ({}, {}, {}))", real(d[0]), real(d[1]), real(d[2])));
    let (up, xd, yd) = (dir([0.0, 0.0, 1.0]), dir([1.0, 0.0, 0.0]), dir([0.0, 1.0, 0.0]));
    let normals = [dir([0.0, -1.0, 0.0]), dir([1.0, 0.0, 0.0]), dir([0.0, 1.0, 0.0]), dir([-1.0, 0.0, 0.0]), dir([0.0, 0.0, -1.0])];
    let mut line = |from: (f64, f64, f64), along: &str, length: f64| {
        let at = e(format!("CARTESIAN_POINT('', ({}, {}, {}))", real(from.0), real(from.1), real(from.2)));
        let v = e(format!("VECTOR('', {along}, {})", real(length)));
        e(format!("LINE('', {at}, {v})"))
    };
    let rise = -FLOOR;
    let lines = [
        line((-HALF, -HALF, FLOOR), &up, rise),
        line((HALF, -HALF, FLOOR), &up, rise),
        line((HALF, HALF, FLOOR), &up, rise),
        line((-HALF, HALF, FLOOR), &up, rise),
        line((-HALF, -HALF, FLOOR), &xd, 2.0 * HALF),
        line((HALF, -HALF, FLOOR), &yd, 2.0 * HALF),
        line((-HALF, HALF, FLOOR), &xd, 2.0 * HALF),
        line((-HALF, -HALF, FLOOR), &yd, 2.0 * HALF),
    ];
    let mut ec = |a: &str, b: &str, c: &str| e(format!("EDGE_CURVE('', {a}, {b}, {c}, .T.)"));
    let (v0, u1, v1, u0) = (ec(&vx[0], &vx[1], &v0), ec(&vx[1], &vx[2], &u1), ec(&vx[3], &vx[2], &v1), ec(&vx[0], &vx[3], &u0));
    let rises: Vec<String> = (0..4).map(|k| ec(&bx[k], &vx[k], &lines[k])).collect();
    let (b0, b1, b2, b3) = (ec(&bx[0], &bx[1], &lines[4]), ec(&bx[1], &bx[2], &lines[5]), ec(&bx[3], &bx[2], &lines[6]), ec(&bx[0], &bx[3], &lines[7]));
    let (t, f) = (true, false);
    // Each loop runs counter-clockwise seen from outside the block.
    let loops: [Vec<(&String, bool)>; 6] = [
        vec![(&v0, t), (&u1, t), (&v1, f), (&u0, f)],
        vec![(&b0, t), (&rises[1], t), (&v0, f), (&rises[0], f)],
        vec![(&b1, t), (&rises[2], t), (&u1, f), (&rises[1], f)],
        vec![(&b2, f), (&rises[3], t), (&v1, t), (&rises[2], f)],
        vec![(&b3, f), (&rises[0], t), (&u0, t), (&rises[3], f)],
        vec![(&b3, t), (&b2, t), (&b1, f), (&b0, f)],
    ];
    let corners = [(-HALF, -HALF, FLOOR), (HALF, -HALF, FLOOR), (HALF, HALF, FLOOR), (-HALF, -HALF, FLOOR), (-HALF, -HALF, FLOOR)];
    let mut faces = Vec::new();
    for (k, lp) in loops.iter().enumerate() {
        let oriented: Vec<String> =
            lp.iter().map(|(edge, sense)| e(format!("ORIENTED_EDGE('', *, *, {edge}, {})", if *sense { ".T." } else { ".F." }))).collect();
        let lp = e(format!("EDGE_LOOP('', ({}))", oriented.join(", ")));
        let bound = e(format!("FACE_OUTER_BOUND('', {lp}, .T.)"));
        let face_surface = if k == 0 {
            surface.clone()
        } else {
            let (x, y, z) = corners[k - 1];
            let at = e(format!("CARTESIAN_POINT('', ({}, {}, {}))", real(x), real(y), real(z)));
            let refd = if k == 5 { &xd } else { &up };
            let place = e(format!("AXIS2_PLACEMENT_3D('', {at}, {}, {refd})", normals[k - 1]));
            e(format!("PLANE('', {place})"))
        };
        faces.push(e(format!("ADVANCED_FACE('', ({bound}), {face_surface}, .T.)")));
    }
    let shell = e(format!("CLOSED_SHELL('', ({}))", faces.join(", ")));
    let solid = e(format!("MANIFOLD_SOLID_BREP('saddle block', {shell})"));
    e(format!("ADVANCED_BREP_SHAPE_REPRESENTATION('', ({solid}), #11)"));
    format!(
        "ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION(('CamCraft #1602 fixture: a saddle'), '2;1');\n\
         FILE_NAME('', '', (''), (''), '', '', '');\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\nDATA;\n\
         #11 = ( GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#15)) \
         GLOBAL_UNIT_ASSIGNED_CONTEXT((#12, #13, #14)) REPRESENTATION_CONTEXT('', '') );\n\
         #12 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );\n\
         #13 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );\n\
         #14 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );\n\
         #15 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #12, 'distance_accuracy_value', '');\n\
         {out}ENDSEC;\nEND-ISO-10303-21;\n"
    )
}

/// How far `p` stands off the saddle along z. The surface's normal is never
/// steeper than the vertical gap reads, so this is an upper bound on the
/// true distance.
fn gap(p: Point3) -> f64 {
    (p.z - ideal_z(p.x, p.y)).abs()
}

fn mid(a: Point3, b: Point3) -> Point3 {
    Point3::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.z + b.z) / 2.0)
}

#[test]
fn a_saddle_patch_meshes_within_the_chord_promise_everywhere() {
    let (mesh, chord) = step::parse_with_chord(saddle_step().as_bytes()).expect("the saddle block imports");
    // The block holds (z - FLOOR) over the square: the saddle term
    // integrates to zero, so its volume is -FLOOR (2 HALF)^2.
    let volume: f64 = mesh.facets.iter().map(|t| t.a.dot(t.b.cross(t.c)) / 6.0).sum();
    let want = -FLOOR * 4.0 * HALF * HALF;
    assert!((volume - want).abs() < 1e-3 * want, "block volume {volume}, want {want}");
    // The saddle's own facets: its normal is never flatter than 1/sqrt(3)
    // up; the walls are vertical and the floor faces down.
    let facets: Vec<_> = mesh.facets.iter().filter(|t| t.normal().z > 0.3).copied().collect();
    let facets = &facets;
    assert!(facets.len() > 100, "a doubly curved patch is divided both ways: {} facets", facets.len());
    assert!(step::coarse(chord).is_none(), "meshed at the promised chord, not reduced: {chord} mm");
    // Vertices lie on the surface (they are surface evaluations), and the
    // patch is covered: the mesh spans the whole square.
    let on = facets.iter().flat_map(|t| [t.a, t.b, t.c]).map(gap).fold(0.0, f64::max);
    assert!(on < 1e-6, "a vertex stands {on} mm off the saddle");
    let span = |f: fn(&Point3) -> f64| {
        let all = facets.iter().flat_map(|t| [t.a, t.b, t.c]);
        let (lo, hi) = all.fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(f(&p)), hi.max(f(&p))));
        hi - lo
    };
    assert!((span(|p| p.x) - 2.0 * HALF).abs() < 1e-6 && (span(|p| p.y) - 2.0 * HALF).abs() < 1e-6, "the mesh covers the patch");
    // Between the vertices: every edge midpoint and facet centre within the
    // chord promise. An under-divided direction shows here first.
    let worst = facets
        .iter()
        .flat_map(|t| {
            let c = Point3::new((t.a.x + t.b.x + t.c.x) / 3.0, (t.a.y + t.b.y + t.c.y) / 3.0, (t.a.z + t.b.z + t.c.z) / 3.0);
            [mid(t.a, t.b), mid(t.b, t.c), mid(t.c, t.a), c]
        })
        .map(gap)
        .fold(0.0, f64::max);
    assert!(worst <= chord, "the mesh stands {worst} mm off the saddle, over the {chord} mm chord promise");
}
