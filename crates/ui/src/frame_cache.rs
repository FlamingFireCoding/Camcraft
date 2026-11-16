//! Per-`State` caches for world-space scene data: transformed entity vertices,
//! cached cutting regions and outlines, and toolpath overlay lines. Every entry
//! keys against `WorldStamp` and exact UI visibility/draft inputs.
//!
//! Provenance: our own per-frame scene cache; no external code or vendor material.

use std::sync::{Arc, Mutex, Weak};

use camcraft_app::geo2d::Region;
use camcraft_app::geometry::Triangle;
use camcraft_app::model;
use camcraft_app::operation::{AreaControls, GeoSel, OpParams};
use flecs_ecs::prelude::Entity;

use crate::egui_app::world::Scene;
use crate::state::State;

const WORLD_VERTS_CAP: usize = 32;

/// Combined revision token for `state.world`, `state.selection_version`, and
/// `state.history` (catches both command runs and direct history execution in tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorldStamp {
    pub doc_version: u64,
    pub selection_version: u64,
    pub history: (u64, usize, u64, usize, u64),
}

impl WorldStamp {
    #[must_use]
    pub(crate) fn of(state: &State) -> Self {
        Self {
            doc_version: state.doc_version,
            selection_version: state.selection_version,
            history: state.history.revision_token(),
        }
    }
}

/// Entity-level guard verifying visibility, facet allocation pointer, revision
/// counter, and aligned setup pose without touching triangle buffers.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EntityGuard {
    name: String,
    hidden: bool,
    rev: u64,
    facets_ptr: usize,
    pose: [u64; 7],
}

impl EntityGuard {
    #[must_use]
    pub(crate) fn capture(state: &State, name: &str) -> Self {
        let hidden = state.hidden_in_view(name);
        if hidden {
            return Self {
                name: name.to_owned(),
                hidden: true,
                rev: 0,
                facets_ptr: 0,
                pose: [0; 7],
            };
        }
        let Some(entity) = model::find_by_name(&state.world, name) else {
            return Self {
                name: name.to_owned(),
                hidden: false,
                rev: 0,
                facets_ptr: 0,
                pose: [0; 7],
            };
        };
        let rev = model::rev_of(&state.world, &entity);
        let facets_ptr = model::facets_of(&state.world, &entity)
            .map_or(0, |a| Arc::as_ptr(&a).cast::<()>() as usize);
        Self {
            name: name.to_owned(),
            hidden: false,
            rev,
            facets_ptr,
            pose: pose_bits(state, &entity),
        }
    }

    #[must_use]
    pub(crate) fn matches(&self, state: &State) -> bool {
        let hidden = state.hidden_in_view(&self.name);
        if hidden != self.hidden {
            return false;
        }
        if hidden {
            return true;
        }
        let Some(entity) = model::find_by_name(&state.world, &self.name) else {
            return self.rev == 0 && self.facets_ptr == 0;
        };
        let rev = model::rev_of(&state.world, &entity);
        let facets_ptr = model::facets_of(&state.world, &entity)
            .map_or(0, |a| Arc::as_ptr(&a).cast::<()>() as usize);
        rev == self.rev && facets_ptr == self.facets_ptr && pose_bits(state, &entity) == self.pose
    }
}

#[must_use]
pub(crate) fn pose_bits(state: &State, entity: &Entity) -> [u64; 7] {
    let loc = camcraft_app::setup_align::view_pose(&state.world, entity).unwrap_or_default();
    [
        loc.pos.x.to_bits(),
        loc.pos.y.to_bits(),
        loc.pos.z.to_bits(),
        loc.rot[0].to_bits(),
        loc.rot[1].to_bits(),
        loc.rot[2].to_bits(),
        loc.rot[3].to_bits(),
    ]
}

#[must_use]
pub(crate) fn guards_for_picks(state: &State, picks: &[GeoSel]) -> Vec<EntityGuard> {
    let mut names: Vec<&str> = Vec::new();
    for g in picks {
        if !names.contains(&g.entity.as_str()) {
            names.push(&g.entity);
        }
    }
    names.into_iter().map(|n| EntityGuard::capture(state, n)).collect()
}

