//! Pins for `cells_across`: a box edge a few ulps longer than a whole
//! number of cells gets that whole number of columns, not one more.
//! Provenance: synthetic boxes; 12.7 mm two ulps long is what a 12.7 mm
//! edge reads after a rigid move in floating point.

use super::super::backend::{Backend, Stock};
use super::super::carry_history::History;
use super::*;

/// 12.7 mm, `k` ulps long.
fn long(k: u64) -> f64 {
    f64::from_bits(12.7f64.to_bits() + k)
}

#[test]
fn a_whole_number_of_cells_up_to_rounding_is_that_number() {
    assert_eq!(cells_across(long(2), 0.254), 50);
    assert_eq!(cells_across(12.7, 0.254), 50);
    assert_eq!(cells_across(1.0, 0.25), 4);
    // A real part of a cell still gets its column.
    assert_eq!(cells_across(12.8, 0.254), 51);
    assert_eq!(cells_across(12.7 + 1e-6, 0.254), 51);
    assert_eq!(cells_across(0.1, 0.254), 1);
    assert_eq!(cells_across(1e-10, 0.254), 1);
}

#[test]
fn no_material_stands_past_a_box_edge_read_a_few_ulps_long() {
    let (x0, x1) = (-long(2), -0.0);
    let stock = Stock::block(Backend::Column, x0, 0.0, x1, 1.0, 5.0, 0.254).expect("block");
    let Stock::Column(d) = &stock else { panic!("column arm") };
    assert_eq!(d.nx, 50, "the last column would be {} mm wide", x1 - (x0 + 50.0 * 0.254));
    assert_eq!(stock.top(-0.1, 0.5), 5.0);
    assert!(stock.top(0.1, 0.5).is_nan(), "material 0.1 mm past the box edge");
}

#[test]
fn the_cell_ceiling_gate_counts_the_same_columns() {
    // 4096 cells a few ulps long: inside the ceiling, not refused as 4097.
    let x1 = 4096.0 * 0.254;
    let x0 = -(x1 * (1.0 + 4.0 * f64::EPSILON) - x1);
    assert!((x1 - x0) / 0.254 > 4096.0);
    assert!(Stock::block(Backend::Column, x0, 0.0, x1, 0.254, 1.0, 0.254).is_ok());
}

#[test]
fn the_exact_carry_read_counts_the_blocks_columns() {
    // The box a side turn reads back: the same edges, a finite floor.
    let mut h = History::block(-long(2), 0.0, -0.0, 1.0, 5.0, 0.254);
    h.lo.z = 0.0;
    let body = h.read(false).expect("read");
    assert_eq!((body.dex.nx, body.dex.ny), (50, 4));
}
