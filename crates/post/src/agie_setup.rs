//! Setup-only native Classic Cut 2S records, from Quintin Smits's MIT Agie Processor and our offline control checks.
//!
//! This encoder references an existing ISO geometry file for one closed punch
//! ([`PunchSetup20::encode`]) or one convex die opening on a 20 mm plate
//! ([`PunchSetup20::encode_die`]). The two differ only in the work side.
//! [`CutoffSetup20::encode`] writes one straight open cut on a 20 mm plate,
//! with a left or right side and two air endpoints instead of a start hole.
//! The caller must verify the nominal contour, its lead from the threading hole,
//! and the compensated cuts against the stock, installed travel and fixtures.
//! The piece uses stock top Z0, steel 0001, Berco Cut 0.25 and a 1 mm separation.
//! The stock thickness must be one of the measured QN2 heights in [`QN2_OFFSETS_UM`].
//!
//! Place the SBL script and SBR resource manifest beside the ISO, with the same
//! stem and their respective extensions. Import the ISO geometry into ISO Sys.
//! Keep the matching SBR beside the SBL when importing the script into SBL Sys.
//! Create a new workpiece from the imported script. Earlier pieces can retain
//! their erosion steps on repeat import. In the new piece, match the work's ISO
//! geometry and start hole to the setup and confirm zero steps. Follow
//! [`SetupFiles::controller_review`] before assigning TECCUT.
//!
//! ```
//! use camcraft_post::agie_setup::PunchSetup20;
//!
//! let setup = PunchSetup20 {
//!     stem: "CCSEP20",
//!     stock_min_um: [-40_000, -30_000],
//!     stock_max_um: [40_000, 30_000],
//!     start_hole_um: [-30_000, 0],
//!     height_um: 20_000,
//! };
//! let files = setup.encode()?;
//! assert_eq!(files.sbr, "AGIE.USING_CCSEP20.ISO IMPORT CCSEP20.ISO;\r\n");
//! assert!(files.sbl.contains("ASDELETE"));
//! # Ok::<(), camcraft_post::agie_setup::SetupError>(())
//! ```

mod inspection;
mod publish;
pub use inspection::{INSPECTION_RECEIPT, InspectionProfile, InspectionSegment};

/// Wire-centre offsets in µm of the rough pass and the three trims on a
/// straight outside face, for each measured stock thickness in mm. Offline
/// Agie Control plots of steel 0001, Berco Cut 0.25 and quality CCB030 (#2954).
/// No other thickness has a measurement, and none is interpolated.
pub const QN2_OFFSETS_UM: [(u16, [u16; 4]); 4] = [
    (15, [245, 155, 140, 134]),
    (20, [245, 155, 140, 134]),
    (40, [260, 170, 140, 130]),
    (80, [266, 176, 143, 133]),
];

/// The measured QN2 offsets for a stock thickness in µm, if it is qualified.
pub fn qn2_offsets_um(height_um: i32) -> Option<[u16; 4]> {
    QN2_OFFSETS_UM
        .iter()
        .find(|(mm, _)| i32::from(*mm) * 1000 == height_um)
        .map(|(_, offsets)| *offsets)
}

/// Setup coordinates for a closed punch or die opening with a predrilled 2 mm start hole.
/// XY coordinates are integer micrometres in the same work frame as the ISO.
#[derive(Debug)]
pub struct PunchSetup20<'a> {
    /// Common DOS file stem for the ISO, SBR and SBL (for example, `CCSEP20`).
    pub stem: &'a str,
    /// Lower stock limits [X, Y].
    pub stock_min_um: [i32; 2],
    /// Upper stock limits [X, Y].
    pub stock_max_um: [i32; 2],
    /// Centre of the threading hole [X, Y].
    pub start_hole_um: [i32; 2],
    /// Stock thickness below the stock top Z0. Must be in [`QN2_OFFSETS_UM`].
    pub height_um: i32,
}

/// The two native setup files. Save their ASCII bytes with the supplied CRLF endings.
#[derive(Debug)]
pub struct SetupFiles {
    /// Piece script for the control.
    pub sbl: String,
    /// Import manifest referring to the caller's ISO geometry.
    pub sbr: String,
    /// Operator checks for the installed effective work technology.
    pub controller_review: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SetupError {
    #[error(
        "Use a DOS stem of 1 to 8 uppercase letters or digits beginning with a letter. Choose a name other than a DOS device."
    )]
    InvalidStem,
    #[error(
        "Use increasing stock bounds. The stock and work origin must fit the 350 × 250 mm XY span."
    )]
    InvalidStock,
    #[error("Place the complete 2 mm start hole inside the stock.")]
    HoleOutsideStock,
    #[error(
        "Use a stock thickness of 15, 20, 40 or 80 mm. Other thicknesses have no measured QN2 offsets."
    )]
    UnqualifiedHeight,
    #[error(
        "Use a 20 mm plate for a die opening. The control checked the die cycle at 20 mm only."
    )]
    UnqualifiedDieHeight,
    #[error(
        "Use a 20 mm plate for an open cutoff. The control checked the open cut at 20 mm only."
    )]
    UnqualifiedCutoffHeight,
    #[error(
        "Place the entry and exit points in air beyond opposite faces of the stock and inside the 350 × 250 mm XY span."
    )]
    CutoffEndpoints,
    #[error(
        "Use a 20 mm plate for a punch group. The control checked the punch group at 20 mm only."
    )]
    UnqualifiedGroupHeight,
    #[error(
        "Use two punch works with distinct ISO stems. The control checked a group of two punches only."
    )]
    GroupWorks,
}

