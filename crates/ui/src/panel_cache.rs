//! Per-`State` caches for dock panel queries: Geometry tab chips, 3-2-1 datum
//! check, selection account, operation status banner, and browser op rows.
//!
//! Provenance: our own per-frame panel query cache; no external code or vendor material.

use std::collections::HashMap;
use std::sync::Mutex;

use camcraft_app::operation::{GeoSel, OpParams, OpStatus};

use crate::frame_cache::{EntityGuard, WorldStamp, cvd_code, guards_for_picks};
use crate::state::{GeoChip, State};

#[derive(Clone, PartialEq)]
struct GeoChipsKey {
    stamp: WorldStamp,
    unit_idx: u8,
    geo: Vec<GeoSel>,
    guards: Vec<EntityGuard>,
}

#[derive(Clone, PartialEq)]
struct AlignKey {
    stamp: WorldStamp,
    geo: Vec<GeoSel>,
    guards: Vec<EntityGuard>,
}

#[derive(Clone, PartialEq)]
struct SelectionAccountKey {
    stamp: WorldStamp,
    live_setup: Option<String>,
    geo: Vec<GeoSel>,
    params: OpParams,
    guards: Vec<EntityGuard>,
}

#[derive(Clone, PartialEq)]
struct UnresolvedNoteKey {
    stamp: WorldStamp,
    live_setup: Option<String>,
    src_kind: usize,
    src_name: String,
    geo: Vec<GeoSel>,
    params: OpParams,
}

#[derive(Clone, PartialEq, Eq)]
struct OpStatusKey {
    stamp: WorldStamp,
    target: String,
    cvd_idx: u8,
}

#[derive(Clone)]
pub(crate) struct OpBannerCached {
    pub line: String,
    pub tone: u8,
    pub details: Vec<String>,
    pub focus: Vec<Option<[f64; 3]>>,
}

#[derive(Clone)]
pub(crate) struct BrowserOpCached {
    pub kind_label: String,
    pub kind_type: Option<camcraft_app::operation::OpType>,
    pub status: OpStatus,
    pub warns: usize,
    pub flow: String,
    pub problem: Option<String>,
    pub label: String,
}

type AccountValue = (Vec<String>, Option<String>);

#[derive(Default)]
struct PanelCacheInner {
    geo_chips: Option<(GeoChipsKey, Vec<GeoChip>)>,
    align_321: Option<(AlignKey, Result<(), String>)>,
    selection_account: Option<(SelectionAccountKey, AccountValue)>,
    unresolved_note: Option<(UnresolvedNoteKey, Option<String>)>,
    op_banner: Option<(OpStatusKey, OpBannerCached)>,
    browser_ops: Option<(WorldStamp, HashMap<String, BrowserOpCached>)>,
}

/// Interior-mutable panel cache stored on `State`.
#[derive(Default)]
pub(crate) struct PanelCache {
    inner: Mutex<PanelCacheInner>,
}

fn lock_inner(cache: &PanelCache) -> std::sync::MutexGuard<'_, PanelCacheInner> {
    cache
        .inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn unit_code(state: &State) -> u8 {
    match camcraft_app::units::of(&state.world) {
        camcraft_app::geometry::units::Unit::Millimeters => 0,
        camcraft_app::geometry::units::Unit::Inches => 1,
    }
}

impl PanelCache {
    pub(crate) fn geo_chips_cached(
        &self,
        state: &State,
        build: impl FnOnce() -> Vec<GeoChip>,
    ) -> Vec<GeoChip> {
        let stamp = WorldStamp::of(state);
        let unit = unit_code(state);
        {
            let inner = lock_inner(self);
            if let Some((k, chips)) = &inner.geo_chips
                && k.stamp == stamp
                && k.unit_idx == unit
                && k.geo == state.op_ui.geo
                && k.guards.iter().all(|g| g.matches(state))
            {
                return chips.clone();
            }
        }
        let chips = build();
        let key = GeoChipsKey {
            stamp,
            unit_idx: unit,
            geo: state.op_ui.geo.clone(),
            guards: guards_for_picks(state, &state.op_ui.geo),
        };
        let mut inner = lock_inner(self);
        inner.geo_chips = Some((key, chips.clone()));
        chips
    }

