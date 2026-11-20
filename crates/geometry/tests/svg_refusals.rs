//! #1880: malformed SVG input is refused by name and never panics or hangs (Prime Directive).
//!
//! Provenance: hand-built malformed SVG text and deterministic byte mutations.
//! Gate map (M4.4): covers camcraft_geometry::svg.

use camcraft_geometry::dxf::Limits;
use camcraft_geometry::svg;

fn page(body: &str) -> String {
    format!(r#"<svg width="100mm" height="100mm" viewBox="0 0 100 100">{body}</svg>"#)
}

fn refuses(text: &str, name: &str) {
    let error = svg::parse(text.as_bytes(), None).expect_err("bad SVG must be refused").to_string();
    assert!(error.to_lowercase().contains(&name.to_lowercase()), "{error:?} does not name {name:?}");
}

#[test]
fn svg_malformed_documents_are_named() {
    refuses("", "no <svg>");
    refuses("hello", "no <svg>");
    refuses("<html></html>", "no <svg>");
    refuses("<svg", "unterminated");
    refuses("<svg><g></svg>", "does not match");
    refuses("<svg><rect/>", "not closed");
    refuses("<svg><!-- open", "unterminated comment");
    refuses("<svg width=100mm></svg>", "quoted");
    let gzip: &[u8] = b"\x1f\x8b\x08\0";
    assert!(svg::parse(gzip, None).expect_err("svgz").to_string().contains("compressed"));
    let bad_utf8: &[u8] = b"<svg>\xff</svg>";
    assert!(svg::parse(bad_utf8, None).expect_err("not UTF-8").to_string().contains("UTF-8"));
}

#[test]
fn svg_malformed_paths_and_shapes_are_named() {
    refuses(&page(r#"<path d="L10 10"/>"#), "malformed path");
    refuses(&page(r#"<path d="M10 10 L20"/>"#), "malformed path");
    refuses(&page(r#"<path d="M10 10 X20 20"/>"#), "malformed path");
    refuses(&page(r#"<path d="M0 0 A0 5 0 0 1 10 10"/>"#), "degenerate arc from (0, 0) to (10, 10): a radius is zero");
    refuses(&page(r#"<path d="M0 0 A5 5 0 2 1 10 10"/>"#), "malformed path");
    refuses(&page(r#"<rect width="-5" height="5"/>"#), "negative");
    refuses(&page(r#"<circle r="-1"/>"#), "negative");
    refuses(&page(r#"<polyline points="0,0 10"/>"#), "points");
    refuses(&page(r#"<rect width="5" height="5" transform="rotate(1 2)"/>"#), "transform");
    refuses(&page(r#"<rect width="5" height="5" transform="scale(0)"/>"#), "transform");
    refuses(&page(r#"<rect width="50%" height="5"/>"#), "percent");
    refuses(&page(r#"<line x2="1e400"/>"#), "finite");
}

#[test]
fn svg_zero_or_negative_document_sizes_are_refused() {
    refuses(r#"<svg width="0mm" height="10mm"><rect width="1" height="1"/></svg>"#, "zero or negative");
    refuses(r#"<svg width="10mm" height="-1mm"><rect width="1" height="1"/></svg>"#, "zero or negative");
    refuses(r#"<svg viewBox="0 0 0 10"><rect width="1" height="1"/></svg>"#, "viewBox");
    refuses(r#"<svg viewBox="0 0 10"><rect width="1" height="1"/></svg>"#, "viewBox");
    refuses(r#"<svg width="10mm"><rect width="1" height="1"/></svg>"#, "no height");
}

#[test]
fn svg_import_budget_is_enforced() {
    let small = Limits { max_bytes: 64, max_entities: 4, max_vertices: 16 };
    let big = page(&"<line x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"/>".repeat(4));
    let error = svg::parse_with_limits(big.as_bytes(), None, small).expect_err("bytes").to_string();
    assert!(error.contains("import budget"), "{error}");
    let roomy = Limits { max_bytes: 1 << 20, ..small };
    let many = page(&"<line x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\"/>".repeat(8));
    let error = svg::parse_with_limits(many.as_bytes(), None, roomy).expect_err("elements").to_string();
    assert!(error.contains("import budget"), "{error}");
    let curve = page(r#"<circle r="40" transform="scale(1 0.5)"/>"#);
    let error = svg::parse_with_limits(curve.as_bytes(), None, roomy).expect_err("vertices").to_string();
    assert!(error.contains("import budget"), "{error}");
    let deep = format!("<svg width=\"1mm\" height=\"1mm\">{}{}</svg>", "<g>".repeat(5000), "</g>".repeat(5000));
    let error = svg::parse(deep.as_bytes(), None).expect_err("depth").to_string();
    assert!(error.contains("nested"), "{error}");
    // A `use` that refers to itself is cut off, not followed for ever.
    let looped = page(r##"<g id="a"><use href="#a"/><line x2="1"/></g>"##);
    let error = svg::parse(looped.as_bytes(), None).expect_err("loop").to_string();
    assert!(error.contains("use"), "{error}");
}

#[test]
fn svg_deterministic_smoke_fuzz_never_panics() {
    let seed = page(
        r#"<g transform="translate(5 5) rotate(30)"><path d="M0 0 C10 0 10 10 0 10 S-10 20 0 20 Q5 25 10 20 T20 20 A5 3 20 1 0 30 30 Z"/>
        <rect x="1" y="1" width="8" height="4" rx="2" ry="1"/><ellipse rx="3" ry="2"/><polygon points="0,0 4,0 4,4"/></g>"#,
    )
    .into_bytes();
    for cut in 0..seed.len() {
        let _ = svg::parse(&seed[..cut], None);
    }
    let mut state = 0x1880u64;
    for _ in 0..1000 {
        let mut bytes = seed.clone();
        for _ in 0..4 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let index = (state as usize) % bytes.len();
            bytes[index] = (state >> 32) as u8;
        }
        let _ = svg::parse(&bytes, None);
        let _ = svg::parse(&bytes, Some(camcraft_geometry::dxf::Unit::Inches));
    }
}
