//! Cached lookup indexes for shared `Arc<Features>` instances so repeated
//! selection, stitch, and highlight queries resolve chains and patches in O(1)
//! without re-walking all feature chains or patch groups per call.
//!
//! Provenance: our own per-Features lookup index built over `geometry::feature` primitives.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

use super::*;

const INDEX_CACHE_CAP: usize = 32;

type ChainMap = HashMap<ChainId, Chain>;
type PatchIndex = (Vec<Patch>, HashMap<PatchId, Vec<PatchRing>>);

#[derive(Default)]
struct FeatureIndex {
    chains_and_runs: Option<ChainMap>,
    singletons: Option<ChainMap>,
    patches: Option<PatchIndex>,
}

thread_local! {
    static INDEX_CACHE: RefCell<Vec<(Weak<Features>, FeatureIndex)>> = const { RefCell::new(Vec::new()) };
}

fn with_index<R>(features: &Arc<Features>, f: impl FnOnce(&mut FeatureIndex) -> R) -> R {
    INDEX_CACHE.with(|cell| {
        let mut entries = cell.borrow_mut();
        entries.retain(|(weak, _)| weak.strong_count() > 0);
        if let Some(pos) = entries
            .iter()
            .position(|(weak, _)| std::ptr::eq(weak.as_ptr(), Arc::as_ptr(features)))
        {
            entries[pos..].rotate_left(1);
            if let Some((_, idx)) = entries.last_mut() {
                return f(idx);
            }
        }
        if entries.len() >= INDEX_CACHE_CAP {
            entries.remove(0);
        }
        entries.push((Arc::downgrade(features), FeatureIndex::default()));
        if let Some((_, idx)) = entries.last_mut() {
            return f(idx);
        }
        let mut fallback = FeatureIndex::default();
        f(&mut fallback)
    })
}

fn build_chains_and_runs(features: &Features) -> HashMap<ChainId, Chain> {
    let cs = chains(features);
    let mut map = HashMap::with_capacity(cs.len().saturating_mul(2));
    for c in &cs {
        map.entry(c.id).or_insert_with(|| c.clone());
    }
    let lattice = runs::lattice_of(features);
    let mut runs_buf = Vec::new();
    for c in &cs {
        runs_buf.clear();
        runs::split_into(features, &lattice, c, &mut runs_buf);
        for r in &runs_buf {
            map.entry(r.id).or_insert_with(|| r.clone());
        }
    }
    map
}

fn build_singletons(features: &Features) -> HashMap<ChainId, Chain> {
    let lattice = runs::lattice_of(features);
    let mut map = HashMap::with_capacity(features.edges.len());
    for i in 0..features.edges.len() as u32 {
        if let Some(r) = runs::run_with(features, &lattice, &[i]) {
            map.entry(r.id).or_insert(r);
        }
    }
    map
}

fn build_patches(features: &Features) -> PatchIndex {
    let (lattice, flat) = patch_lattice(features);
    let groups = patch_groups(features);
    let mut list = Vec::with_capacity(groups.len());
    let mut rings_map = HashMap::with_capacity(groups.len());
    for g in groups {
        let (pid, rings) = patch_identity_with_lattice(features, &g, &lattice, &flat);
        let mut faces: Vec<FaceId> = g.iter().map(|f| f.id).collect();
        faces.sort_unstable();
        list.push(Patch { id: pid, faces });
        rings_map.entry(pid).or_insert(rings);
    }
    list.sort_unstable_by_key(|p| p.id);
    (list, rings_map)
}

/// Cached variant of [`resolve_chain`] for shared `Arc<Features>` values.
/// Produces the exact same chain, run, or Alt-pick singleton as `resolve_chain`.
#[must_use]
pub fn resolve_chain_arc(features: &Arc<Features>, id: ChainId) -> Option<Chain> {
    with_index(features, |idx| {
        let main = idx
            .chains_and_runs
            .get_or_insert_with(|| build_chains_and_runs(features));
        if let Some(c) = main.get(&id) {
            return Some(c.clone());
        }
        let singles = idx
            .singletons
            .get_or_insert_with(|| build_singletons(features));
        singles.get(&id).cloned()
    })
}

/// Cached variant of [`patch_rings`] for shared `Arc<Features>` values.
#[must_use]
pub fn patch_rings_arc(features: &Arc<Features>, id: PatchId) -> Option<Vec<PatchRing>> {
    with_index(features, |idx| {
        let (_, map) = idx.patches.get_or_insert_with(|| build_patches(features));
        map.get(&id).cloned()
    })
}

/// Cached variant of [`patches`] for shared `Arc<Features>` values.
#[must_use]
pub fn patches_arc(features: &Arc<Features>) -> Vec<Patch> {
    with_index(features, |idx| {
        let (list, _) = idx.patches.get_or_insert_with(|| build_patches(features));
        list.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::primitives::{cube, cylinder};
    use crate::geometry::Point3;

    #[test]
    fn resolve_chain_arc_matches_uncached_for_chains_runs_singletons_and_missing() {
        let mesh = cube(Point3::ZERO, Point3::new(40.0, 25.0, 10.0));
        let f = Arc::new(features_of_facets(&mesh.facets));
        for c in chains(&f) {
            let a = resolve_chain_arc(&f, c.id);
            let b = resolve_chain(&f, c.id);
            assert_eq!(a.as_ref().map(|x| (&x.edges, x.closed, x.id)), b.as_ref().map(|x| (&x.edges, x.closed, x.id)));
        }
        for r in edge_runs(&f) {
            let a = resolve_chain_arc(&f, r.id);
            let b = resolve_chain(&f, r.id);
            assert_eq!(a.as_ref().map(|x| (&x.edges, x.closed, x.id)), b.as_ref().map(|x| (&x.edges, x.closed, x.id)));
        }
        if !f.edges.is_empty() {
            let single = run_of_edges(&f, &[0]).expect("singleton run");
            let a = resolve_chain_arc(&f, single.id);
            let b = resolve_chain(&f, single.id);
            assert_eq!(a.as_ref().map(|x| (&x.edges, x.closed, x.id)), b.as_ref().map(|x| (&x.edges, x.closed, x.id)));
        }
        assert!(resolve_chain_arc(&f, 0xDEAD_BEEF_CAFE_BABE).is_none());
    }

    #[test]
    fn patch_apis_arc_match_uncached() {
        let mesh = cylinder(20.0, 10.0, 24);
        let f = Arc::new(features_of_facets(&mesh.facets));
        let uncached = patches(&f);
        let cached = patches_arc(&f);
        assert!(!uncached.is_empty(), "cylinder produces a smooth cylindrical patch");
        assert_eq!(cached.len(), uncached.len());
        for (c, u) in cached.iter().zip(uncached.iter()) {
            assert_eq!(c.id, u.id);
            assert_eq!(c.faces, u.faces);
            let cr = patch_rings_arc(&f, c.id);
            let ur = patch_rings(&f, u.id);
            assert_eq!(cr.as_ref().map(|v| v.len()), ur.as_ref().map(|v| v.len()));
        }
        assert!(patch_rings_arc(&f, 0xDEAD_BEEF_CAFE_BABE).is_none());
    }
}
