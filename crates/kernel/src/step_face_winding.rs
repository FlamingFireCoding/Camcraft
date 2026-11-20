//! The faces of a STEP closed shell whose loops run against their
//! neighbours (#917, the note of #1576): solidcore's reader orients them
//! with the shell and leaves no trace, so the seam finds them in the file
//! and names them, as the earlier importer did ("shell #7: 1 face(s) wound
//! against their neighbours were flipped: face #32 (3 of 6)").
//! Provenance: ISO 10303-42 topology (CLOSED_SHELL, ORIENTED_FACE,
//! FACE_BOUND orientation, ORIENTED_EDGE orientation: in a closed shell
//! each edge is used twice, once in each direction); consistent orientation
//! by relative parity across shared edges (Botsch et al., "Polygon Mesh
//! Processing", 2010, ch. 1) as a union-find with parity (Tarjan, J. ACM
//! 22(2), 1975). Read through the public p21 API of solidcore-step. Our
//! own code; no HSMWorks material consulted.

use std::collections::{BTreeMap, BTreeSet};

use solidcore_step::p21::{Exchange, Param};

/// The most faces one note names; the rest are counted.
const NAMED: usize = 8;

/// One note for each closed shell that has faces wound against their
/// neighbours, naming the faces (the smaller side of each connected
/// group). A shell whose edge uses do not resolve to one winding (an edge
/// used other than twice, or an odd cycle) gets no note: the reader and
/// the mesh judge it.
pub(crate) fn notes(ex: &Exchange) -> Vec<String> {
    let mut out = Vec::new();
    for id in closed_shells(ex) {
        if let Some(note) = shell_note(ex, id) {
            out.push(note);
        }
    }
    out
}

/// Every CLOSED_SHELL, and every outer shell of a MANIFOLD_SOLID_BREP
/// whatever its keyword, by id.
fn closed_shells(ex: &Exchange) -> BTreeSet<u64> {
    let mut found = BTreeSet::new();
    for (id, e) in &ex.entities {
        if e.has("CLOSED_SHELL") && !e.has("ORIENTED_CLOSED_SHELL") {
            found.insert(*id);
        } else if let Some(outer) = e.record("MANIFOLD_SOLID_BREP").and_then(|p| p.get(1)).and_then(Param::as_ref_id) {
            found.insert(outer);
        }
    }
    found
}

fn flag(p: Option<&Param>) -> Option<bool> {
    p.and_then(Param::as_bool)
}

/// The face entity of a shell member and whether the shell uses it as
/// written.
fn face_of(ex: &Exchange, id: u64) -> Option<(u64, bool)> {
    let e = ex.get(id)?;
    if let Some(p) = e.record("ORIENTED_FACE") {
        let (inner, fwd) = face_of(ex, p.get(2)?.as_ref_id()?)?;
        return Some((inner, fwd == flag(p.get(3))?));
    }
    Some((id, true))
}

/// The edge entity of a loop member and whether the loop runs along it.
fn edge_of(ex: &Exchange, id: u64, depth: u32) -> Option<(u64, bool)> {
    let e = ex.get(id)?;
    match e.record("ORIENTED_EDGE") {
        Some(p) if depth < 4 => {
            let (inner, fwd) = edge_of(ex, p.get(3)?.as_ref_id()?, depth + 1)?;
            Some((inner, fwd == flag(p.get(4))?))
        }
        Some(_) => None,
        None => Some((id, true)),
    }
}

/// Each edge use of one face, `(edge, runs along it)`, with the face's use
/// in the shell applied. `None`: a member this check does not read.
fn edge_uses(ex: &Exchange, face: u64, fwd: bool) -> Option<Vec<(u64, bool)>> {
    let e = ex.get(face)?;
    let p = ["ADVANCED_FACE", "FACE_SURFACE", "FACE"].iter().find_map(|n| e.record(n))?;
    let mut uses = Vec::new();
    for b in p.get(1)?.as_list()? {
        let bound = ex.get(b.as_ref_id()?)?;
        let bp = bound.record("FACE_OUTER_BOUND").or_else(|| bound.record("FACE_BOUND"))?;
        let along = flag(bp.get(2))? == fwd;
        let Some(lp) = ex.get(bp.get(1)?.as_ref_id()?)?.record("EDGE_LOOP") else {
            continue; // a VERTEX_LOOP uses no edge
        };
        for oe in lp.get(1)?.as_list()? {
            let (edge, dir) = edge_of(ex, oe.as_ref_id()?, 0)?;
            uses.push((edge, dir == along));
        }
    }
    Some(uses)
}

