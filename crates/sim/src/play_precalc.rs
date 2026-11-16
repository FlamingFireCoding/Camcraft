//! The calculation before playback (#2181): cut the whole program once,
//! stride by stride, keeping keyframes (`play_keys`), with a stop, a
//! cancel and a progress report. Provenance: a plain cooperative work
//! loop over our own playback cut; nothing from vendor binaries.
//!
//! ## The laws here
//!
//! - **Display only.** The loop drives the playback picture's cut and
//!   nothing else; the verdict and the post gate never read it.
//! - **Every end is said by name.** Each stride reports `Running`; the
//!   end reports `Ready`, or `Incomplete` with "cancelled", "stopped"
//!   or the cut's own refusal. No end is silent.
//! - **A cancel keeps playback; a stop or a refusal ends it.** After a
//!   cancel the keyframes made so far stay valid and the rest is cut
//!   live; a stop is the owner going away; a refusal is published.

use crate::play_keys::Precalc;

/// The playback cut, as the calculation drives it.
pub trait PrecalcCut {
    /// Moves in the whole program.
    fn total(&self) -> usize;
    /// Moves the calculation has cut so far.
    fn done(&self) -> usize;
    /// Back to the uncut stock, keyframes and budget set.
    fn begin(&mut self) -> Result<(), String>;
    /// Cut one stride and offer its keyframe; `Ok(true)` at the end.
    fn step(&mut self) -> Result<bool, String>;
    /// The finished store's reading, `secs` after the start.
    fn ready(&self, secs: f64) -> Precalc;
}

/// How the calculation ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrecalcEnd {
    /// Ready, or cancelled: playback goes on.
    Play,
    /// The owner stopped: playback ends, nothing to publish.
    Stopped,
    /// The cut refused: playback ends and the refusal is published.
    Refused(String),
}

/// The owner's side of the loop: what it asks and where it hears.
pub trait PrecalcOwner {
    /// The owner went away (the worker is dropped).
    fn stopped(&self) -> bool;
    /// The user cancelled the calculation.
    fn cancelled(&self) -> bool;
    /// Seconds since the start (the owner holds the clock).
    fn elapsed(&self) -> f64;
    /// The calculation's state, after each change.
    fn report(&self, p: Precalc);
}

/// Run the calculation to an end (the module law).
pub fn drive(cut: &mut impl PrecalcCut, owner: &impl PrecalcOwner) -> PrecalcEnd {
    let total = cut.total();
    if let Err(e) = cut.begin() {
        owner.report(Precalc::Incomplete { done: 0, total, why: e.clone() });
        return PrecalcEnd::Refused(e);
    }
    loop {
        if owner.stopped() {
            owner.report(Precalc::Incomplete { done: cut.done(), total, why: "stopped".into() });
            return PrecalcEnd::Stopped;
        }
        if owner.cancelled() {
            owner.report(Precalc::Incomplete { done: cut.done(), total, why: "cancelled".into() });
            return PrecalcEnd::Play;
        }
        match cut.step() {
            Ok(true) => {
                owner.report(cut.ready(owner.elapsed()));
                return PrecalcEnd::Play;
            }
            Ok(false) => owner.report(Precalc::Running { done: cut.done(), total }),
            Err(e) => {
                owner.report(Precalc::Incomplete { done: cut.done(), total, why: e.clone() });
                return PrecalcEnd::Refused(e);
            }
        }
    }
}
