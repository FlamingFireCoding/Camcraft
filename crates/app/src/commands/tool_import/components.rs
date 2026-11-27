//! #3645: the tool components that an import lands with its tools, in
//! the same undo step. Our own command model.
//!
//! The id law: a library row whose id the document already holds is
//! reused when its content (name, item and geometry) is the same, and
//! refuses the whole import by the id when it differs. An import never
//! changes or renames a document row: other tools may be built from it,
//! and a silent rename would hide which row a stack names.

use flecs_ecs::prelude::World;

use crate::commands::CommandError;
use crate::model;
use crate::schema::{DocumentComponent, ToolAssembly};
use crate::tool_assembly as ta;
use crate::tool_library::LibraryAssembly;

/// The table before and after an import that adds rows.
pub(super) type TableChange = (Vec<DocumentComponent>, Vec<DocumentComponent>);

fn same_content(a: &DocumentComponent, b: &DocumentComponent) -> bool {
    a.component == b.component && a.geometry == b.geometry
}

/// What [`plan`] gives: the table change (`None`: every row is in the
/// document already), and the ids added and reused (sorted).
pub(super) type TablePlan = (Option<TableChange>, Vec<String>, Vec<String>);

/// The table plan for the stacks that land.
pub(super) fn plan(world: &World, landing: &[&LibraryAssembly]) -> Result<TablePlan, CommandError> {
    let prev = ta::components(world);
    let mut next = prev.clone();
    let mut added: Vec<String> = Vec::new();
    let mut reused: Vec<String> = Vec::new();
    // The file's table order, so an export after the import is the same.
    let mut rows: Vec<&(usize, DocumentComponent)> = landing.iter().flat_map(|a| a.rows.iter()).collect();
    rows.sort_by_key(|(i, _)| *i);
    for (_, row) in rows {
        let id = row.component.id();
        if added.iter().any(|a| a == id) || reused.iter().any(|r| r == id) {
            continue;
        }
        match prev.iter().find(|r| r.component.id() == id) {
            Some(doc) if same_content(doc, row) => reused.push(id.to_owned()),
            Some(_) => {
                return Err(CommandError::Invalid(format!(
                    "tool component '{id}' in the tool library is not the same as the document's tool component '{id}' - rename one of them; nothing imported"
                )));
            }
            None => {
                added.push(id.to_owned());
                next.push(row.clone());
            }
        }
    }
    added.sort();
    reused.sort();
    let change = if added.is_empty() { None } else { Some((prev, next)) };
    Ok((change, added, reused))
}

/// Land (`Some`) or clear (`None`) the stack of tool `name`.
pub(super) fn put_assembly(world: &World, name: &str, stack: Option<&ToolAssembly>) {
    if let Some(entity) = model::find_by_name(world, name) {
        ta::set_assembly(world, &entity, stack);
    }
}
