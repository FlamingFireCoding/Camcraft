//! Outline text: a typed string in a TrueType outline font made into closed shapes, and into
//! sketch data that the document places like a DXF drawing - for a pocket, a V-carve or a
//! cut-out.
//!
//! Provenance: our own design for issue 3072. The font tables are read by `ttf` from the public
//! OpenType specification; a quadratic B-spline contour is flattened to lines by the standard
//! chord-error bound of a quadratic Bezier curve. No vendor material was consulted.
//!
//! Contract. The shapes are in millimetres. The origin is the left end of the baseline, and
//! `height` is the cap height of the font. Nothing is guessed: a font that cannot be read, a
//! font with CFF outlines, a character the font does not have, and outlines that cross or
//! touch are refused by name. The sketch that [`sketch`] returns has only closed chains, each
//! with a clear nesting, or the call fails. This module has no file I/O and no application
//! state.

#[doc(hidden)]
pub mod sample_font;
mod ttf;

use crate::dxf::{ImportSummary, LayeredPrimitive, Primitive, Unit, Units};
use crate::geo2d::Point2;
use crate::sketch_data::{SketchData, SketchError, SourceUnits};
use crate::stroke_text::{MAX_CHARS, MAX_HEIGHT, MIN_HEIGHT};
use ttf::{Font, OutlinePoint};

/// The layer of each shape.
pub const LAYER: &str = "text";
/// The largest font file that is read, bytes.
pub const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
/// The largest distance of a flattened curve from the true curve, mm.
pub const CHORD_TOLERANCE: f64 = 0.005;
/// The join tolerance of the text sketch, mm: far below the chord tolerance, so a short
/// segment of a small letter is still a segment.
pub const JOIN_TOLERANCE: f64 = 0.0005;
/// A flattened point closer than this to the point before it is dropped, mm.
const MIN_SEGMENT: f64 = 2.0 * JOIN_TOLERANCE;
/// The most lines that one quadratic piece becomes.
const MAX_PIECES: usize = 64;

/// A font that cannot be used, by name.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum FontError {
    #[error("the file is not a TrueType or OpenType font")]
    NotAFont,
    #[error(
        "the font has CFF (PostScript) outlines: only fonts with TrueType outlines can be read - \
         pick a .ttf font"
    )]
    CffOutlines,
    #[error("the file is a font collection (.ttc): pick a single font file (.ttf or .otf)")]
    Collection,
    #[error("the font file is cut short or an offset in it points past its end")]
    Truncated,
    #[error("the font has no '{0}' table")]
    MissingTable(&'static str),
    #[error("the font is corrupt: {0}")]
    Corrupt(&'static str),
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum OutlineTextError {
    #[error("the text is empty: type one or more characters")]
    Empty,
    #[error("the text has {chars} characters: the limit is {max}")]
    TooLong { chars: usize, max: usize },
    #[error("the text height {height} mm is not usable: state a cap height from 0.5 mm to 1000 mm")]
    Height { height: f64 },
    #[error("the letter spacing {spacing} mm is not usable: state 0 mm to 1000 mm")]
    Spacing { spacing: f64 },
    #[error("the font file is {bytes} bytes: the limit is {max} bytes")]
    FontTooLarge { bytes: usize, max: usize },
    #[error("the font cannot be read: {0}")]
    Font(#[from] FontError),
    #[error("{ch:?} at character {position} is not in the font: pick a font that has it")]
    NoGlyph { ch: char, position: usize },
    #[error("the text has only spaces in this font: there is no shape to place")]
    NoShape,
    /// The outlines cross or touch (an overlap in the font, or letters that run into each
    /// other), so a pocket or a cut-out of them is not clear.
    #[error(
        "the text outlines cross or touch, so the shapes are not clear: {0}. Add letter spacing, \
         or pick a font whose letters do not overlap"
    )]
    Shapes(String),
}

/// What to write: the font file's bytes, the string, the cap height in mm, and the extra gap
/// between two characters in mm (0 is the font's own advance).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlineSpec<'a> {
    pub font: &'a [u8],
    pub text: &'a str,
    pub height: f64,
    pub letter_spacing: f64,
}

/// Check that `font` is a font this module reads, without a text.
pub fn check_font(font: &[u8]) -> Result<(), OutlineTextError> {
    if font.len() > MAX_FONT_BYTES {
        return Err(OutlineTextError::FontTooLarge { bytes: font.len(), max: MAX_FONT_BYTES });
    }
    Font::parse(font)?;
    Ok(())
}

/// The closed outlines of `spec` as polylines in millimetres, in reading order. Each polyline
/// repeats its first point as its last point.
pub fn outlines(spec: &OutlineSpec) -> Result<Vec<Vec<Point2>>, OutlineTextError> {
    let OutlineSpec { font, text, height, letter_spacing } = *spec;
    if text.trim().is_empty() {
        return Err(OutlineTextError::Empty);
    }
    let chars = text.chars().count();
    if chars > MAX_CHARS {
        return Err(OutlineTextError::TooLong { chars, max: MAX_CHARS });
    }
    if !(height.is_finite() && (MIN_HEIGHT..=MAX_HEIGHT).contains(&height)) {
        return Err(OutlineTextError::Height { height });
    }
    if !(letter_spacing.is_finite() && (0.0..=MAX_HEIGHT).contains(&letter_spacing)) {
        return Err(OutlineTextError::Spacing { spacing: letter_spacing });
    }
    check_font(font)?;
    let font = Font::parse(font)?;
    let scale = height / font.cap_height()?;
    let space = font.glyph_id(' ')?;
    let mut out = Vec::new();
    let mut pen = 0.0_f64;
    for (index, ch) in text.chars().enumerate() {
        let gid = match font.glyph_id(ch)? {
            Some(g) => g,
            // A font that has no space glyph still spaces a word: a quarter of the em.
            None if ch == ' ' && space.is_none() => {
                pen += 0.25 * f64::from(font.units_per_em) * scale + letter_spacing;
                continue;
            }
            None => return Err(OutlineTextError::NoGlyph { ch, position: index + 1 }),
        };
        for contour in font.outline(gid)? {
            if let Some(line) = flatten(&contour, pen, scale) {
                out.push(line);
            }
        }
        pen += f64::from(font.advance(gid)?) * scale + letter_spacing;
    }
    if out.is_empty() {
        return Err(OutlineTextError::NoShape);
    }
    Ok(out)
}

