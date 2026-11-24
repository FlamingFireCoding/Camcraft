//! The modeled workpiece of each operation of a native wire EDM job: one work, or a group of
//! independently captured punches (#2961). Provenance: CAMCraft's Wire Contour workflow.

use flecs_ecs::prelude::World;

use super::AgieJob;
use crate::operation::{self, SrcRef};
use crate::{model, schema::Operation};

/// The nominal jobs of the active operations, in program order.
///
/// One operation needs exactly one modeled workpiece. A group needs one
/// complete modeled punch for each operation, a different model for each,
/// and no other model, so that no unowned part can meet a controller path.
pub(super) fn jobs(
    world: &World,
    operations: &[(&str, Operation)],
) -> Result<Vec<AgieJob>, String> {
    let models = model::model_entities(world);
    if let [(name, op)] = operations {
        let [(source, _)] = models.as_slice() else {
            return Err("The native wire EDM job requires exactly one modeled workpiece; additional parts or fixture models can intersect the controller path".into());
        };
        return Ok(vec![operation::agie_job(world, op, name, source)?]);
    }
    let mut owners: Vec<&str> = Vec::with_capacity(operations.len());
    for (_, op) in operations {
        let SrcRef::Entity(owner) = &op.src else {
            return Err("Select one complete modeled punch for each work of a punch group".into());
        };
        if owners.contains(&owner.as_str()) {
            return Err("The works of a punch group must cut different modeled punches".into());
        }
        owners.push(owner);
    }
    if models.len() != owners.len()
        || models
            .iter()
            .any(|(model, _)| !owners.contains(&model.as_str()))
    {
        return Err("Each modeled workpiece of a punch group must be the punch of one work; additional parts or fixture models can intersect the controller path".into());
    }
    operations
        .iter()
        .zip(owners)
        .map(|((name, op), owner)| operation::agie_job(world, op, name, owner))
        .collect()
}
