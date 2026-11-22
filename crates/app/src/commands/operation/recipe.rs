//! ApplyCutRecipe (#3073): make a new operation from a saved cut recipe
//! on new geometry, as one undoable step. The recipe's tool must still
//! be in the document and its material must still be in the table and
//! agree with the material in force; else it is refused with the
//! recipe's name. Every creation gate then applies through
//! `CreateOperation`, so a recipe can never make an operation the
//! property panel could not.
//!
//! Provenance: our own command, a thin composite over CreateOperation.

use flecs_ecs::prelude::World;

use crate::commands::{Command, CommandError};
use crate::cut_recipe::CutRecipe;
use crate::model;
use crate::operation::SrcRef;

use super::CreateOperation;

/// One undo step: the operation the recipe made.
pub struct ApplyCutRecipe(CreateOperation);

impl ApplyCutRecipe {
    /// Operation `name` from `recipe` on `src`, in the active setup.
    pub fn new(world: &World, recipe: &CutRecipe, name: &str, src: SrcRef) -> Result<Self, CommandError> {
        let r = &recipe.name;
        if !recipe.tool.is_empty() {
            let found = model::find_by_name(world, &recipe.tool).is_some_and(|e| model::is_tool(world, &e));
            if !found {
                return Err(CommandError::Needs(format!(
                    "cut recipe '{r}': its tool '{}' is not in the document - add it, or save the recipe again with a tool that is",
                    recipe.tool
                )));
            }
        }
        let wanted = recipe.material().map_err(CommandError::Needs)?;
        let setup = crate::setup::active(world);
        if let (Some(wanted), Some(here)) = (wanted, crate::stock_material::in_force(world, setup.as_deref()))
            && wanted != here
        {
            return Err(CommandError::Needs(format!(
                "cut recipe '{r}' is for {} ('{}'), but this setup cuts {} - its feeds and steps do not fit",
                wanted.label(),
                wanted.word(),
                here.label()
            )));
        }
        CreateOperation::new(world, name, &recipe.tool, src, recipe.params.clone()).map(Self)
    }
}

impl Command for ApplyCutRecipe {
    fn apply(&self, world: &World) {
        self.0.apply(world);
    }

    fn invert(&self, world: &World) {
        self.0.invert(world);
    }
}
