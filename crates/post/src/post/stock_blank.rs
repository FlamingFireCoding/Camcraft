//! The setup's stock as a controller blank (#2769): the stream's
//! `StockBlank` / `StockBlankZBottom` PARAMs, and the engine filter that
//! passes them only to a script declaring `stockBlank = true`. Provenance:
//! our own stream design; the blank forms it serves are FANUC MANUAL GUIDE i
//! (B-63874EN) G1902 / G1900, behaviour only.

use super::cnc::Record;
use super::js::Script;
use super::PostError;
use serde_json::json;

/// The PARAM names this module owns.
const SHAPE: &str = "StockBlank";
const BOTTOM: &str = "StockBlankZBottom";

/// What a setup's stock is, as a controller blank: a shape the blank forms
/// can describe, or the reason no form can.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StockBlank {
    /// An axis-aligned box; `z_bottom` is its lower face (world mm).
    Box { z_bottom: f64 },
    /// A cylinder along the world axis `axis` (`'X'`, `'Y'` or `'Z'`);
    /// `z_bottom` is the lower face of its bounding box (world mm).
    Cylinder { axis: char, z_bottom: f64 },
    /// No blank form describes the stock; the word says why.
    Unsupported(&'static str),
}

impl StockBlank {
    /// The `StockBlank` PARAM word: `box`, `cylinder-x|y|z`, or the reason.
    #[must_use]
    pub fn word(self) -> String {
        match self {
            StockBlank::Box { .. } => "box".to_owned(),
            StockBlank::Cylinder { axis, .. } => format!("cylinder-{}", axis.to_ascii_lowercase()),
            StockBlank::Unsupported(why) => why.to_owned(),
        }
    }

    fn z_bottom(self) -> Option<f64> {
        match self {
            StockBlank::Box { z_bottom } | StockBlank::Cylinder { z_bottom, .. } => {
                z_bottom.is_finite().then_some(z_bottom)
            }
            StockBlank::Unsupported(_) => None,
        }
    }

    /// The stream PARAMs for this blank (none when the bottom is not finite
    /// for a shape that needs it: the post then states no shape at all).
    #[must_use]
    pub fn params(self) -> Vec<Record> {
        let mut out = vec![Record::Param { name: SHAPE.into(), value: json!(self.word()) }];
        match (self, self.z_bottom()) {
            (StockBlank::Unsupported(_), _) => {}
            (_, Some(z)) => out.push(Record::Param { name: BOTTOM.into(), value: json!(z) }),
            (_, None) => return vec![Record::Param { name: SHAPE.into(), value: json!("non-finite") }],
        }
        out
    }
}

/// Read the manifest once, before dispatching the blank PARAMs. Keep the
/// records in the stream so line maps and traces retain their record indices.
pub(super) fn reads_params(s: &mut Script) -> Result<bool, PostError> {
    let reads = s.eval_src("typeof stockBlank !== \"undefined\" && stockBlank === true")?;
    Ok(reads.as_boolean().unwrap_or(false))
}

pub(super) fn is_param(record: &Record) -> bool {
    matches!(record, Record::Param { name, .. } if name == SHAPE || name == BOTTOM)
}
