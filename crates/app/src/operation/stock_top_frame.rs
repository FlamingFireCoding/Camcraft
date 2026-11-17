//! Stock caps for the #2004 level law in the cutting frame.
//! Provenance: rigid-frame projection of box corners; our own implementation.

use super::*;

/// The stock and material bounds after the full setup-to-cutting transform.
/// Upright operations keep the exact old reading. An indexed operation
/// projects every box corner onto its tool axis, including the origin shift.
pub(super) fn stock_top_in_frame(
    part: &[Triangle],
    part_top: f64,
    stock: Option<(Point3, Point3)>,
    frame: Option<&crate::cl::TiltedFrame>,
) -> Option<f64> {
    let Some(f) = frame else {
        let top = part_top.max(stock.map_or(f64::NEG_INFINITY, |(_, hi)| hi.z));
        return top.is_finite().then_some(top);
    };
    let mut bounds: Option<(Point3, Point3)> = None;
    for p in part.iter().flat_map(|t| [t.a, t.b, t.c]) {
        let (lo, hi) = bounds.get_or_insert((p, p));
        lo.x = lo.x.min(p.x);
        lo.y = lo.y.min(p.y);
        lo.z = lo.z.min(p.z);
        hi.x = hi.x.max(p.x);
        hi.y = hi.y.max(p.y);
        hi.z = hi.z.max(p.z);
    }
    let top = stock
        .into_iter()
        .chain(bounds)
        .flat_map(|(lo, hi)| {
            (0..8).map(move |i| {
                f.z_local(Point3::new(
                    [lo.x, hi.x][(i >> 2) & 1],
                    [lo.y, hi.y][(i >> 1) & 1],
                    [lo.z, hi.z][i & 1],
                ))
            })
        })
        .fold(f64::NEG_INFINITY, f64::max);
    top.is_finite().then_some(top)
}

#[cfg(test)]
mod tests {
    use super::super::area::{ladder_planes, stock_top};
    use super::*;

    #[test]
    fn indexed_area_planes_keep_top_z_under_rotated_stock() {
        use crate::commands::{History, SetStock};
        let world = crate::world::new_world();
        let mut h = History::default();
        h.execute(
            &world,
            SetStock::new(
                &world,
                crate::stock::StockShape::Box {
                    dx: 80.0,
                    dy: 60.0,
                    dz: 20.0,
                },
                crate::schema::Location::default(),
            )
            .expect("stock"),
        );
        for (b, origin, cap) in [
            (90.0, [0.0; 3], 40.0),
            (45.0, [4.0, 0.0, 2.0], 54.0 / 2.0_f64.sqrt()),
        ] {
            let f = Tilt {
                basis_version: 1,
                a_deg: 0.0,
                b_deg: b,
                origin,
            }
            .frame()
            .expect("frame");
            let mt = stock_top(&world, None, &[], f64::NEG_INFINITY, Some(&f)).expect("stock cap");
            assert!((mt - cap).abs() < 1e-9, "B{b}: {mt} != {cap}");
            assert_eq!(
                ladder_planes(cap - 2.0, Some(mt), cap - 8.0, 3.0),
                vec![cap - 2.0, cap - 5.0, cap - 8.0]
            );
        }
    }

    #[test]
    fn indexed_material_bounds_supply_the_stock_cap_without_setup_stock() {
        let part = [Triangle::new(
            Point3::new(-40.0, -30.0, 0.0),
            Point3::new(40.0, 30.0, 20.0),
            Point3::new(40.0, -30.0, 0.0),
        )];
        let f = Tilt {
            basis_version: 1,
            a_deg: 0.0,
            b_deg: 90.0,
            origin: [3.0, 0.0, 2.0],
        }
        .frame()
        .expect("frame");
        let cap = stock_top_in_frame(&part, 20.0, None, Some(&f)).expect("part cap");
        assert!((cap - 37.0).abs() < 1e-9, "{cap}");
        assert_eq!(stock_top_in_frame(&part, 20.0, None, None), Some(20.0));
    }
}
