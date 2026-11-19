//! The Deburr operation's picked edge chains (#1888), read from the
//! world: the source's picks are the edges to ride, `exclude_edges` the
//! edges to leave out. Provenance: our own resolution over the feature
//! layer's chain ids, the walk `selection::drive_chains` does for the
//! drive-curve contour, with the deburr's own refusals.
//!
//! One list answers both: the source's chains first (one per pick),
//! then the excluded ones. [`picked`] says where the list splits. An
//! `Entity` or `Stock` source, or an empty selection, has no picked
//! chains: the ride finds the model's exposed edges itself.

use super::*;

/// How many leading chains of [`chains_of`]'s list are the source's.
pub(super) fn picked(src: &SrcRef) -> usize {
    match src {
        SrcRef::Selection(items) => items.len(),
        SrcRef::Stock | SrcRef::Entity(_) => 0,
    }
}

/// The picked chains, then the excluded chains, in the setup's frame.
pub(super) fn chains_of(world: &World, op: &Operation, exclude: &[GeoSel]) -> Result<Vec<Vec<Point3>>, GenError> {
    let mut out = Vec::new();
    if let SrcRef::Selection(items) = &op.src {
        for g in items {
            out.push(chain(world, op.setup.as_deref(), g, "to break")?);
        }
    }
    for g in exclude {
        out.push(chain(world, op.setup.as_deref(), g, "to exclude")?);
    }
    Ok(out)
}

/// One pick as a polyline where it stands; anything but a live edge
/// chain is refused by its label.
fn chain(world: &World, setup: Option<&str>, g: &GeoSel, role: &str) -> Result<Vec<Point3>, GenError> {
    let Some(entity) = model::find_by_name(world, &g.entity) else {
        return Err(GenError::Invalid(format!(
            "the Deburr edges {role} name '{}', which is gone",
            g.entity
        )));
    };
    let Some(features) = model::features_of(world, &entity) else {
        return Err(GenError::Invalid(format!(
            "'{}' has no geometry to select from",
            g.entity
        )));
    };
    let label = crate::pick::geo_label(world, g);
    let FeatureSel::Chain(cid) = g.feature else {
        return Err(GenError::Invalid(format!(
            "{label} is not an edge chain - Deburr takes edge chains as the edges {role}; pick edges, or clear the picks to break every exposed edge"
        )));
    };
    let Some(c) = crate::model::resolve_chain(&features, cid) else {
        return Err(GenError::Invalid(format!(
            "{label} is no longer in the model: re-pick it"
        )));
    };
    let loc = crate::setup_align::pose_in(world, &entity, setup).unwrap_or_default();
    let pts: Vec<Point3> = feature::chain_points(&features, &c)
        .iter()
        .map(|p| transform_point(Point3::new(p[0], p[1], p[2]), loc.pos, loc.rot))
        .collect();
    if pts.len() < 2 {
        return Err(GenError::Invalid(format!(
            "{label} holds a single point - re-pick a real edge"
        )));
    }
    Ok(pts)
}
