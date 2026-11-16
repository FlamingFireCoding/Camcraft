//! Cached world-space highlight polylines for selected picks, viewport hover,
//! and Geometry-row hover. Resolves `GeoSel` picks once at selection time instead
//! of re-walking features per frame.
//!
//! Provenance: our own selection highlight cache built over `geometry::feature`.

use std::sync::{Arc, Mutex};

use camcraft_app::geometry::{Point3, feature, transform_point};
use camcraft_app::model;
use camcraft_app::operation::{FeatureSel, GeoSel};

use crate::frame_cache::{EntityGuard, WorldStamp, guards_for_picks};
use crate::state::State;

/// One resolved feature outline in world space.
#[derive(Clone, Debug)]
pub(crate) struct WorldPolyline {
    pub pts: Vec<[f64; 3]>,
    pub closed: bool,
}

/// Cached resolved geometry for a set of picks or one hovered feature.
#[derive(Clone, Debug, Default)]
pub(crate) struct ResolvedHighlights {
    pub id: u64,
    pub polylines: Vec<WorldPolyline>,
    pub vertices: Vec<[f64; 3]>,
}

#[derive(Clone, PartialEq)]
struct SelectionKey {
    stamp: WorldStamp,
    op_ui_name: String,
    picks: Vec<GeoSel>,
    hidden_models: Vec<String>,
    hidden_stocks: Vec<String>,
    hidden_paths: Vec<String>,
    shown_setups: Vec<String>,
    guards: Vec<EntityGuard>,
}

#[derive(Clone, PartialEq)]
struct SingleKey {
    stamp: WorldStamp,
    pick: Option<GeoSel>,
    hidden: bool,
    guard: Option<EntityGuard>,
}

#[derive(Default)]
pub(crate) struct HighlightInner {
    pub(crate) next_id: u64,
    pub(crate) selection_builds: u64,
    selection: Option<(SelectionKey, Arc<ResolvedHighlights>)>,
    hover: Option<(SingleKey, Arc<ResolvedHighlights>)>,
    row_hover: Option<(SingleKey, Arc<ResolvedHighlights>)>,
    pub(crate) selected_mesh: Option<crate::pick_highlight_mesh::CachedMesh>,
    pub(crate) hover_mesh: Option<crate::pick_highlight_mesh::CachedMesh>,
    pub(crate) row_hover_mesh: Option<crate::pick_highlight_mesh::CachedMesh>,
}

/// Per-`State` cache of resolved pick outlines and batched screen meshes.
#[derive(Default)]
pub(crate) struct HighlightCache {
    pub(crate) inner: Mutex<HighlightInner>,
}

pub(crate) fn lock(state: &State) -> std::sync::MutexGuard<'_, HighlightInner> {
    state
        .pick_highlight
        .inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Number of times selected-feature world polylines were rebuilt for this state.
#[cfg(test)]
#[must_use]
pub(crate) fn selection_builds(state: &State) -> u64 {
    lock(state).selection_builds
}

fn append_feature(
    state: &State,
    item: &GeoSel,
    polylines: &mut Vec<WorldPolyline>,
    vertices: &mut Vec<[f64; 3]>,
) {
    if state.hidden_in_view(&item.entity) {
        return;
    }
    let Some(entity) = model::find_by_name(&state.world, &item.entity) else {
        return;
    };
    let Some(features) = model::features_of(&state.world, &entity) else {
        return;
    };
    let loc = camcraft_app::setup_align::view_pose(&state.world, &entity).unwrap_or_default();
    let world_pts = |pts: &[[f64; 3]]| -> Vec<[f64; 3]> {
        pts.iter()
            .map(|p| {
                let w = transform_point(Point3::new(p[0], p[1], p[2]), loc.pos, loc.rot);
                [w.x, w.y, w.z]
            })
            .collect()
    };
    match item.feature {
        FeatureSel::Face(fid) => {
            let Some(face) = features.faces.iter().find(|f| f.id == fid) else {
                return;
            };
            for l in &face.loops {
                let local: Vec<[f64; 3]> = l
                    .verts
                    .iter()
                    .filter_map(|&vi| features.verts.get(vi as usize).copied())
                    .collect();
                polylines.push(WorldPolyline {
                    pts: world_pts(&local),
                    closed: !matches!(l.kind, feature::LoopKind::Open),
                });
            }
        }
        FeatureSel::Chain(cid) => {
            let Some(chain) = feature::resolve_chain_arc(&features, cid) else {
                return;
            };
            let local = feature::chain_points(&features, &chain);
            polylines.push(WorldPolyline {
                pts: world_pts(&local),
                closed: chain.closed,
            });
        }
        FeatureSel::Patch(pid) => {
            let Some(rings) = feature::patch_rings_arc(&features, pid) else {
                return;
            };
            for r in &rings {
                let local: Vec<[f64; 3]> = r
                    .verts
                    .iter()
                    .filter_map(|&vi| features.verts.get(vi as usize).copied())
                    .collect();
                polylines.push(WorldPolyline {
                    pts: world_pts(&local),
                    closed: r.closed,
                });
            }
        }
        FeatureSel::Vertex(vid) => {
            let Some(v) = feature::resolve_vertex(&features, vid)
                .and_then(|vi| features.verts.get(vi as usize))
            else {
                return;
            };
            let w = transform_point(Point3::new(v[0], v[1], v[2]), loc.pos, loc.rot);
            vertices.push([w.x, w.y, w.z]);
        }
    }
}

