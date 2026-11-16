//! Curved fixture facets cannot clear collisions with the intended solid.
//! Gate map (M4.4): covers app::fixture, app::fixture_profile, geometry::profile_mesh.
//! Provenance: analytic circles and the fixture rapid law.

use camcraft_app::cl::{Move, MoveKind, Toolpath};
use camcraft_app::commands::{History, InsertProfileFixture};
use camcraft_app::fixture;
use camcraft_app::fixture_profile::{ExtrudeAxis, FixtureProfile, ProfilePoint, SpunPoint};
use camcraft_app::geometry::Point3;
use camcraft_app::schema::Location;
use camcraft_app::verify::{
    MeshSteel, ToolGeom, Warning, Workholding, mesh_refined, rapids_vs_workholding, steel_stack,
};
use camcraft_app::world::new_world;
use std::f64::consts::PI;

fn assert_curved_fixture_keeps_hit(profile: FixtureProfile, angle: f64, check_y_bound: bool) {
    let world = new_world();
    let mut history = History::default();
    history.execute(
        &world,
        InsertProfileFixture::new(&world, "Curved", profile, None, Location::default()).unwrap(),
    );
    let status = camcraft_app::fixture_view::mesh_states(&world, None);
    assert_eq!(status[0].1, camcraft_app::fixture_view::MeshState::BoxOnly(camcraft_app::fixture_view::BoxReason::ProfileChord));
    assert!(status[0].1.detail().contains("approximates curved surfaces"));
    let boxes = fixture::boxes_for(&world, None);
    if check_y_bound {
        assert!(
            boxes[0].max.y >= 1000.0,
            "box must contain true arc between mesh vertices: {:?}",
            boxes[0]
        );
    }
    let meshes = fixture::meshes_for(&world, None);
    assert!(
        meshes.is_empty(),
        "approximate facets cannot acquit real steel"
    );
    let path = Toolpath {
        moves: vec![Move::new(
            MoveKind::Rapid,
            Point3::new(999.0 * angle.cos(), 999.0 * angle.sin(), 10.0),
            Point3::new(999.0 * angle.cos(), 999.0 * angle.sin(), 20.0),
            None,
        )],
        ..Default::default()
    };
    let tool = ToolGeom {
        radius: 1.0,
        nose_radius: 0.0,
        flute_len: 1.0,
        shank_radius: 1.0,
        shank_len: 1.0,
        holder_radius: 1.0,
        holder_len: 1.0,
    };
    let stack = steel_stack(
        &tool,
        &camcraft_tools::holder_profile::HolderStack::cylinder(&tool),
        None,
    );
    let wh = Workholding {
        turning: &boxes,
        fixed: &[],
        axis: None,
    };
    let steel = MeshSteel {
        fixtures: &meshes,
        stack: &stack,
        chord_tol: 0.01,
    };
    let boxed = rapids_vs_workholding(&path, &tool, &wh, None);
    assert!(
        boxed
            .iter()
            .any(|w| matches!(w, Warning::FixtureHit { .. }))
    );
    assert_eq!(
        mesh_refined(&path, &wh, &steel, true, |w| rapids_vs_workholding(
            &path, &tool, w, None
        )),
        boxed
    );
}

#[test]
fn spun_fixture_facets_cannot_clear_true_cylinder_collision() {
    assert_curved_fixture_keeps_hit(
        FixtureProfile::Spun {
            points: vec![
                SpunPoint::line(0.0, 0.0),
                SpunPoint::line(0.0, 1000.0),
                SpunPoint::line(50.0, 1000.0),
                SpunPoint::line(50.0, 0.0),
            ],
        },
        PI / 32.0,
        false,
    );
}

#[test]
fn curved_extrusion_grows_box_and_cannot_clear_true_arc_collision() {
    // Two semicircles, rotated five degrees. The true Y extreme lies
    // between the ten-degree mesh samples, outside their unpadded AABB.
    let angle = PI / 36.0;
    let (x, y) = (1000.0 * angle.cos(), 1000.0 * angle.sin());
    assert_curved_fixture_keeps_hit(
        FixtureProfile::Extruded {
            points: vec![
                ProfilePoint {
                    u: x,
                    v: y,
                    bulge: 1.0,
                },
                ProfilePoint {
                    u: -x,
                    v: -y,
                    bulge: 1.0,
                },
            ],
            axis: ExtrudeAxis::Z,
            depth: 50.0,
        },
        PI / 2.0,
        true,
    );
}
