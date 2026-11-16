//! The playback worker's calculation before playback (#2181): drives
//! `FieldPlay::precalc_step` stride by stride on the worker thread,
//! with its state and the UI's cancel on the shared handle. Provenance:
//! the worker's own latest-wins mailbox (`stock_play_worker`); nothing
//! from vendor binaries.
//!
//! A worker started with `spawn_precalc` first cuts the whole program
//! once, keeping keyframes (`stock_play_precalc`); its progress is
//! `precalc()` and the UI may `cancel_precalc()`. Requests wait until
//! it ends. A cancel or a stop leaves `Precalc::Incomplete` by name,
//! and the moves past the last keyframe are cut live.

use std::sync::atomic::Ordering;

use super::{lock, publish, PlayWorker, Shared};
use crate::clock::Instant;
use crate::stock_play::{FieldPlay, PlaySeed, Precalc, MAX_KEYFRAMES, MIN_STRIDE};

impl PlayWorker {
    /// Where the calculation before playback stands (`Off` when the
    /// worker was spawned without one; a lost lock reads incomplete).
    #[must_use]
    pub fn precalc(&self) -> Precalc {
        match lock(&self.shared.precalc) {
            Some(g) => g.clone().unwrap_or(Precalc::Off),
            None => Precalc::Incomplete { done: 0, total: self.index.total(), why: "the calculation lost its state".into() },
        }
    }

    /// Stop the calculation where it stands; playback goes on, cutting
    /// live past the last keyframe.
    pub fn cancel_precalc(&self) {
        self.shared.cancel.store(true, Ordering::Relaxed);
        self.shared.wake.notify_all();
    }

    /// `spawn`, but the worker first calculates the whole removal with
    /// keyframes held to `budget` bytes (the module law).
    pub fn spawn_precalc(seed: PlaySeed, budget: usize) -> Result<Self, String> {
        Self::start(seed, Some(budget))
    }
}

/// A playback that would not build: with a calculation asked, that is
/// an incomplete calculation by name; then the refusal is published.
pub(super) fn refuse(shared: &Shared, budget: Option<usize>, e: String) {
    if budget.is_some() {
        set_precalc(shared, Precalc::Incomplete { done: 0, total: 0, why: e.clone() });
    }
    publish(shared, Err(e));
}

pub(super) fn set_precalc(shared: &Shared, p: Precalc) {
    if let Some(mut g) = lock(&shared.precalc) {
        *g = Some(p);
    }
}

/// The calculation before playback (the module law): `false` when the
/// worker must end (stopped, or a cut refused and published).
pub(super) fn precalc(shared: &Shared, play: &mut FieldPlay, budget: usize) -> bool {
    let total = play.total();
    let t0 = Instant::now();
    if let Err(e) = play.begin_precalc(MAX_KEYFRAMES, MIN_STRIDE) {
        set_precalc(shared, Precalc::Incomplete { done: 0, total, why: e.clone() });
        publish(shared, Err(e));
        return false;
    }
    loop {
        if lock(&shared.req).is_none_or(|g| g.stop) {
            set_precalc(shared, Precalc::Incomplete { done: play.at(), total, why: "stopped".into() });
            return false;
        }
        if shared.cancel.load(Ordering::Relaxed) {
            set_precalc(shared, Precalc::Incomplete { done: play.at(), total, why: "cancelled".into() });
            break;
        }
        match play.precalc_step(budget) {
            Ok(true) => {
                let p = Precalc::Ready { keys: play.checkpoints(), bytes: play.keyframe_bytes(), stride: play.stride(), secs: t0.elapsed().as_secs_f64() };
                tracing::info!("playback precalculated: {p:?}");
                set_precalc(shared, p);
                break;
            }
            Ok(false) => set_precalc(shared, Precalc::Running { done: play.at(), total }),
            Err(e) => {
                set_precalc(shared, Precalc::Incomplete { done: play.at(), total, why: e.clone() });
                publish(shared, Err(e));
                return false;
            }
        }
    }
    // The ledger starts fresh from the next (whole) picture.
    play.set_batched(false);
    true
}
