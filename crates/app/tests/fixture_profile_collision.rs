//! Curved fixture facets cannot clear collisions with the intended solid.
//! Gate map (M4.4): covers app::fixture, app::fixture_view, app::fixture_profile,
//! app::commands::profile_fixture, app::fixture_library, geometry::profile_mesh.
//! Provenance: analytic circles and the fixture rapid law.

use camcraft_app::cl::{Move, MoveKind, Toolpath};
use camcraft_app::commands::{
    DeriveFixtureProfile, History, ImportFixture, ImportModel, InsertFixture, InsertProfileFixture,
};
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

#[test]
fn step_reference_profile_keeps_collision_results_through_library_and_undo() {
    use camcraft_app::fixture_library::{self, DatumChoice};
    use camcraft_app::{fixture_view, model};

    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/box.step"),
    )
    .unwrap();
    let world = new_world();
    let mut history = History::default();
    let import = ImportModel::from_bytes("box.step", &bytes, None).unwrap();
    let insert = ImportFixture::new(&world, import, None).unwrap();
    let name = insert.name().to_owned();
    history.execute(&world, insert);
    let entity = model::find_by_name(&world, &name).unwrap();
    let facets = model::facets_of(&world, &entity).unwrap();
    let boxes = fixture::boxes_for(&world, None);
    let state = fixture_view::mesh_states(&world, None)[0].1;
    assert_eq!(fixture::meshes_for(&world, None).len(), 1);
    assert_eq!(state, fixture_view::MeshState::Mesh);

    let check = |world: &flecs_ecs::prelude::World, name: &str| {
        let entity = model::find_by_name(world, name).unwrap();
        assert_eq!(
            model::facets_of(world, &entity).unwrap().as_ref(),
            facets.as_ref()
        );
        assert_eq!(
            fixture::boxes_for(world, None),
            boxes,
            "reference profiles must not grow STEP steel"
        );
        assert_eq!(
            fixture::meshes_for(world, None).len(),
            1,
            "STEP mesh stays eligible"
        );
        assert_eq!(fixture_view::mesh_states(world, None)[0].1, state);
        let path = Toolpath {
            moves: vec![Move::new(
                MoveKind::Rapid,
                Point3::new(1.003, 0.5, 0.2),
                Point3::new(1.003, 0.5, 0.8),
                None,
            )],
            ..Default::default()
        };
        let tool = ToolGeom {
            radius: 0.001,
            nose_radius: 0.0,
            flute_len: 0.01,
            shank_radius: 0.001,
            shank_len: 0.01,
            holder_radius: 0.001,
            holder_len: 0.01,
        };
        let stack = steel_stack(
            &tool,
            &camcraft_tools::holder_profile::HolderStack::cylinder(&tool),
            None,
        );
        let meshes = fixture::meshes_for(world, None);
        let current_boxes = fixture::boxes_for(world, None);
        let wh = Workholding {
            turning: &current_boxes,
            fixed: &[],
            axis: None,
        };
        let steel = MeshSteel {
            fixtures: &meshes,
            stack: &stack,
            chord_tol: 0.01,
        };
        assert!(
            mesh_refined(&path, &wh, &steel, true, |w| {
                rapids_vs_workholding(&path, &tool, w, None)
            })
            .is_empty(),
            "derivation must not add a false collision beside unchanged STEP steel"
        );
    };
    check(&world, &name);
    history.execute(&world, DeriveFixtureProfile::spun(&world, &name).unwrap());
    assert!(
        fixture::def_of(&world, &entity)
            .unwrap()
            .profile
            .unwrap()
            .collision_chord_error()
            > 0.0
    );
    check(&world, &name);
    assert!(history.undo(&world));
    check(&world, &name);
    assert!(history.redo(&world));
    check(&world, &name);

    let entry =
        fixture_library::entry_from_model(&world, &name, DatumChoice::ModelOrigin, None).unwrap();
    assert!(entry.step_source.is_some());
    assert!(entry.profile.is_some());
    let text = fixture_library::to_text(&[entry]).unwrap();
    let entries = fixture_library::parse(&text).unwrap();
    let fresh = new_world();
    let mut fresh_history = History::default();
    let insert = InsertFixture::new(&fresh, &entries[0], None, Location::default()).unwrap();
    let inserted_name = insert.name().to_owned();
    fresh_history.execute(&fresh, insert);
    check(&fresh, &inserted_name);
}

#[test]
fn step_reference_profile_preserves_reduced_and_unknown_fidelity_rules() {
    use camcraft_app::{model, model_fidelity};
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/box.step"),
    )
    .unwrap();
    for chord in [0.5, model_fidelity::UNKNOWN_MM] {
        let world = new_world();
        let mut history = History::default();
        let import = ImportModel::from_bytes("box.step", &bytes, None).unwrap();
        let insert = ImportFixture::new(&world, import, None).unwrap();
        let name = insert.name().to_owned();
        history.execute(&world, insert);
        let entity = model::find_by_name(&world, &name).unwrap();
        model_fidelity::set(&world, &entity, Some(chord));
        let boxes = fixture::boxes_for(&world, None);
        assert!(fixture::meshes_for(&world, None).is_empty());
        history.execute(&world, DeriveFixtureProfile::spun(&world, &name).unwrap());
        assert_eq!(fixture::boxes_for(&world, None), boxes);
        assert!(
            fixture::meshes_for(&world, None).is_empty(),
            "reference profiles cannot override STEP fidelity"
        );
        assert_eq!(model_fidelity::chord_of(&world, &entity), Some(chord));
    }
}
