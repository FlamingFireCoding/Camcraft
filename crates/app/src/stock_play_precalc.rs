//! `FieldPlay`'s side of the calculation before playback (#2181): the
//! accessors and the `PrecalcCut` seam that `camcraft_sim::play_precalc`
//! drives. The keyframe store, its budget and thinning, the `Precalc`
//! state and the loop are the sim crate's (`play_keys`,
//! `play_precalc`); only the methods of this app type stay here, since
//! `FieldPlay` itself is app's (#2192 review). Provenance: our own seam;
//! nothing from vendor binaries.

use super::FieldPlay;
use crate::stock_sim::Stock;
use camcraft_sim::play_keys::{Keyframes, Precalc, MAX_KEYFRAMES};
use camcraft_sim::play_precalc::PrecalcCut;

impl FieldPlay {
    /// The checkpoint stride, in moves.
    #[must_use]
    pub const fn stride(&self) -> usize {
        self.keys.stride()
    }

    /// Moves cut since the build (the replay law's yardstick: a seek
    /// after the calculation adds at most one stride).
    #[must_use]
    pub const fn moves_cut(&self) -> usize {
        self.moves_cut
    }

    /// Cut-layer bytes the keyframes hold (within the budget once a
    /// calculation set one, live seeks included).
    #[must_use]
    pub const fn keyframe_bytes(&self) -> usize {
        self.keys.bytes()
    }

    /// The checkpoint store (read only).
    #[must_use]
    pub const fn keyframes(&self) -> &Keyframes<Stock> {
        &self.keys
    }

    /// Start the calculation: back to the uncut stock, keyframes every
    /// `total / keys` moves (at least `min_stride`) within `budget`
    /// bytes from here on, the dirty ledger off (no picture is taken
    /// while it runs).
    pub fn begin_precalc(&mut self, keys: usize, min_stride: usize, budget: usize) -> Result<(), String> {
        self.drop_partial();
        let uncut = self.keys.floor(0).ok_or("playback lost its uncut stock")?.1.clone();
        let stride = self.total.div_ceil(keys.max(1)).max(min_stride.max(1));
        self.keys.restart(stride, keys.max(MAX_KEYFRAMES), budget);
        self.cur = uncut;
        self.at = 0;
        self.set_batched(true);
        self.restored = true;
        Ok(())
    }

    /// Cut one stride and offer its keyframe to the store (its budget
    /// thins it); `Ok(true)` when the whole program is calculated.
    pub fn precalc_step(&mut self) -> Result<bool, String> {
        if self.at >= self.total {
            return Ok(true);
        }
        let to = self.keys.next_boundary(self.at, self.total);
        self.cut(self.at, to)?;
        self.at = to;
        self.keys.offer(to, &self.cur);
        Ok(self.at >= self.total)
    }
}

/// The worker's calculation drives the playback through this seam.
pub(crate) struct PrecalcPlay<'a> {
    pub play: &'a mut FieldPlay,
    pub keys: usize,
    pub min_stride: usize,
    pub budget: usize,
}

impl PrecalcCut for PrecalcPlay<'_> {
    fn total(&self) -> usize {
        self.play.total()
    }

    fn done(&self) -> usize {
        self.play.at()
    }

    fn begin(&mut self) -> Result<(), String> {
        self.play.begin_precalc(self.keys, self.min_stride, self.budget)
    }

    fn step(&mut self) -> Result<bool, String> {
        self.play.precalc_step()
    }

    fn ready(&self, secs: f64) -> Precalc {
        Precalc::Ready { keys: self.play.checkpoints(), bytes: self.play.keyframe_bytes(), stride: self.play.stride(), secs }
    }
}
