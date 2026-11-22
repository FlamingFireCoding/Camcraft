//! #2769: a setup's stock as a controller blank - read from the document
//! (the stock entity, its shape and placement, the WCS) for posts that draw
//! the blank (FANUC MANUAL GUIDE i B-63874EN G1902 / G1900, behaviour only).

use crate::geometry::{Point3, transform_point};
use crate::model;
use crate::post::stock_blank::StockBlank;
use crate::posting::{OffsetFrame, offset_frame};
use crate::setup_stock::StockSource;
use crate::stock::StockShape;
use flecs_ecs::prelude::*;

/// A unit vector component within this of 1 is that axis.
const AXIS_TOL: f64 = 1e-9;

/// The blank of the stock that operations of the `setup` context cut: a box
/// or a cylinder with its world axis when the stock is one, square to the
/// machine axes, under an unrotated, unflipped WCS; else the reason.
#[must_use]
pub(crate) fn of(world: &World, setup: Option<&str>) -> StockBlank {
    if let Some(name) = crate::setup::effective(world, setup)
        && crate::setup_stock::effective(world, &name) == Some(StockSource::FromPrevious)
    {
        return StockBlank::Unsupported("from-previous");
    }
    let Some(stock) = crate::setup::stock_entity(world, setup) else {
        return StockBlank::Unsupported("none");
    };
    let Some(aabb) = model::stock_world_aabb_for(world, setup) else {
        return StockBlank::Unsupported("none");
    };
    if let Some(loc) = crate::setup::wcs_entity(world, setup).and_then(|e| model::transform_of(world, &e))
        && !matches!(offset_frame(&loc), OffsetFrame::Square)
    {
        return StockBlank::Unsupported("rotated-wcs");
    }
    if crate::setup_wcs_flip::of_setup(world, setup).iter().any(|f| *f) {
        return StockBlank::Unsupported("flipped-wcs");
    }
    let rot = model::transform_of(world, &stock).unwrap_or_default().rot;
    if !crate::stock_follow::is_square(rot) {
        return StockBlank::Unsupported("rotated-stock");
    }
    let z_bottom = aabb.min.z;
    match model::stock_def_of(world, &stock) {
        Some(StockShape::Box { .. }) => StockBlank::Box { z_bottom },
        Some(StockShape::Cylinder { .. }) => {
            // The stock-local Z axis (the bar's) in world, a signed axis here.
            let a = transform_point(Point3::new(0.0, 0.0, 1.0), Point3::ZERO, rot);
            let axis = if (a.z.abs() - 1.0).abs() <= AXIS_TOL {
                'Z'
            } else if (a.x.abs() - 1.0).abs() <= AXIS_TOL {
                'X'
            } else {
                'Y'
            };
            StockBlank::Cylinder { axis, z_bottom }
        }
        Some(StockShape::FromModel { .. }) => StockBlank::Unsupported("from-model"),
        None => StockBlank::Unsupported("none"),
    }
}
