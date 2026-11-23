//! Collect one nominal punch or die opening for the native wire EDM job,
//! before the wire generator offsets it.
//! Provenance: CAMCraft's Wire Contour geometry and named controller recipe.

use super::{FeatureSel, OpParams, SrcRef, WireSide, job};
use crate::geo2d::{Loop, Region};
use crate::{model, posting::agie, schema::Operation, setup};
use flecs_ecs::prelude::World;

pub(crate) fn agie_job(world: &World, op: &Operation, name: &str) -> Result<agie::AgieJob, String> {
    let OpParams::WireContour(wire) = &op.params else {
        return Err("The native wire EDM job requires a Wire Contour".into());
    };
    if !matches!(wire.side, WireSide::Punch | WireSide::Die)
        || !wire.capture
        || wire.taper_deg != 0.0
        || wire.tabs.is_some()
        || wire.burnout.is_some()
        || !wire.skims.is_empty()
        || wire.four_axis.is_some()
    {
        return Err("The native wire EDM job supports one captured vertical punch or die opening. Disable taper, four-axis, tabs, burnout and requested skims; the controller selects trim passes".into());
    }
    let side = match (&wire.side, &op.src) {
        (WireSide::Punch, SrcRef::Entity(_)) => agie::AgieSide::Punch,
        (WireSide::Die, SrcRef::Selection(_)) => agie::AgieSide::Die,
        _ => return Err("The native wire EDM job requires the complete punch model, or the selected rim of a through-opening in the retained die plate".into()),
    };
    let models = model::model_entities(world);
    let [(source, _)] = models.as_slice() else {
        return Err("The native wire EDM job requires exactly one modeled workpiece; additional parts or fixture models can intersect the controller path".into());
    };
    let on_source = match &op.src {
        SrcRef::Entity(name) => source == name,
        SrcRef::Selection(picks) => {
            !picks.is_empty()
                && picks.iter().all(|pick| {
                    pick.entity == *source && matches!(pick.feature, FeatureSel::Chain(_))
                })
        }
        SrcRef::Stock => false,
    };
    if !on_source {
        return Err("The native wire EDM job requires the complete punch, or the edge chains of one through-opening in the retained die plate".into());
    }
    let start = wire
        .start
        .ok_or("The native wire EDM job requires a predrilled 2 mm start hole on the waste side")?;
    let setup = op.setup.as_deref();
    let condition = super::wire_op::condition_of(world, setup, &op.tool, wire)?;
    if condition.id != agie::CONDITION
        || (condition.wire_diameter - 0.25).abs() > 1e-9
        || (condition.gap - agie::PREVIEW_GAP).abs() > 1e-9
    {
        return Err("Select AGIE_QN2 with its unmodified preview gap to request Fine Ra 0.30 µm for cold die steel 0001 and Berco Cut 0.25 mm".into());
    }
    if crate::stock_material::in_force(world, setup).is_some() {
        return Err("The stated stock material conflicts with AGIE_QN2 cold die steel 0001".into());
    }
    let edm = crate::machine::of_setup(world, setup)
        .1
        .and_then(|m| m.edm)
        .ok_or("The native wire EDM job requires a mounted wire EDM profile")?;
    let row = edm.condition(&condition.machine, &condition.id, &[])?;
    if row.role != "rough" || !row.wire_diameter.is_some_and(|d| (d - 0.25).abs() <= 1e-9) {
        return Err("The native wire EDM job requires the AGIE_QN2 machine condition to state role rough and wire diameter 0.25 mm".into());
    }
    if edm.flushing.is_some() || edm.skim_flushing.is_some() || edm.tank.is_some() {
        return Err("The AGIE_QN2 condition controls the dielectric settings; explicit tank or flushing modes are unsupported".into());
    }
    if op.after != super::AfterOperation::None
        || op.coolant != super::CoolantMode::default()
        || crate::program_entry::sequence(world).iter().any(|(_, e)| {
            crate::program_entry::entry_of(world, e).is_some_and(|entry| !entry.suppressed)
        })
    {
        return Err(
            "Native wire EDM jobs do not support additional stops, homing, or coolant overrides".into(),
        );
    }
    if setup::wcs_slot(world, setup) != 0
        || crate::setup_wcs_flip::of_setup(world, setup) != [false; 3]
        || setup::wcs_entity(world, setup)
            .and_then(|e| model::transform_of(world, &e))
            .is_some_and(|p| !crate::setup_align::is_identity_rot(p.rot))
        || crate::setup_align::alignment(world, setup).is_some()
        || setup::effective(world, setup).is_some_and(|s| {
            crate::setup_relative::relative_to(world, &s).is_some()
                || crate::setup_stock::effective(world, &s)
                    == Some(crate::setup_stock::StockSource::FromPrevious)
        })
    {
        return Err(
            "The native wire EDM job requires one unrotated, unflipped setup with its own stock and first work offset"
                .into(),
        );
    }
    let stock = setup::stock_entity(world, setup).ok_or("The native wire EDM job requires rectangular box stock")?;
    if !matches!(
        model::stock_def_of(world, &stock),
        Some(crate::stock::StockShape::Box { .. })
    ) || model::transform_of(world, &stock)
        .is_some_and(|p| !crate::setup_align::is_identity_rot(p.rot))
    {
        return Err("The native wire EDM job requires axis-aligned rectangular box stock".into());
    }
    let job = job::collect_job(world, op, name).map_err(|e| e.to_string())?;
    if !job.fixtures.is_empty() || !job.unread_steel.is_empty() {
        return Err("The native wire EDM post cannot verify controller-generated leads against fixtures".into());
    }
    let (lo, hi) = job.stock_aabb.ok_or("The native wire EDM job requires finite stock bounds")?;
    let [ox, oy, oz] = job.wcs_origin;
    if job.hs.top != Some(hi.z) || job.hs.bottom != Some(lo.z) || oz != hi.z {
        return Err("The native wire EDM job requires a through cut from stock top to bottom and the setup Z origin at stock top".into());
    }
    // Guide heights and posted stock Z are in the setup frame (stock top Z0).
    if edm.lower_guide_z.is_some_and(|z| z > lo.z - oz)
        || edm.upper_guide_z.is_some_and(|z| z < 0.0)
    {
        return Err("The wire work extends outside the stated wire guide heights".into());
    }
    if !job.material.bodies.is_empty() {
        return Err("The native wire EDM job requires one closed vertical prism spanning the stock height; cavities, steps and multiple bodies are unsupported".into());
    }
    // Both caps of the complete source model have the exact planar outline.
    // A union of sampled slices can add rounded vertices along a sloping face.
    let region = crate::geometry::closed_vertical_prism(&job.material.facets, lo.z, hi.z)
        .map_err(|why| format!("The native wire EDM job requires one closed vertical prism spanning the stock height: {why}"))?;
    let ring = match side {
        agie::AgieSide::Punch => {
            let [ring] = region.loops() else {
                return Err("The native wire EDM job requires exactly one closed punch boundary, without openings or additional contours".into());
            };
            ring
        }
        agie::AgieSide::Die => die_opening(&region, &job, [lo.x, lo.y], [hi.x, hi.y])?,
    };
    Ok(agie::AgieJob {
        side,
        contour: ring.points().iter().map(|p| [p.x - ox, p.y - oy]).collect(),
        start: [start[0] - ox, start[1] - oy],
        stock_min: [lo.x - ox, lo.y - oy, lo.z - oz],
        stock_max: [hi.x - ox, hi.y - oy, hi.z - oz],
    })
}

