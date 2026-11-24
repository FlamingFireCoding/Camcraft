//! The operation-level file mirror of [`crate::path_picks::PathPicks`]
//! (#3339): `picks: { start: [[x, y, z?], ...], exit: [...] }`, written
//! only when a role list is not empty so files without picks stay unchanged.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::geo2d::Point2;
use crate::path_picks::{PathPick, PathPicks};

/// One picked coordinate in project JSON: `[x, y]` or `[x, y, z]`.
#[derive(Clone, Debug, PartialEq)]
pub struct PathPickDto(pub PathPick);

impl Serialize for PathPickDto {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.0.z {
            Some(z) => [self.0.xy.x, self.0.xy.y, z].serialize(s),
            None => [self.0.xy.x, self.0.xy.y].serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for PathPickDto {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Xy([f64; 2]),
            Xyz([f64; 3]),
        }
        let pick = match Repr::deserialize(d)? {
            Repr::Xy([x, y]) => PathPick::xy(Point2::new(x, y)),
            Repr::Xyz([x, y, z]) => PathPick { xy: Point2::new(x, y), z: Some(z) },
        };
        Ok(Self(pick))
    }
}

/// Operation-level `picks` object in project JSON.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct PathPicksDto {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub start: Vec<PathPickDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exit: Vec<PathPickDto>,
}

impl PathPicksDto {
    /// True when neither role holds a pick; keeps `picks` off the wire.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start.is_empty() && self.exit.is_empty()
    }

    #[must_use]
    pub fn from_picks(picks: &PathPicks) -> Self {
        Self {
            start: picks.start.iter().copied().map(PathPickDto).collect(),
            exit: picks.exit.iter().copied().map(PathPickDto).collect(),
        }
    }

    #[must_use]
    pub fn to_picks(&self) -> PathPicks {
        PathPicks {
            start: self.start.iter().map(|d| d.0).collect(),
            exit: self.exit.iter().map(|d| d.0).collect(),
        }
    }

    /// Convert to runtime `PathPicks`, filling any empty role from pre-#3339
    /// legacy keys (`links.approach.start_point`/`exit_point` or wire `start`).
    #[must_use]
    pub fn with_legacy(&self, start: Option<[f64; 2]>, exit: Option<[f64; 2]>) -> PathPicks {
        let mut out = self.to_picks();
        if out.start.is_empty() {
            if let Some([x, y]) = start {
                out.start.push(PathPick::xy(Point2::new(x, y)));
            }
        }
        if out.exit.is_empty() {
            if let Some([x, y]) = exit {
                out.exit.push(PathPick::xy(Point2::new(x, y)));
            }
        }
        out
    }
}
