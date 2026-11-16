//! The stock removal calculated before playback (#2181): the whole
//! program is cut once, keyframes of the cut stock are kept at a fixed
//! stride, and playback then replays from the nearest keyframe instead
//! of cutting the program as the playhead moves. Provenance: the M7.1
//! field engine's order-free subtraction (any prefix of the program is
//! cut exactly by its own moves, so a keyframe plus the rest of the
//! moves IS the live cut) and plain keyframe thinning (halve the
//! keyframes, double the stride) for the memory bound. Nothing from
//! vendor binaries.
//!
//! ## The laws here
//!
//! - **Display only.** The keyframes are the playback picture's
//!   checkpoints and nothing else reads them. The Deviate verdict and
//!   the checks that block the post are walked elsewhere, unchanged.
//! - **Memory is bounded by a budget, never by the program.** The
//!   keyframes' cut layers are summed as they are kept; past the budget
//!   every other keyframe goes and the stride doubles, so the store
//!   never holds more than the budget (at worst only the uncut stock).
//! - **Incomplete is said, never shown as done.** A cancelled or failed
//!   calculation is `Precalc::Incomplete` by name; the keyframes it made
//!   stay valid (each is an exact prefix) and the moves past them are
//!   cut live, as before #2181.

use super::FieldPlay;

/// The keyframe store's memory budget: 512 MiB of cut layers.
pub const PRECALC_BUDGET: usize = 512 << 20;

/// Keyframes at most: the stride is at least the program over this.
pub const MAX_KEYFRAMES: usize = 1024;

/// Where the calculation before playback stands.
#[derive(Clone, Debug, PartialEq)]
pub enum Precalc {
    /// Not asked: playback cuts as the playhead moves.
    Off,
    /// Cutting: `done` of `total` moves have keyframes behind them.
    Running { done: usize, total: usize },
    /// Every move is calculated: a seek cuts at most `stride` moves.
    Ready { keys: usize, bytes: usize, stride: usize, secs: f64 },
    /// Stopped at `done` of `total` moves, and why: past `done` the
    /// playback cuts live.
    Incomplete { done: usize, total: usize, why: String },
}

impl FieldPlay {
    /// The checkpoint stride, in moves.
    #[must_use]
    pub const fn stride(&self) -> usize {
        self.stride
    }

    /// Moves cut since the build (the replay law's yardstick: a seek
    /// after the calculation adds at most one stride).
    #[must_use]
    pub const fn moves_cut(&self) -> usize {
        self.moves_cut
    }

    /// Cut-layer bytes the keyframes hold (bounded by the budget).
    #[must_use]
    pub const fn keyframe_bytes(&self) -> usize {
        self.key_bytes
    }

    /// Start the calculation: back to the uncut stock, keyframes every
    /// `total / keys` moves (at least `min_stride`), the dirty ledger off
    /// (no picture is taken while it runs).
    pub fn begin_precalc(&mut self, keys: usize, min_stride: usize) -> Result<(), String> {
        self.drop_partial();
        self.checkpoints.truncate(1);
        let (_, uncut) = self.checkpoints.first().ok_or("playback lost its uncut stock")?;
        self.cur = uncut.clone();
        self.at = 0;
        self.key_bytes = 0;
        self.stride = self.total.div_ceil(keys.max(1)).max(min_stride.max(1));
        self.set_batched(true);
        self.restored = true;
        Ok(())
    }

    /// Cut one stride and keep its keyframe, thinning the store to
    /// `budget` bytes; `Ok(true)` when the whole program is calculated.
    pub fn precalc_step(&mut self, budget: usize) -> Result<bool, String> {
        if self.at >= self.total {
            return Ok(true);
        }
        let to = (self.at / self.stride).saturating_add(1).saturating_mul(self.stride).min(self.total);
        self.cut(self.at, to)?;
        self.at = to;
        self.key_bytes = self.key_bytes.saturating_add(cut_bytes(&self.cur));
        self.checkpoints.push((to, self.cur.clone()));
        while self.key_bytes > budget && self.checkpoints.len() > 1 {
            self.thin();
        }
        Ok(self.at >= self.total)
    }

    /// Halve the keyframes: double the stride and keep only those on it.
    fn thin(&mut self) {
        self.stride = self.stride.saturating_mul(2);
        let stride = self.stride.max(1);
        self.checkpoints.retain(|(n, _)| n % stride == 0);
        self.key_bytes = self.checkpoints.iter().map(|(_, s)| cut_bytes(s)).fold(0, usize::saturating_add);
    }
}

/// What one keyframe's clone copies (the mesh layer is shared).
fn cut_bytes(s: &crate::stock_sim::Stock) -> usize {
    s.field().map_or(0, crate::stock_sim::SdfField::cut_layer_bytes)
}