/// The one through-opening of the retained die plate, matched to the selected
/// rim. Reading only the picked loop could post a cut through a blind floor or
/// on an unrelated face, so compare it with the full-height hole of the prism.
fn die_opening<'r>(
    region: &'r Region,
    job: &job::GenJob,
    lo: [f64; 2],
    hi: [f64; 2],
) -> Result<&'r Loop, String> {
    let loops = region.loops();
    if loops.len() != 2 || loops.iter().filter(|r| r.area() > 0.0).count() != 1 {
        return Err("The native wire EDM job requires one retained die plate with exactly one through-opening".into());
    }
    if loops
        .iter()
        .flat_map(|r| r.points())
        .any(|p| p.x < lo[0] || p.x > hi[0] || p.y < lo[1] || p.y > hi[1])
    {
        return Err("The retained die plate must fit inside the stock".into());
    }
    let hole = loops
        .iter()
        .find(|r| r.area() < 0.0)
        .ok_or("The native wire EDM job requires a through-opening in the retained plate")?;
    let (picked, open, _) = job
        .contour_sel
        .as_ref()
        .ok_or("Select the complete rim of the die opening")?
        .as_ref()
        .map_err(ToString::to_string)?;
    let [picked] = picked.as_slice() else {
        return Err("Select the edge chains of exactly one closed die opening rim".into());
    };
    let footprint = |r: &Loop| Region::new(vec![r.clone()]).map_err(|e| e.to_string());
    let (a, b) = (footprint(hole)?, footprint(picked)?);
    if !open.is_empty()
        || !a.difference(&b).is_ok_and(|r| r.is_empty())
        || !b.difference(&a).is_ok_and(|r| r.is_empty())
    {
        return Err(
            "The selected chains must match the full through-opening of the retained die plate"
                .into(),
        );
    }
    Ok(hole)
}
