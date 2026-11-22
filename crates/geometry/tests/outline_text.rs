//! #3072: typed text in a TrueType outline font becomes closed shapes - holes nest in their
//! letters, curves are flattened inside the chord tolerance, composite glyphs are resolved, and
//! every font or text that cannot give clear shapes is refused by name.
//!
//! Provenance: our own tests over our own reader, on the generated font of
//! `outline_text::sample_font` (our own glyphs, no font file).
//!
//! Gate map: covers camcraft_geometry::outline_text.

use camcraft_geometry::outline_text::sample_font::sample_font;
use camcraft_geometry::outline_text::{
    CHORD_TOLERANCE, FontError, OutlineSpec, OutlineTextError, check_font, outlines, sketch,
};
use camcraft_geometry::sketch_chains::LoopRole;

fn spec<'a>(font: &'a [u8], text: &'a str) -> OutlineSpec<'a> {
    OutlineSpec { font, text, height: 7.0, letter_spacing: 0.0 }
}

fn roles(font: &[u8], text: &str) -> (usize, usize) {
    let data = sketch(&spec(font, text)).unwrap_or_else(|e| panic!("{text}: {e}"));
    let closed = &data.chains().closed;
    let holes = closed.iter().filter(|c| matches!(c.nesting, Ok(n) if n.role == LoopRole::Hole)).count();
    assert!(data.chains().open.is_empty() && data.chains().rejected.is_empty());
    (closed.len() - holes, holes)
}

#[test]
fn letters_become_closed_shapes_with_their_holes() {
    let font = sample_font();
    // H and I are one shape each; O is a shape with a hole.
    assert_eq!(roles(&font, "HI"), (2, 0));
    assert_eq!(roles(&font, "O"), (1, 1));
    assert_eq!(roles(&font, "HOD O"), (4, 2));
    // The composite 8 is two half-size Os: two shapes, two holes.
    assert_eq!(roles(&font, "8"), (2, 2));
}

#[test]
fn the_cap_height_and_the_advances_place_the_letters_in_mm() {
    let font = sample_font();
    // H is 700 units tall: at a cap height of 7 mm one unit is 0.01 mm.
    let lines = outlines(&spec(&font, "H I")).expect("H I");
    let (mut lo, mut hi) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
    for p in lines.iter().flatten() {
        lo = (lo.0.min(p.x), lo.1.min(p.y));
        hi = (hi.0.max(p.x), hi.1.max(p.y));
    }
    // H advances 600, the space 300; I is 100 wide.
    assert_eq!((lo, hi), ((0.0, 0.0), (10.0, 7.0)));
    // Letter spacing adds its gap after each character.
    let spaced = outlines(&OutlineSpec { letter_spacing: 1.5, ..spec(&font, "II") }).expect("II");
    let right = spaced.iter().flatten().map(|p| p.x).fold(f64::MIN, f64::max);
    assert!((right - (2.0 + 1.5 + 1.0)).abs() < 1e-9, "{right}");
}

#[test]
fn curves_are_flattened_inside_the_chord_tolerance() {
    let font = sample_font();
    // D's right side is two quadratic pieces through (6, 3.5); each point lies on the curve.
    let lines = outlines(&spec(&font, "D")).expect("D");
    let curve: Vec<_> = lines[0].iter().filter(|p| p.x > 3.0 + 1e-9).collect();
    assert!(curve.len() > 8, "the curve is flattened into many lines, got {}", curve.len());
    assert!(curve.iter().any(|p| (p.x - 6.0).abs() < 1e-9 && (p.y - 3.5).abs() < 1e-9));
    // The upper piece is (3, 7) (6, 7) (6, 3.5): each flattened point lies on it.
    for p in curve.iter().filter(|p| p.y > 3.5) {
        // B(s) = (3 + 3 (2s - s^2), 7 - 3.5 s^2): find s from y, then check x.
        let s = ((7.0 - p.y) / 3.5).sqrt();
        assert!((3.0 + 3.0 * (2.0 * s - s * s) - p.x).abs() < 1e-6, "{p:?} is not on the curve");
    }
    // The contour of o has no on-curve point: it still closes as one smooth shape.
    let o = outlines(&spec(&font, "o")).expect("o");
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].first(), o[0].last());
    assert!(o[0].len() > 16 && CHORD_TOLERANCE <= 0.005);
}

#[test]
fn a_font_that_cannot_be_read_is_refused_by_name() {
    let font = sample_font();
    let why = |bytes: &[u8]| match check_font(bytes) {
        Ok(()) => panic!("must be refused"),
        Err(e) => e,
    };
    assert_eq!(why(b"not a font at all"), OutlineTextError::Font(FontError::NotAFont));
    assert_eq!(why(b"OTTO\0\0\0\0\0\0\0\0"), OutlineTextError::Font(FontError::CffOutlines));
    assert_eq!(why(b"ttcf\0\0\0\0\0\0\0\0"), OutlineTextError::Font(FontError::Collection));
    assert!(why(&font[..40]).to_string().starts_with("the font cannot be read"));
    // Every cut of the file is refused or read, never a panic.
    for len in 0..font.len() {
        let _ = sketch(&spec(&font[..len], "HO8Do"));
    }
    // Every flipped byte is refused or read, never a panic or a hang.
    for at in 0..font.len() {
        let mut bad = font.clone();
        bad[at] ^= 0xA5;
        let _ = sketch(&spec(&bad, "HO8Do X"));
    }
}

#[test]
fn bad_text_and_overlapping_outlines_are_refused_by_name() {
    let font = sample_font();
    let err = |s: OutlineSpec| sketch(&s).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(err(spec(&font, "  ")).contains("the text is empty"));
    assert!(err(spec(&font, "HZ")).contains("'Z' at character 2 is not in the font"));
    assert!(err(OutlineSpec { height: 0.1, ..spec(&font, "H") }).contains("cap height from 0.5 mm"));
    assert!(err(OutlineSpec { letter_spacing: -1.0, ..spec(&font, "H") }).contains("letter spacing"));
    // X's two bars cross: a pocket of them is not clear.
    assert!(err(spec(&font, "X")).contains("the text outlines cross or touch"), "{}", err(spec(&font, "X")));
}
