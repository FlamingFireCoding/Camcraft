//! Where the passes of one level start (#3316): the order of a ring
//! family. One shared type for every kind with a pass order (#1448 adds
//! the start point beside it). The law is ours, from general machining
//! practice (cut from the open side so each pass engages one stepover);
//! no vendor material consulted.

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

    /// The walking rank of a ring loop: the loops of a level are cut in
    /// ascending rank (a stable sort, so equal ranks keep their listed
    /// order). `k` is the ring's offset index (0 is the area's own
    /// boundary), `kmax` the largest index of the level, `outer` true for
    /// a loop of the outer-boundary family (counter-clockwise) and false
    /// for a loop of a hole family (clockwise).
    ///
    /// - Auto: the core first (`kmax - k`), the planner's old order.
    /// - Outside: the outer family from its boundary in (`k`), then the
    ///   hole family from the core back to its wall. A ring then always
    ///   has cut air on one side: no pass is a full slot.
    /// - Inside: the mirror - the hole family from its wall out, then the
    ///   outer family from the core back to its boundary.
    #[must_use]
    pub fn rank(self, k: u32, kmax: u32, outer: bool) -> u64 {
        let k = u64::from(k.min(kmax));
        let kmax = u64::from(kmax);
        let back = kmax - k;
        match (self, outer) {
            (PassStart::Auto, _) => back,
            (PassStart::Outside, true) | (PassStart::Inside, false) => k,
            (PassStart::Outside, false) | (PassStart::Inside, true) => kmax + 1 + back,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PassStart;

    #[test]
    fn auto_is_core_first_and_the_two_sides_mirror() {
        assert!(PassStart::Auto.rank(3, 3, true) < PassStart::Auto.rank(0, 3, false));
        // Outside: outer family ascending, then holes from the core back.
        let o: Vec<u64> = [(0, true), (1, true), (3, false), (0, false)]
            .iter()
            .map(|&(k, f)| PassStart::Outside.rank(k, 3, f))
            .collect();
        assert!(o.windows(2).all(|w| w[0] < w[1]), "{o:?}");
        let i: Vec<u64> = [(0, false), (2, false), (3, true), (0, true)]
            .iter()
            .map(|&(k, f)| PassStart::Inside.rank(k, 3, f))
            .collect();
        assert!(i.windows(2).all(|w| w[0] < w[1]), "{i:?}");
        assert_eq!(PassStart::default(), PassStart::Auto);
    }
}
