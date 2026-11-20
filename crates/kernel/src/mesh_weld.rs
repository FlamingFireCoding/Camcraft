//! The machining mesh's vertex weld (#940, #917): the evaluated positions
//! snapped off their last-bit noise onto the 1e-3 mm grid, then welded by
//! exact position, with the side count that proves the weld watertight.
//! Provenance: textbook mesh topology (each side used by exactly two
//! triangles, once in each direction) and IEEE 754 rounding; the snap
//! rule is our own (docs/decisions/917-kernel-parity.md). No HSMWorks
//! material consulted.

use std::collections::HashMap;

use crate::P3;

/// The snap grid (mm): the thousandth of a millimetre that model files
/// state their dimensions in.
pub const SNAP_GRID_MM: f64 = 1e-3;

/// The snap window (mm): only a coordinate this close to a grid value is
/// moved onto it. A coordinate farther off is never moved. The window is
/// far under any machining tolerance (at least a millionth of the body,
/// `MIN_TOL_FRACTION`), so the measured deviation stays true to 1e-9 mm.
pub const SNAP_WINDOW_MM: f64 = 1e-9;

/// `c` moved onto the nearest multiple of [`SNAP_GRID_MM`] when it lies
/// within [`SNAP_WINDOW_MM`] of it; otherwise `c` unchanged. The grid value
/// is `k / 1000` rounded once (IEEE 754), so equal inputs always give
/// equal outputs and 20.000000000000004 becomes exactly 20.
#[must_use]
pub fn snap(c: f64) -> f64 {
    let k = (c / SNAP_GRID_MM).round();
    if !k.is_finite() || k.abs() > 9e15 {
        return c;
    }
    let g = k / 1000.0;
    if (c - g).abs() <= SNAP_WINDOW_MM { g + 0.0 } else { c }
}

/// Vertices snapped ([`snap`]) and welded by exact position (+0.0 and -0.0
/// are one). A triangle whose corners snap or weld onto a repeated vertex
/// carries no area and is dropped; the side count after the weld refuses
/// the mesh if a drop opened a hole.
pub fn weld(mesh: &solidcore_tess::Mesh) -> (Vec<P3>, Vec<[u32; 3]>, Vec<u32>) {
    let mut ids: HashMap<[u64; 3], u32> = HashMap::new();
    let mut positions: Vec<P3> = Vec::new();
    let mut map: Vec<u32> = Vec::with_capacity(mesh.positions.len());
    for p in &mesh.positions {
        let q = [snap(p.x), snap(p.y), snap(p.z)];
        let key = q.map(|c| (c + 0.0).to_bits());
        let next = u32::try_from(positions.len()).unwrap_or(u32::MAX);
        let id = *ids.entry(key).or_insert_with(|| {
            positions.push(q);
            next
        });
        map.push(id);
    }
    let mut tris = Vec::with_capacity(mesh.triangles.len());
    let mut faces = Vec::with_capacity(mesh.triangles.len());
    for (t, f) in mesh.triangles.iter().zip(&mesh.tri_face) {
        let w = t.map(|i| map.get(i as usize).copied().unwrap_or(u32::MAX));
        if w.contains(&u32::MAX) || w[0] == w[1] || w[1] == w[2] || w[2] == w[0] {
            continue;
        }
        tris.push(w);
        faces.push(*f);
    }
    (positions, tris, faces)
}

/// (sides used once, sides used more than twice, sides used twice in the
/// same direction). Each use counts +1 running low-to-high index, -1 back.
pub fn sides(tris: &[[u32; 3]]) -> (usize, usize, usize) {
    let mut count: HashMap<(u32, u32), (u32, i32)> = HashMap::new();
    for t in tris {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let e = count.entry((a.min(b), a.max(b))).or_default();
            e.0 = e.0.saturating_add(1);
            e.1 += if a < b { 1 } else { -1 };
        }
    }
    let open = count.values().filter(|n| n.0 == 1).count();
    let over = count.values().filter(|n| n.0 > 2).count();
    let turned = count.values().filter(|n| n.0 == 2 && n.1 != 0).count();
    (open, over, turned)
}
