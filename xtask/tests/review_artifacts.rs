//! Provenance: issue #1794; PR review evidence belongs outside the product tree.

use std::path::Path;

#[test]
fn reviewed_screenshot_directories_are_absent() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let present: Vec<_> = [
        "docs/pr-1711",
        "docs/pr-1771",
        "docs/review/1765",
        "docs/reviews/1756",
    ]
    .into_iter()
    .filter(|path| root.join(path).exists())
    .collect();

    assert!(
        present.is_empty(),
        "review screenshot directories remain in the product tree: {}",
        present.join(", ")
    );
}
