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

#[test]
fn the_field_box_gate_counts_the_same_columns() {
    // The column case above, on the field arm's own gate.
    let x1 = 4096.0 * 0.254;
    let x0 = -(x1 * (1.0 + 4.0 * f64::EPSILON) - x1);
    assert!((x1 - x0) / 0.254 > 4096.0);
    assert!(Stock::block(Backend::Field { band: 4 }, x0, 0.0, x1, 0.254, 1.0, 0.254).is_ok());
}

#[test]
fn both_cylinder_gates_count_the_same_columns() {
    use crate::stock::StockShape;
    let cell = 0.254;
    let gate = |diameter: f64| {
        let shape = StockShape::Cylinder { diameter, height: 1.0 };
        let column = Stock::from_shape(Backend::Column, &shape, cell);
        let field = Stock::from_shape(Backend::Field { band: 4 }, &shape, cell);
        (column.map(|_| ()).map_err(|e| e.to_string()), field.map(|_| ()).map_err(|e| e.to_string()))
    };
    // 4096 cells two ulps long: inside the grid ceiling on both arms. The
    // column arm builds it; the field arm passes the same gate and then
    // refuses by name on its own extra truth, the per-facet cell budget
    // (a cap facet of a round this wide spans millions of cells), which
    // it says for any round this size, whole cells or not.
    let (column, field) = gate(f64::from_bits((4096.0_f64 * cell).to_bits() + 2));
    assert!(column.is_ok(), "column arm: {column:?}");
    let said = field.expect_err("the field arm's facet budget");
    assert!(!said.contains("grid would be"), "the field arm's grid gate refused a whole number of cells: {said}");
    assert!(said.contains("needs tessellating"), "{said}");
    // A real part of a cell more: refused on both arms, in the same words.
    let (column, field) = gate(4096.0 * cell + 0.01);
    assert_eq!(column, field);
    assert!(column.as_ref().is_err_and(|e| e.contains("4097x4097")), "{column:?}");
}
