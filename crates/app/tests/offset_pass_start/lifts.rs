//! #3747: the lifts of a 2D Offset path inside one level, and the lifts
//! that its link notes name. A lift leaves a floor plane with a planar cut
//! at that plane still to come (a level change is not a lift). Provenance:
//! our own test meter; no vendor material consulted.

use camcraft_app::cl::{MoveKind, Toolpath};

/// Is move `i` a planar feed at `z`?
fn floor_at(t: &Toolpath, i: usize, z: f64) -> bool {
    t.moves.get(i).is_some_and(|m| m.kind != MoveKind::Rapid && (m.from.z - z).abs() < 1e-9 && (m.to.z - z).abs() < 1e-9)
}

/// The lifts inside a level: each move that rises off a planar feed at
/// `z` when another planar feed at `z` comes later.
pub fn level_lifts(t: &Toolpath) -> usize {
    let n = t.moves.len();
    (0..n)
        .filter(|&i| {
            let m = &t.moves[i];
            let z = m.from.z;
            m.to.z > z + 1e-6 && i > 0 && floor_at(t, i - 1, z) && (i + 1..n).any(|k| floor_at(t, k, z))
        })
        .count()
}

/// The lifts that the link notes name ("link: ... N time(s) ...").
pub fn named_lifts(t: &Toolpath) -> usize {
    t.notes
        .iter()
        .filter(|n| n.starts_with("link: "))
        .filter_map(|n| n.split(" time(s)").next()?.rsplit(' ').next()?.parse::<usize>().ok())
        .sum()
}

/// #3747 pin: the maintainer's fig8 (#3741), Inside and Outside. Each lift
/// inside a level has a named reason on the path (the link notes).
#[test]
#[ignore = "heavy lane (#3747 fig8 Inside and Outside): two generations run past the 60 s debug budget - run with --release -- --ignored"]
fn fig8_start_sides_lift_only_with_a_named_reason() {
    for start in [super::PassStart::Inside, super::PassStart::Outside] {
        let (world, mut h, name, stock) = super::fig8_world();
        let (path, _) = super::offset_op(&world, &mut h, &name, stock.0.z, start);
        let (up, named) = (level_lifts(&path), named_lifts(&path));
        eprintln!("fig8 {start:?}: {up} lift(s) inside a level, {named} named: {:?}", path.notes);
        assert_eq!(up, named, "{start:?}: a lift with no named reason: {:?}", path.notes);
    }
}
