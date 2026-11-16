//! The playback worker's mailbox for the calculation before playback
//! (#2181): the UI's progress read and cancel on the shared handle, and
//! the owner seam that `camcraft_sim::play_precalc::drive` reports to.
//! The loop, its ends and the keyframe store are the sim crate's; only
//! `PlayWorker`'s own state stays here (#2192 review). Provenance: the
//! worker's own latest-wins mailbox (`stock_play_worker`); nothing from
//! vendor binaries.
//!
//! A worker started with `spawn_precalc` first cuts the whole program
//! once, keeping keyframes; its progress is `precalc()` and the UI may
//! `cancel_precalc()`. Requests wait until it ends. A cancel or a stop
//! leaves `Precalc::Incomplete` by name, and the moves past the last
//! keyframe are cut live, within the same byte budget.

use std::sync::atomic::Ordering;

use super::{lock, publish, PlayWorker, Shared};
use crate::clock::Instant;
use crate::stock_play::{FieldPlay, PlaySeed, Precalc, PrecalcPlay, MAX_KEYFRAMES, MIN_STRIDE};
use camcraft_sim::play_precalc::{drive, PrecalcEnd, PrecalcOwner};

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
    /// keyframes held to `budget` bytes (the sim crate's law).
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

/// The worker as the calculation's owner.
struct Owner<'a> {
    shared: &'a Shared,
    t0: Instant,
}

impl PrecalcOwner for Owner<'_> {
    fn stopped(&self) -> bool {
        lock(&self.shared.req).is_none_or(|g| g.stop)
    }

    fn cancelled(&self) -> bool {
        self.shared.cancel.load(Ordering::Relaxed)
    }

    fn elapsed(&self) -> f64 {
        self.t0.elapsed().as_secs_f64()
    }

    fn report(&self, p: Precalc) {
        if matches!(p, Precalc::Ready { .. }) {
            tracing::info!("playback precalculated: {p:?}");
        }
        set_precalc(self.shared, p);
    }
}

/// Run the calculation on the worker: `false` when the worker must end
/// (stopped, or a cut refused and published).
pub(super) fn precalc(shared: &Shared, play: &mut FieldPlay, budget: usize) -> bool {
    let owner = Owner { shared, t0: Instant::now() };
    let end = drive(&mut PrecalcPlay { play: &mut *play, keys: MAX_KEYFRAMES, min_stride: MIN_STRIDE, budget }, &owner);
    match end {
        PrecalcEnd::Play => {
            // The ledger starts fresh from the next (whole) picture.
            play.set_batched(false);
            true
        }
        PrecalcEnd::Stopped => false,
        PrecalcEnd::Refused(e) => {
            publish(shared, Err(e));
            false
        }
    }
}