    pub(crate) fn align_321_cached(
        &self,
        state: &State,
        build: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        let stamp = WorldStamp::of(state);
        {
            let inner = lock_inner(self);
            if let Some((k, res)) = &inner.align_321
                && k.stamp == stamp
                && k.geo == state.op_ui.geo
                && k.guards.iter().all(|g| g.matches(state))
            {
                return res.clone();
            }
        }
        let res = build();
        let key = AlignKey {
            stamp,
            geo: state.op_ui.geo.clone(),
            guards: guards_for_picks(state, &state.op_ui.geo),
        };
        let mut inner = lock_inner(self);
        inner.align_321 = Some((key, res.clone()));
        res
    }

    pub(crate) fn selection_account_cached(
        &self,
        state: &State,
        live_setup: Option<&str>,
        build: impl FnOnce() -> (Vec<String>, Option<String>),
    ) -> (Vec<String>, Option<String>) {
        let stamp = WorldStamp::of(state);
        {
            let inner = lock_inner(self);
            if let Some((k, val)) = &inner.selection_account
                && k.stamp == stamp
                && k.live_setup.as_deref() == live_setup
                && k.geo == state.op_ui.geo
                && k.params == state.op_ui.params
                && k.guards.iter().all(|g| g.matches(state))
            {
                return val.clone();
            }
        }
        let val = build();
        let key = SelectionAccountKey {
            stamp,
            live_setup: live_setup.map(str::to_owned),
            geo: state.op_ui.geo.clone(),
            params: state.op_ui.params.clone(),
            guards: guards_for_picks(state, &state.op_ui.geo),
        };
        let mut inner = lock_inner(self);
        inner.selection_account = Some((key, val.clone()));
        val
    }

    pub(crate) fn unresolved_note_cached(
        &self,
        state: &State,
        live_setup: Option<&str>,
        build: impl FnOnce() -> Option<String>,
    ) -> Option<String> {
        let stamp = WorldStamp::of(state);
        {
            let inner = lock_inner(self);
            if let Some((k, note)) = &inner.unresolved_note
                && k.stamp == stamp
                && k.live_setup.as_deref() == live_setup
                && k.src_kind == state.op_ui.src_kind
                && k.src_name == state.op_ui.src_name
                && k.geo == state.op_ui.geo
                && k.params == state.op_ui.params
            {
                return note.clone();
            }
        }
        let note = build();
        let key = UnresolvedNoteKey {
            stamp,
            live_setup: live_setup.map(str::to_owned),
            src_kind: state.op_ui.src_kind,
            src_name: state.op_ui.src_name.clone(),
            geo: state.op_ui.geo.clone(),
            params: state.op_ui.params.clone(),
        };
        let mut inner = lock_inner(self);
        inner.unresolved_note = Some((key, note.clone()));
        note
    }

    pub(crate) fn op_banner_cached(
        &self,
        state: &State,
        target: &str,
        build: impl FnOnce() -> OpBannerCached,
    ) -> OpBannerCached {
        let key = OpStatusKey {
            stamp: WorldStamp::of(state),
            target: target.to_owned(),
            cvd_idx: cvd_code(state),
        };
        {
            let inner = lock_inner(self);
            if let Some((k, val)) = &inner.op_banner
                && *k == key
            {
                return val.clone();
            }
        }
        let val = build();
        let mut inner = lock_inner(self);
        inner.op_banner = Some((key, val.clone()));
        val
    }

    pub(crate) fn browser_op_cached(
        &self,
        state: &State,
        name: &str,
        build: impl FnOnce() -> BrowserOpCached,
    ) -> BrowserOpCached {
        let stamp = WorldStamp::of(state);
        {
            let inner = lock_inner(self);
            if let Some((cached_stamp, map)) = &inner.browser_ops
                && *cached_stamp == stamp
                && let Some(entry) = map.get(name)
            {
                return entry.clone();
            }
        }
        let entry = build();
        let mut inner = lock_inner(self);
        if inner.browser_ops.as_ref().is_some_and(|(s, _)| *s != stamp) {
            inner.browser_ops = None;
        }
        let (_, map) = inner
            .browser_ops
            .get_or_insert_with(|| (stamp, HashMap::new()));
        map.insert(name.to_owned(), entry.clone());
        entry
    }
}
