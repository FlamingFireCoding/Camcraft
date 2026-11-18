//! The faced-block fixture of the WCS flip suites (#1720, #1649).
//! Provenance: our own fixture over our own WCS model.
#![allow(dead_code)]

use flecs_ecs::prelude::World;

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, SetStock, SetWcsFlip};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

pub const TOOL: &str = "EM-0400-2FL";

pub fn face() -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        blend_turns: true,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    }
}

/// A 60 x 60 x 20 box faced at Z 19.5 and generated, no WCS placed.
pub fn faced_block() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let stock = SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default());
    h.execute(&world, stock.expect("box stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("tool");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("add tool"));
    h.execute(&world, CreateOperation::new(&world, "Face1", TOOL, SrcRef::Stock, face()).expect("Face1"));
    h.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate"));
    (world, h)
}

pub fn flip(world: &World, h: &mut History, setup: Option<&str>, f: [bool; 3]) {
    h.execute(world, SetWcsFlip::for_setup(world, setup, f).expect("flip"));
}