#[must_use]
pub(crate) fn selected_entity_names(state: &State) -> Vec<String> {
    model::named_entities(&state.world)
        .into_iter()
        .filter(|(_, e)| model::is_selected(&state.world, e))
        .map(|(n, _)| n)
        .collect()
}

#[must_use]
pub(crate) fn viewport_is_light() -> bool {
    crate::egui_app::ground_ink::light_ground(crate::egui_app::theme::current().viewport)
}

#[must_use]
pub(crate) fn cvd_code(state: &State) -> u8 {
    use camcraft_app::schema::ColorVision;
    match camcraft_app::commands::color_vision_of(&state.world) {
        ColorVision::Off => 0,
        ColorVision::Protan => 1,
        ColorVision::Deutan => 2,
        ColorVision::Tritan => 3,
    }
}

#[derive(Clone, PartialEq, Eq)]
struct RegionsKey {
    stamp: WorldStamp,
    selected: Vec<String>,
    hidden_paths: Vec<String>,
    light: bool,
}

#[derive(Clone, PartialEq, Eq)]
struct ToolpathLinesKey {
    stamp: WorldStamp,
    selected: Vec<String>,
    hidden_paths: Vec<String>,
    shown_setups: Vec<String>,
    show_all_paths: bool,
    arm_bits: u64,
    light: bool,
    cvd_idx: u8,
}

#[derive(Clone, PartialEq)]
pub(crate) struct SceneCacheKey {
    pub stamp: WorldStamp,
    pub selected: Vec<String>,
    pub ui_target_op: Option<String>,
    pub hidden_models: Vec<String>,
    pub hidden_stocks: Vec<String>,
    pub hidden_offsets: Vec<String>,
    pub hidden_paths: Vec<String>,
    pub shown_setups: Vec<String>,
    pub active_setup: Option<String>,
    pub incoming_key: (bool, Option<String>, u64),
    pub show_world_axes: bool,
    pub show_all_paths: bool,
    pub show_work_envelope: bool,
    pub show_gizmo_names: bool,
    pub cam_dist_bits: u64,
    pub cam_target_bits: [u64; 3],
    pub wcs_pick_anchor: bool,
    pub wcs_anchor_xy: usize,
    pub wcs_anchor_z: usize,
    pub wcs_anchor_source: usize,
    pub op_name: String,
    pub op_params: OpParams,
    pub op_area: AreaControls,
    pub op_src_kind: usize,
    pub op_src_name: String,
    pub op_geo: Vec<GeoSel>,
    pub viewport_rgba: [u8; 4],
    pub cvd_idx: u8,
}

struct WorldVertsEntry {
    facets: Weak<[Triangle]>,
    facets_len: usize,
    pose: [u64; 7],
    verts: Arc<Vec<[f32; 3]>>,
}

pub(crate) type RegionsArc = Arc<Vec<(f64, Option<Region>)>>;
pub(crate) type LinesArc = Arc<Vec<[f32; 6]>>;

struct RegionsCached {
    key: RegionsKey,
    regions: RegionsArc,
    lines: LinesArc,
}

struct SceneCached {
    key: SceneCacheKey,
    scene: Arc<Scene>,
    digest: u64,
}

#[derive(Default)]
struct FrameCacheInner {
    world_verts: Vec<WorldVertsEntry>,
    regions: Option<RegionsCached>,
    toolpath_lines: Option<(ToolpathLinesKey, Arc<Vec<[f32; 6]>>)>,
    scene: Option<SceneCached>,
}

/// Interior-mutable scene cache held on `State`.
#[derive(Default)]
pub(crate) struct FrameCache {
    inner: Mutex<FrameCacheInner>,
}

