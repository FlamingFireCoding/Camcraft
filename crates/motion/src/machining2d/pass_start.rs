//! Where the passes of one level start (#3316): the side of the area the
//! level begins on. One shared type for every kind with a pass order
//! (#1448 adds the start point beside it). The law is ours, from general
//! machining practice (cut from the open side so each pass engages one
//! stepover); no vendor material consulted.

use crate::geo2d::Region;

/// The first words of every note the start-side planner writes: the op
/// page shows each one as a warning line.
pub const PASS_START_NOTE: &str = "Pass start";

/// Which side of the cutter-center area a level starts on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PassStart {
    /// The order each kind always cut: for the ring planner the core
    /// first, then outward. Every file before the option, byte for byte.
    #[default]
    Auto,
    /// Start at the area's holes (an island wall, the part wall of an
    /// Outside band) and walk out to its outer boundary.
    Inside,
    /// Start at the area's outer boundary (a pocket wall, the stock edge
    /// of an Outside band) and walk in to its holes.
    Outside,
}

impl PassStart {
    /// Every value, in the order the page lists them.
    pub const ALL: [PassStart; 3] = [PassStart::Auto, PassStart::Inside, PassStart::Outside];

    /// The name the page and the notes use.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            PassStart::Auto => "Auto",
            PassStart::Inside => "Inside",
            PassStart::Outside => "Outside",
        }
    }
}

/// The start side and what the planner needs to judge it: where the
/// stock is, and what the area's holes are.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StartSide {
    pub start: PassStart,
    /// The stock's footprint in the area's frame, when it is known. A
    /// start ring whose disc stands off it runs in open air; without it
    /// every start ring is read as a wall in uncut stock.
    pub stock: Option<Region>,
    /// The area surrounds the part (an Outside band): its holes are the
    /// part, not islands.
    pub around_part: bool,
}

impl From<PassStart> for StartSide {
    fn from(start: PassStart) -> Self {
        StartSide { start, stock: None, around_part: false }
    }
}

impl StartSide {
    /// The warning for a level whose first ring runs along a wall in
    /// uncut stock, so that it cuts the full tool width (#3316): the
    /// outer wall of a closed pocket for Outside, the part or island
    /// wall for Inside. `None` for Auto.
    #[must_use]
    pub fn wall_first_note(&self) -> Option<String> {
        match (self.start, self.around_part) {
            (PassStart::Auto, _) => None,
            (PassStart::Outside, _) => Some(format!(
                "{PASS_START_NOTE} Outside: the outer edge of this area is a wall in the stock, not the open \
                 stock edge, so each level starts with a cut of the full tool width along that wall"
            )),
            (PassStart::Inside, true) => Some(format!(
                "{PASS_START_NOTE} Inside: the inner edge of this area is the part wall, so each level starts \
                 with a cut of the full tool width along the part"
            )),
            (PassStart::Inside, false) => Some(format!(
                "{PASS_START_NOTE} Inside: the inner edge of this area is an island wall, so each level starts \
                 with a cut of the full tool width along the island"
            )),
        }
    }

    /// The warning for cuts after the first ring that still take the
    /// full tool width (#3316): `n` of them, the first at `(x, y)`.
    #[must_use]
    pub fn narrow_note(&self, n: usize, x: f64, y: f64) -> String {
        format!(
            "{PASS_START_NOTE} {}: {n} cut(s) take the full tool width where the area is narrower than the \
             tool plus one stepover, or where a wall hides the start side; the first is at X {x:.3}, Y {y:.3}",
            self.start.label()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{PASS_START_NOTE, PassStart, StartSide};

    #[test]
    fn auto_says_nothing_and_each_wall_case_is_named() {
        assert_eq!(PassStart::default(), PassStart::Auto);
        assert_eq!(StartSide::from(PassStart::Auto).wall_first_note(), None);
        let out = StartSide::from(PassStart::Outside).wall_first_note().unwrap_or_default();
        assert!(out.starts_with(PASS_START_NOTE) && out.contains("outer edge"), "{out}");
        let part = StartSide { start: PassStart::Inside, stock: None, around_part: true };
        assert!(part.wall_first_note().is_some_and(|n| n.contains("part wall")));
        let island = StartSide::from(PassStart::Inside).wall_first_note().unwrap_or_default();
        assert!(island.contains("island wall"), "{island}");
        let narrow = StartSide::from(PassStart::Outside).narrow_note(2, 1.0, -2.5);
        assert!(narrow.starts_with("Pass start Outside: 2 cut(s)") && narrow.contains("X 1.000, Y -2.500"), "{narrow}");
    }
}