fn next_resolved(
    inner: &mut HighlightInner,
    polylines: Vec<WorldPolyline>,
    vertices: Vec<[f64; 3]>,
) -> Arc<ResolvedHighlights> {
    inner.next_id = inner.next_id.wrapping_add(1);
    Arc::new(ResolvedHighlights {
        id: inner.next_id,
        polylines,
        vertices,
    })
}

/// Cached resolved world outlines and points for the currently shown selection.
pub(crate) fn resolved_selected(state: &State) -> Arc<ResolvedHighlights> {
    let stamp = WorldStamp::of(state);
    let picks = crate::interact::shown_picks(state);
    {
        let inner = lock(state);
        if let Some((key, cached)) = &inner.selection
            && key.stamp == stamp
            && key.op_ui_name == state.op_ui.name
            && key.picks == picks
            && key.hidden_models == state.hidden_models
            && key.hidden_stocks == state.hidden_stocks
            && key.hidden_paths == state.hidden_paths
            && key.shown_setups == state.shown_setups
            && key.guards.iter().all(|g| g.matches(state))
        {
            return Arc::clone(cached);
        }
    }
    let guards = guards_for_picks(state, &picks);
    let mut polylines = Vec::new();
    let mut vertices = Vec::new();
    for item in &picks {
        append_feature(state, item, &mut polylines, &mut vertices);
    }
    let key = SelectionKey {
        stamp,
        op_ui_name: state.op_ui.name.clone(),
        picks,
        hidden_models: state.hidden_models.clone(),
        hidden_stocks: state.hidden_stocks.clone(),
        hidden_paths: state.hidden_paths.clone(),
        shown_setups: state.shown_setups.clone(),
        guards,
    };
    let mut inner = lock(state);
    let empty_unchanged = key.picks.is_empty()
        && inner
            .selection
            .as_ref()
            .is_some_and(|(old_key, _)| old_key.picks.is_empty());
    let resolved = next_resolved(&mut inner, polylines, vertices);
    if !(key.picks.is_empty() && (empty_unchanged || inner.selection_builds == 0)) {
        inner.selection_builds = inner.selection_builds.saturating_add(1);
    }
    inner.selection = Some((key, Arc::clone(&resolved)));
    resolved
}

fn resolve_single_cached(
    state: &State,
    pick: Option<GeoSel>,
    get_slot: impl Fn(&mut HighlightInner) -> &mut Option<(SingleKey, Arc<ResolvedHighlights>)>,
) -> Arc<ResolvedHighlights> {
    let stamp = WorldStamp::of(state);
    let hidden = pick
        .as_ref()
        .is_some_and(|p| state.hidden_in_view(&p.entity));
    {
        let mut inner = lock(state);
        if let Some((key, cached)) = get_slot(&mut inner)
            && key.stamp == stamp
            && key.pick == pick
            && key.hidden == hidden
            && key.guard.as_ref().is_none_or(|g| g.matches(state))
        {
            return Arc::clone(cached);
        }
    }
    let guard = pick
        .as_ref()
        .map(|p| EntityGuard::capture(state, &p.entity));
    let mut polylines = Vec::new();
    let mut vertices = Vec::new();
    if let Some(item) = &pick {
        append_feature(state, item, &mut polylines, &mut vertices);
    }
    let key = SingleKey {
        stamp,
        pick,
        hidden,
        guard,
    };
    let mut inner = lock(state);
    let resolved = next_resolved(&mut inner, polylines, vertices);
    *get_slot(&mut inner) = Some((key, Arc::clone(&resolved)));
    resolved
}

pub(crate) fn resolved_hover(state: &State) -> Arc<ResolvedHighlights> {
    let pick = state.hover.as_ref().and_then(|hit| {
        hit.feature.map(|feature| GeoSel {
            entity: hit.entity.clone(),
            feature,
        })
    });
    resolve_single_cached(state, pick, |inner| &mut inner.hover)
}

pub(crate) fn resolved_row_hover(state: &State) -> Arc<ResolvedHighlights> {
    resolve_single_cached(state, state.geo_hover_drawn.clone(), |inner| &mut inner.row_hover)
}

/// Project cached world polylines (and optional vertex crosses) to viewport UV segments.
#[cfg(test)]
pub(crate) fn project_segments(
    resolved: &ResolvedHighlights,
    include_vertices: bool,
    mvp: &[[f32; 4]; 4],
    aspect: f64,
) -> Vec<([f32; 2], [f32; 2])> {
    let mut out = Vec::new();
    for poly in &resolved.polylines {
        let n = poly.pts.len();
        if n < 2 {
            continue;
        }
        let uvs: Vec<Option<[f32; 2]>> = poly
            .pts
            .iter()
            .map(|&p| crate::interact::project_uv(mvp, p))
            .collect();
        let last = if poly.closed { n } else { n - 1 };
        for i in 0..last {
            if let (Some(a), Some(b)) = (uvs[i], uvs[(i + 1) % n])
                && a != b
            {
                out.push((a, b));
            }
        }
    }
    if include_vertices {
        for &w in &resolved.vertices {
            crate::vertex_pick::vertex_marker(mvp, w, aspect, &mut out);
        }
    }
    out
}
