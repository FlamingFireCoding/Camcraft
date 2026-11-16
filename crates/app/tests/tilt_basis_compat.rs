//! Saved tilt basis compatibility (#2133 / PR #2157).
//! Provenance: our own project from 06dfbcaafab3e6bc98fd2cbed85c9ee884bbfd2c,
//! with the display-only model removed. Operation definitions and caches are unchanged.
//!
//! Gate map (M4.4): covers app::project, app::operation, app::posting, cl::frame.

use camcraft_app::commands::{GenerateOperation, History};
use camcraft_app::geometry::Point3;
use camcraft_app::operation::{self, OpStatus};
use camcraft_app::{model, posting, project};
use flecs_ecs::prelude::World;
use serde_json::Value;

const OLD: &str = include_str!("fixtures/legacy-tilt-basis.camproj");
const NAME: &str = "Flat A180";

fn opened(text: &str) -> World {
    let world = camcraft_app::world::new_world();
    project::open(&world, text).expect("load saved project");
    world
}

fn post(world: &World, suffix: &str) -> String {
    let out = std::env::temp_dir().join(format!("legacy-tilt-{}-{suffix}.nc", std::process::id()));
    let nc = posting::post_setup(world, &out)
        .expect("safe saved cut posts")
        .program;
    let _ = std::fs::remove_file(out);
    nc
}

#[test]
fn old_project_keeps_basis_status_regeneration_and_posted_cut() {
    let world = opened(OLD);
    let entity = model::find_by_name(&world, NAME).expect("saved operation");
    let op = model::operation_of(&world, &entity).expect("definition");
    let tilt = op.params.tilt().expect("saved tilt");
    assert_eq!(tilt.basis_version, 0);
    let world_point = tilt
        .frame()
        .expect("legacy frame")
        .to_world(Point3::new(0.0, -19.05, 0.0));
    assert!(
        (world_point.x + 19.05).abs() < 1e-9 && world_point.y.abs() < 1e-9,
        "{world_point:?}"
    );
    assert_eq!(
        operation::status_of(&world, &entity),
        OpStatus::Ok,
        "the unchanged legacy frame keeps its compatible cache"
    );
    let cached = model::toolpath_of(&world, &entity)
        .expect("saved cache")
        .path;
    let before = post(&world, "before");
    let mut history = History::default();
    history.execute(
        &world,
        GenerateOperation::new(&world, NAME).expect("regenerate old cut"),
    );
    assert_eq!(operation::status_of(&world, &entity), OpStatus::Ok);
    assert_eq!(
        post(&world, "after"),
        before,
        "regeneration must preserve the intended machine program"
    );
    let saved = project::save_text(&world).expect("save explicit basis");
    let reopened = opened(&saved);
    let reopened_entity = model::find_by_name(&reopened, NAME).expect("reopened operation");
    assert_eq!(
        model::operation_of(&reopened, &reopened_entity)
            .unwrap()
            .params
            .tilt()
            .unwrap()
            .basis_version,
        0
    );
    assert_eq!(
        operation::status_of(&reopened, &reopened_entity),
        OpStatus::Ok
    );
    // The file quantizes CL coordinates to its documented lattice.
    assert_eq!(
        model::toolpath_of(&reopened, &reopened_entity)
            .unwrap()
            .path,
        cached
    );
    assert_eq!(post(&reopened, "reopened"), before);
}

#[test]
fn corrected_basis_rejects_a_legacy_cache_stamp() {
    let mut file: Value = serde_json::from_str(OLD).unwrap();
    let entity = file["entities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["name"] == NAME)
        .unwrap();
    // Keep the exact old stamp and cached path. Change only the basis contract.
    entity["operation"]["params"]["slot"]["tilt"]["basis_version"] = 1.into();
    let world = opened(&file.to_string());
    let entity = model::find_by_name(&world, NAME).unwrap();
    assert_eq!(operation::status_of(&world, &entity), OpStatus::Stale);
    assert!(
        posting::verification_gate(&world)
            .iter()
            .any(|w| w.to_string().contains(NAME)),
        "post must name the incompatible cache"
    );
    let out = std::env::temp_dir().join(format!("legacy-tilt-{}-stale.nc", std::process::id()));
    assert!(
        posting::post_setup(&world, &out).is_err(),
        "incompatible cache must not post"
    );
    let _ = std::fs::remove_file(out);
}

#[test]
fn live_json_tilts_use_current_basis_and_preserve_an_explicit_legacy_basis() {
    let file: Value = serde_json::from_str(OLD).unwrap();
    let op = file["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == NAME)
        .unwrap();
    let fresh = project::params_from_json(&op["operation"]["params"])
        .expect("new tilt through live JSON door");
    assert_eq!(fresh.tilt().unwrap().basis_version, 1);
    let world = opened(OLD);
    let entity = model::find_by_name(&world, NAME).unwrap();
    let legacy = model::operation_of(&world, &entity).unwrap().params;
    let mut patch = project::params_to_json(&legacy);
    patch["slot"]["feed"] = 300.0.into();
    let edited = project::params_from_json(&patch).expect("partial edit through live JSON door");
    assert_eq!(edited.tilt().unwrap().basis_version, 0);
}

#[test]
fn unknown_saved_basis_is_visibly_refused_before_loading() {
    let mut file: Value = serde_json::from_str(OLD).unwrap();
    let entity = file["entities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["name"] == NAME)
        .unwrap();
    entity["operation"]["params"]["slot"]["tilt"]["basis_version"] = 2.into();
    let world = camcraft_app::world::new_world();
    let error = project::open(&world, &file.to_string()).expect_err("unknown basis must not load");
    let message = error.to_string();
    assert!(
        message.contains(NAME) && message.contains("unsupported frame basis version"),
        "{message}"
    );
    assert!(
        model::find_by_name(&world, NAME).is_none(),
        "refuse before any operation is installed"
    );
}

#[test]
fn saved_basis_contract_requires_a_version_that_legacy_readers_refuse() {
    let old_file: Value = serde_json::from_str(OLD).unwrap();
    assert_eq!(old_file["version"], 62);
    let world = opened(OLD);
    let saved: Value = serde_json::from_str(&project::save_text(&world).unwrap()).unwrap();
    assert!(
        saved["version"].as_u64().unwrap() >= 63,
        "a version-62 reader ignores the new basis field and must refuse this file"
    );
    // The version guard also refuses future basis contracts in this reader.
    let mut future = saved;
    future["version"] = (project::PROJECT_VERSION + 1).into();
    let error = project::open(&camcraft_app::world::new_world(), &future.to_string()).unwrap_err();
    assert!(matches!(
        error,
        project::ProjectError::UnsupportedVersion(_)
    ));
}
