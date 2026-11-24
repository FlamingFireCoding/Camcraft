//! The work qualities that a zero-step piece can request, from the MIT Agie Processor quality
//! presets and our offline Agie Control checks (#2954, #2984, #2966).

use super::QN2_OFFSETS_UM;

/// The one quality object of a piece. Its name identifies the requested
/// values, so that no generic name binds other resident data (#2984).
/// The control preselects the trims for the quality; the piece arms nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Quality {
    /// Fine Ra 0.30 µm, Te 1.50 µm, Tkm 1.00 µm, at every height in
    /// [`QN2_OFFSETS_UM`]. Every punch, die, group and cutoff uses it.
    #[default]
    Ccb030,
    /// Fine+ Ra 0.35 µm, Te 1.50 µm, Tkm 1.00 µm, for one closed punch on a
    /// 20 mm plate only (#2966). See [`CCB035_OFFSETS_UM`].
    Ccb035,
}

/// Wire-centre offsets in µm of the rough pass and the three trims on a
/// straight outside face for quality CCB035. The control displayed Ra 0.35,
/// Te 1.50 and Tkm 1.00 on a 20 mm Berco Cut 0.25 punch in steel 0001 and
/// plotted these offsets after TECCUT (#2966). No other height is measured.
pub const CCB035_OFFSETS_UM: [(u16, [u16; 4]); 1] = [(20, [245, 155, 140, 134])];

impl Quality {
    /// The native quality name that the piece creates.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ccb030 => "CCB030",
            Self::Ccb035 => "CCB035",
        }
    }

    /// The requested Ra in µm, as the control displays it.
    #[must_use]
    pub fn ra(self) -> &'static str {
        match self {
            Self::Ccb030 => "0.30",
            Self::Ccb035 => "0.35",
        }
    }

    /// The six `JE_CreateQuality` values: Ra, Te and Tkm in µm and three options.
    pub(super) fn request(self) -> String {
        format!("{},1.50,1.00,F,F,F", self.ra())
    }

    /// The measured stock thicknesses in mm and their offsets in µm.
    #[must_use]
    pub fn offsets_table(self) -> &'static [(u16, [u16; 4])] {
        match self {
            Self::Ccb030 => &QN2_OFFSETS_UM,
            Self::Ccb035 => &CCB035_OFFSETS_UM,
        }
    }

    /// The measured offsets for a stock thickness in µm, if it is qualified.
    #[must_use]
    pub fn offsets_um(self, height_um: i32) -> Option<[u16; 4]> {
        self.offsets_table()
            .iter()
            .find(|(mm, _)| i32::from(*mm) * 1000 == height_um)
            .map(|(_, offsets)| *offsets)
    }
}