/// The stock thickness of the control-checked die cycle (#2957).
pub const DIE_HEIGHT_UM: i32 = 20_000;

/// The native work side. The control applies the offset toward the waste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkSide {
    Punch,
    Die,
    Left,
    Right,
}

impl WorkSide {
    fn native(self) -> &'static str {
        match self {
            Self::Punch => "c_punch",
            Self::Die => "c_die",
            Self::Left => "c_left",
            Self::Right => "c_right",
        }
    }
}

mod cutoff;
mod group;
mod piece;
pub use cutoff::{CUTOFF_HEIGHT_UM, CutoffSetup20, CutoffSide};
pub use group::{GROUP_HEIGHT_UM, GroupPunch, PunchGroup20};
use piece::{attribute_call, checked};

impl PunchSetup20<'_> {
    /// Encode one native work with zero erosion steps on import.
    ///
    /// # Errors
    /// Refuses invalid file stems, stock bounds, threading-hole positions and
    /// stock thicknesses without measured offsets.
    pub fn encode(&self) -> Result<SetupFiles, SetupError> {
        self.encode_side(WorkSide::Punch)
    }

    /// Encode one native die opening (`ID_PUNCH,c_die`) with zero erosion
    /// steps on import. The control offsets a die toward the inside of the
    /// opening. The die cycle is qualified for a 20 mm plate only.
    ///
    /// # Errors
    /// The errors of [`Self::encode`], and any thickness other than 20 mm.
    pub fn encode_die(&self) -> Result<SetupFiles, SetupError> {
        if self.height_um != DIE_HEIGHT_UM {
            return Err(SetupError::UnqualifiedDieHeight);
        }
        self.encode_side(WorkSide::Die)
    }

    fn encode_side(&self, side: WorkSide) -> Result<SetupFiles, SetupError> {
        if !valid_stem(self.stem) {
            return Err(SetupError::InvalidStem);
        }
        let [rough, trim1, trim2, trim3] =
            qn2_offsets_um(self.height_um).ok_or(SetupError::UnqualifiedHeight)?;
        for (axis, stroke_um) in [350_000, 250_000].into_iter().enumerate() {
            let min = i64::from(self.stock_min_um[axis]);
            let max = i64::from(self.stock_max_um[axis]);
            let hole = i64::from(self.start_hole_um[axis]);
            if min >= max || max.max(0) - min.min(0) > stroke_um {
                return Err(SetupError::InvalidStock);
            }
            if hole - 1000 < min || hole + 1000 > max {
                return Err(SetupError::HoleOutsideStock);
            }
        }
        // The punch text is unchanged. A die adds the side and the slug check.
        let die_check = match side {
            WorkSide::Die => concat!(
                " Confirm the c_die side and that each offset is inside the opening.",
                " Support the slug before the control releases the separation section.",
            ),
            _ => "",
        };
        let piece = piece::Piece {
            stem: self.stem,
            stock_min_um: self.stock_min_um,
            stock_max_um: self.stock_max_um,
            height_um: self.height_um,
        };
        let work = piece::Work {
            geometry: self.stem,
            side: side.native(),
            start_um: self.start_hole_um,
            exit_um: None,
        };
        Ok(setup_files(
            &piece,
            &work,
            "geometry and start hole",
            [rough, trim1, trim2, trim3],
            die_check,
        ))
    }
}

/// The SBL, the SBR and the operator review of one zero-step work.
fn setup_files(
    piece: &piece::Piece<'_>,
    work: &piece::Work<'_>,
    points: &str,
    [rough, trim1, trim2, trim3]: [u16; 4],
    side_check: &str,
) -> SetupFiles {
    SetupFiles {
        sbl: piece::piece(piece, std::slice::from_ref(work)),
        sbr: resources(std::slice::from_ref(work)),
        controller_review: format!(
            concat!(
                "Import the ISO into ISO Sys and keep its SBR beside the SBL during script import. ",
                "Select the newly created piece. Earlier pieces can retain their steps on repeat import. ",
                "Match this work's ISO resource, {points} to the setup. Confirm zero steps. ",
                "Verify steel 0001, Berco Cut 0.25 and {height} mm. CCB030 requests Ra 0.30, Te 1.50, ",
                "Tkm 1.00 and three disabled options. Check the effective values and both visible options ",
                "on this work. Assign the matching TECCUT row. Check offsets of {rough}, {trim1}, {trim2} and ",
                "{trim3} µm in order before GRAFICHECK and FORMALCHECK.{side_check}",
            ),
            points = points,
            height = piece.height_um / 1000,
            rough = rough,
            trim1 = trim1,
            trim2 = trim2,
            trim3 = trim3,
            side_check = side_check,
        ),
    }
}

/// The SBR import manifest: one ISO resource line for each work.
fn resources(works: &[piece::Work<'_>]) -> String {
    works
        .iter()
        .map(|work| format!("AGIE.USING_{0}.ISO IMPORT {0}.ISO;\r\n", work.geometry))
        .collect()
}

/// Whether the stem can name the matching ISO, SBR and SBL on Agie Control.
pub fn valid_stem(stem: &str) -> bool {
    let bytes = stem.as_bytes();
    (1..=8).contains(&bytes.len())
        && bytes[0].is_ascii_uppercase()
        && bytes
            .iter()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && !matches!(stem, "CON" | "AUX" | "NUL" | "PRN")
        && !(bytes.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && bytes[3].is_ascii_digit())
}