/// Union-find with parity: a face's winding relative to its root.
struct Parity {
    up: Vec<usize>,
    odd: Vec<bool>,
}

impl Parity {
    fn find(&mut self, f: usize) -> (usize, bool) {
        let (mut r, mut odd) = (f, false);
        while let Some(&u) = self.up.get(r) {
            if u == r {
                break;
            }
            odd ^= self.odd.get(r).copied().unwrap_or(false);
            r = u;
        }
        (r, odd)
    }

    /// Joins `a` and `b` with `b` wound `differ` against `a`; `false` when
    /// that contradicts what is known.
    fn join(&mut self, a: usize, b: usize, differ: bool) -> bool {
        let ((ra, pa), (rb, pb)) = (self.find(a), self.find(b));
        if ra == rb {
            return (pa ^ pb) == differ;
        }
        if let (Some(u), Some(o)) = (self.up.get_mut(rb), self.odd.get_mut(rb)) {
            (*u, *o) = (ra, pa ^ pb ^ differ);
        }
        true
    }
}

fn shell_note(ex: &Exchange, shell: u64) -> Option<String> {
    let members: Vec<u64> = ex
        .get(shell)?
        .params()
        .get(1)?
        .as_list()?
        .iter()
        .map(Param::as_ref_id)
        .collect::<Option<_>>()?;
    let mut by_edge: BTreeMap<u64, Vec<(usize, bool)>> = BTreeMap::new();
    let mut names = Vec::with_capacity(members.len());
    for (i, m) in members.iter().enumerate() {
        let (face, fwd) = face_of(ex, *m)?;
        names.push(face);
        for (edge, dir) in edge_uses(ex, face, fwd)? {
            by_edge.entry(edge).or_default().push((i, dir));
        }
    }
    let n = members.len();
    let mut parity = Parity { up: (0..n).collect(), odd: vec![false; n] };
    for uses in by_edge.values() {
        let [(a, da), (b, db)] = uses.as_slice() else { return None };
        // Two uses the same way: the two faces are wound against each other.
        if a != b && !parity.join(*a, *b, da == db) {
            return None;
        }
    }
    // In each group the side with fewer faces is the one wound against the
    // rest (a tie: the side without the group's lowest face).
    let mut groups: BTreeMap<usize, (Vec<usize>, Vec<usize>)> = BTreeMap::new();
    for f in 0..n {
        let (root, odd) = parity.find(f);
        let g = groups.entry(root).or_default();
        if odd { g.1.push(f) } else { g.0.push(f) }
    }
    let mut flipped: Vec<usize> = Vec::new();
    for (even, odd) in groups.into_values() {
        let lowest_even = even.first().copied().unwrap_or(usize::MAX) < odd.first().copied().unwrap_or(usize::MAX);
        let take_odd = odd.len() < even.len() || (odd.len() == even.len() && lowest_even);
        flipped.extend(if take_odd { odd } else { even });
    }
    if flipped.is_empty() {
        return None;
    }
    flipped.sort_unstable();
    let mut said: Vec<String> = flipped
        .iter()
        .take(NAMED)
        .filter_map(|&f| names.get(f).map(|id| format!("face #{id} ({} of {n})", f + 1)))
        .collect();
    if flipped.len() > NAMED {
        said.push(format!("{} more", flipped.len() - NAMED));
    }
    Some(format!(
        "shell #{shell}: {} face(s) wound against their neighbours were flipped: {}",
        flipped.len(),
        said.join(", ")
    ))
}