fn lock_inner(cache: &FrameCache) -> std::sync::MutexGuard<'_, FrameCacheInner> {
    cache
        .inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl FrameCache {
    pub(crate) fn world_verts_cached(
        &self,
        facets: &Arc<[Triangle]>,
        pose: [u64; 7],
        build: impl FnOnce() -> Vec<[f32; 3]>,
    ) -> Arc<Vec<[f32; 3]>> {
        let matches = |e: &WorldVertsEntry| {
            e.facets.strong_count() > 0
                && std::ptr::eq(e.facets.as_ptr(), Arc::as_ptr(facets))
                && e.facets_len == facets.len()
                && e.pose == pose
        };
        {
            let mut inner = lock_inner(self);
            inner.world_verts.retain(|e| e.facets.strong_count() > 0);
            if let Some(i) = inner.world_verts.iter().position(matches) {
                inner.world_verts[i..].rotate_left(1);
                if let Some(entry) = inner.world_verts.last() {
                    return Arc::clone(&entry.verts);
                }
            }
        }
        let verts = Arc::new(build());
        let mut inner = lock_inner(self);
        inner.world_verts.retain(|e| e.facets.strong_count() > 0);
        if let Some(i) = inner.world_verts.iter().position(matches) {
            return Arc::clone(&inner.world_verts[i].verts);
        }
        if inner.world_verts.len() >= WORLD_VERTS_CAP {
            inner.world_verts.remove(0);
        }
        inner.world_verts.push(WorldVertsEntry {
            facets: Arc::downgrade(facets),
            facets_len: facets.len(),
            pose,
            verts: Arc::clone(&verts),
        });
        verts
    }

    pub(crate) fn regions_and_lines(
        &self,
        state: &State,
        selected: Vec<String>,
        light: bool,
        build: impl FnOnce() -> (Vec<(f64, Option<Region>)>, Vec<[f32; 6]>),
    ) -> (RegionsArc, LinesArc) {
        let key = RegionsKey {
            stamp: WorldStamp::of(state),
            selected,
            hidden_paths: state.hidden_paths.clone(),
            light,
        };
        {
            let inner = lock_inner(self);
            if let Some(c) = &inner.regions
                && c.key == key
            {
                return (Arc::clone(&c.regions), Arc::clone(&c.lines));
            }
        }
        let (regs, lines) = build();
        let regions = Arc::new(regs);
        let lines = Arc::new(lines);
        let mut inner = lock_inner(self);
        inner.regions = Some(RegionsCached {
            key,
            regions: Arc::clone(&regions),
            lines: Arc::clone(&lines),
        });
        (regions, lines)
    }

    pub(crate) fn toolpath_lines_cached(
        &self,
        state: &State,
        selected: Vec<String>,
        arm_bits: u64,
        build: impl FnOnce() -> Vec<[f32; 6]>,
    ) -> Arc<Vec<[f32; 6]>> {
        let key = ToolpathLinesKey {
            stamp: WorldStamp::of(state),
            selected,
            hidden_paths: state.hidden_paths.clone(),
            shown_setups: state.shown_setups.clone(),
            show_all_paths: state.show_all_paths,
            arm_bits,
            light: viewport_is_light(),
            cvd_idx: cvd_code(state),
        };
        {
            let inner = lock_inner(self);
            if let Some((k, lines)) = &inner.toolpath_lines
                && *k == key
            {
                return Arc::clone(lines);
            }
        }
        let lines = Arc::new(build());
        let mut inner = lock_inner(self);
        inner.toolpath_lines = Some((key, Arc::clone(&lines)));
        lines
    }

    pub(crate) fn scene_cached(
        &self,
        key: &SceneCacheKey,
        build: impl FnOnce() -> (Scene, u64),
    ) -> (Arc<Scene>, u64) {
        {
            let inner = lock_inner(self);
            if let Some(c) = &inner.scene
                && c.key == *key
            {
                return (Arc::clone(&c.scene), c.digest);
            }
        }
        let (scene, digest) = build();
        let scene = Arc::new(scene);
        let mut inner = lock_inner(self);
        inner.scene = Some(SceneCached {
            key: key.clone(),
            scene: Arc::clone(&scene),
            digest,
        });
        (scene, digest)
    }
}

#[cfg(test)]
#[path = "frame_cache_tests.rs"]
mod tests;
