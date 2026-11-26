//! DeleteTool: delete a tool entity with every record it carries, and
//! restore it identically on undo. Our own command model; the snapshot-
//! and-restore inverse every sibling uses.

use flecs_ecs::prelude::World;

use crate::commands::{Command, CommandError};
use crate::model;
use crate::schema::{Location, ToolBody, ToolDef};
use crate::tools::Tool;

/// Delete a tool entity; the snapshot restores it identically, undoable
/// like every other persistent object.
pub struct DeleteTool {
    name: String,
    tool: Tool,
    /// #221: the tool's number, restored with it on undo.
    number: Option<u32>,
    /// #368: its modelled body, restored with it on undo.
    body: Option<ToolBody>,
    /// #93: its drawn cutter, restored with it on undo.
    cutter: Option<camcraft_tools::spun::SpunProfile>,
    /// #3644: its component stack, restored with it on undo.
    assembly: Option<crate::schema::ToolAssembly>,
    location: Location,
}

impl DeleteTool {
    pub fn new(world: &World, name: &str) -> Result<Self, CommandError> {
        let Some(entity) = model::find_by_name(world, name) else {
            return Err(CommandError::NotFound(name.to_string()));
        };
        let tool = model::tool_of(world, &entity)
            .ok_or_else(|| CommandError::Invalid(format!("'{name}' is not a tool")))?;
        Ok(DeleteTool {
            name: name.to_string(),
            tool,
            number: model::tool_number_of(world, &entity),
            body: crate::tool_body::body_of(world, &entity),
            cutter: crate::tool_cutter::cutter_of(world, &entity),
            assembly: crate::tool_assembly::assembly_of(world, &entity),
            location: model::transform_of(world, &entity).unwrap_or_default(),
        })
    }
}

impl Command for DeleteTool {
    fn apply(&self, world: &World) {
        if let Some(entity) = model::find_by_name(world, &self.name) {
            model::remove(world, &entity);
        }
    }

    fn invert(&self, world: &World) {
        if model::find_by_name(world, &self.name).is_some() {
            return; // already restored (idempotent redo path)
        }
        let entity = model::spawn(world, &self.name);
        model::set_transform(world, &entity, self.location);
        entity
            .entity_view(world)
            .set(ToolDef { tool: self.tool.clone(), number: self.number });
        crate::tool_body::set_body(world, &entity, self.body.as_ref());
        crate::tool_cutter::set_cutter(world, &entity, self.cutter.as_ref());
        crate::tool_assembly::set_assembly(world, &entity, self.assembly.as_ref());
    }
}