/// One TrueType contour (on-curve points and quadratic control points, closed) as a closed
/// polyline in mm, moved right by `pen`. A contour that flattens to fewer than three distinct
/// points encloses nothing and is dropped.
fn flatten(contour: &[OutlinePoint], pen: f64, scale: f64) -> Option<Vec<Point2>> {
    let at = |p: &OutlinePoint| Point2::new(pen + p.x * scale, p.y * scale);
    let mid = |a: Point2, b: Point2| Point2::new(0.5 * (a.x + b.x), 0.5 * (a.y + b.y));
    let n = contour.len();
    // Start on the curve: an on point, else the midpoint of two control points.
    let first_on = contour.iter().position(|p| p.on);
    let (start, offset) = match first_on {
        Some(i) => (at(&contour[i]), i),
        None => (mid(at(contour.first()?), at(contour.last()?)), 0),
    };
    let mut line = vec![start];
    // With no on point, the first control point follows the start; the last one closes.
    let mut control = if first_on.is_some() { None } else { Some(at(&contour[0])) };
    let mut last = start;
    let steps = if first_on.is_some() { n } else { n - 1 };
    for k in 1..=steps {
        let p = &contour[(offset + k) % n];
        let q = at(p);
        let (end, next_control) = match (p.on, control) {
            (true, _) => (q, None),
            (false, None) => {
                control = Some(q);
                continue;
            }
            (false, Some(c)) => (mid(c, q), Some(q)),
        };
        match control {
            Some(c) => quadratic(&mut line, last, c, end),
            None => push(&mut line, end),
        }
        last = end;
        control = next_control;
    }
    if let Some(c) = control {
        quadratic(&mut line, last, c, start);
    }
    // Close exactly on the start point.
    while line.len() > 1 && line.last().is_some_and(|p| dist(*p, start) < MIN_SEGMENT) {
        line.pop();
    }
    line.push(start);
    (line.len() >= 4).then_some(line)
}

fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn push(line: &mut Vec<Point2>, p: Point2) {
    if line.last().is_none_or(|q| dist(*q, p) >= MIN_SEGMENT) {
        line.push(p);
    }
}

/// The quadratic Bezier curve `a`, `c`, `b` as lines whose chord error is at most
/// [`CHORD_TOLERANCE`]: the error of `n` even steps is |a - 2c + b| / (4 n^2).
fn quadratic(line: &mut Vec<Point2>, a: Point2, c: Point2, b: Point2) {
    let bend = (a.x - 2.0 * c.x + b.x).hypot(a.y - 2.0 * c.y + b.y);
    let pieces = (bend / (4.0 * CHORD_TOLERANCE)).sqrt().ceil();
    let pieces = if pieces.is_finite() { (pieces as usize).clamp(1, MAX_PIECES) } else { 1 };
    for i in 1..=pieces {
        let t = i as f64 / pieces as f64;
        let s = 1.0 - t;
        push(line, Point2::new(s * s * a.x + 2.0 * s * t * c.x + t * t * b.x, s * s * a.y + 2.0 * s * t * c.y + t * t * b.y));
    }
}

/// The outlines of `spec` as sketch data: line primitives in millimetres on the layer
/// [`LAYER`], chained with [`JOIN_TOLERANCE`]. Every chain is closed with a clear nesting.
pub fn sketch(spec: &OutlineSpec) -> Result<SketchData, OutlineTextError> {
    let lines = outlines(spec)?;
    let mut primitives = Vec::new();
    let mut vertices = 0_usize;
    for line in &lines {
        vertices = vertices.saturating_add(line.len());
        for pair in line.windows(2) {
            if let [start, end] = *pair {
                primitives.push(LayeredPrimitive { layer: LAYER.into(), primitive: Primitive::Line { start, end } });
            }
        }
    }
    let units = SourceUnits { stated: Units::Known(Unit::Millimeters), applied: Unit::Millimeters };
    let summary = ImportSummary { entities: primitives.len(), vertices, skipped: Default::default() };
    let data = SketchData::from_parts(primitives, vec![LAYER.into()], units, Some(JOIN_TOLERANCE), summary)
        .map_err(|e| match e {
            SketchError::Empty { .. } => OutlineTextError::NoShape,
            other => OutlineTextError::Shapes(other.to_string()),
        })?;
    let chains = data.chains();
    if let Some(rejected) = chains.rejected.first() {
        return Err(OutlineTextError::Shapes(rejected.error.to_string()));
    }
    if let Some(open) = chains.open.first() {
        return Err(OutlineTextError::Shapes(format!("an outline does not close near ({:.3}, {:.3})", open.start.x, open.start.y)));
    }
    if let Some(error) = chains.closed.iter().find_map(|chain| chain.nesting.as_ref().err()) {
        return Err(OutlineTextError::Shapes(error.to_string()));
    }
    Ok(data)
}
