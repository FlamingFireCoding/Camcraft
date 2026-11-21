//! Operator-requested native inspection export over the same selected-operation reader as the GUI.

use super::setup_tools::obj;
use super::tools::{Tool, req_str};
use crate::script::Env;
use serde_json::{Map, Value, json};
use std::path::Path;

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "export_agie_inspection",
        description: "Export one selected DXF Wire Contour die opening for Classic 2S inspection. The setup uses 10 mm steel, 0.25 mm wire and a 3 mm start hole. Physical stock XY is unknown. Native length and width are zero. Writes a new ISO/SBR/SBL directory with zero working steps.",
        schema: || {
            obj(
                json!({
                    "operation":{"type":"string","description":"Wire Contour operation name"},
                    "directory":{"type":"string","description":"New directory whose final component is a 1–8 character uppercase DOS stem"}
                }),
                &["operation", "directory"],
            )
        },
        run: export,
    },
    Tool {
        name: "create_agie_inspection",
        description: "Create one nominal inspection opening on the active Classic 2S setup with a selected DXF chain and 3 mm start hole. No machining condition or capture is assigned. One undo step.",
        schema: || {
            obj(
                json!({
                    "name":{"type":"string"}, "tool":{"type":"string"}, "drawing":{"type":"string"},
                    "chain":{"type":"string","description":"Stable 32-digit DXF chain id from list_features"},
                    "start":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2,"description":"Centre of the 3 mm hole in world XY millimetres"}
                }),
                &["name", "tool", "drawing", "chain", "start"],
            )
        },
        run: create,
    },
];

fn create(env: &mut Env, args: &Map<String, Value>) -> Result<Value, String> {
    let name = req_str(args, "name")?;
    let id = crate::geometry::sketch_chains::parse_chain_id_hex(req_str(args, "chain")?)
        .ok_or("use a 32-digit lowercase hexadecimal chain id")?;
    let start = super::setup_tools::opt_nums::<2>(args, "start")?
        .ok_or("state the 3 mm hole centre in world XY")?;
    let cmd = crate::commands::CreateOperation::agie_inspection(
        &env.world,
        name,
        req_str(args, "tool")?,
        req_str(args, "drawing")?,
        id,
        start,
    )
    .map_err(|e| e.to_string())?;
    env.history.execute(&env.world, cmd);
    Ok(
        json!({"operation":name,"chain":id.to_string(),"purpose":"nominal inspection","working_steps":0}),
    )
}

fn export(env: &mut Env, args: &Map<String, Value>) -> Result<Value, String> {
    let receipt = crate::operation::export_agie_inspection(
        &env.world,
        req_str(args, "operation")?,
        Path::new(req_str(args, "directory")?),
    )?;
    serde_json::to_value(receipt).map_err(|e| e.to_string())
}
