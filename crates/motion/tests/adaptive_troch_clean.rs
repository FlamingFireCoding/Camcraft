//! Issue #3849: 2D Adaptive (Trochoidal) clean-up passes are checked against
//! `max_engagement` (including the 90 deg default), stepped down when needed,
//! and surfaced with a named diagnostic when even reduced steps exceed it.

#[path = "common/adaptive_troch.rs"]
mod fixtures;

use camcraft_motion::geo2d::Region;
use camcraft_motion::geometry::Point3;
use camcraft_motion::machining2d::{
    AdaptiveTrochParams, CLEANUP_ENGAGEMENT_NOTE_PREFIX, PocketParams, adaptive_trochoidal_with,
    cleanup_engagement_exceeded, cleanup_engagement_note,
};
use camcraft_motion::quality::{QualitySpec, measure, stock_for};
use fixtures::*;

fn assert_cleanup_checked(name: &str, region: &Region, p: &PocketParams, max_engagement: f64) {
    let t = AdaptiveTrochParams { max_engagement, ..AdaptiveTrochParams::default() };
    let path = adaptive_trochoidal_with(region, p, &t).unwrap_or_else(|e| panic!("{name}: {e}"));
    let spec = QualitySpec::new(p.tool_diameter);
    let mut stock = stock_for(region, p.top_z, p.bottom_z, p.tool_diameter, &spec).expect("stock");
    let q = measure(&path, &mut stock, &spec).expect("measure");
    let warned = cleanup_engagement_exceeded(&path.notes);
    if q.max_deg > max_engagement + 0.5 {
        let (measured, limit) = warned.unwrap_or_else(|| {
            panic!(
                "{name}: engagement {:.1} deg exceeds {:.1} deg without a named warning; notes={:?}",
                q.max_deg, max_engagement, path.notes
            )
        });
        assert!(measured > limit && (limit - max_engagement).abs() < 0.2, "{name}: {warned:?}");
    } else {
        assert!(warned.is_none(), "{name}: unexpected warning {warned:?} at {:.1} deg", q.max_deg);
    }
}

#[test]
fn cleanup_engagement_is_checked_on_default_and_custom_limits() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    // Default 90 deg limit on #1587 face fixture stays <= 90 deg cleanly.
    assert_cleanup_checked("face 34x16 D6 (90 deg)", &face(), &p, 90.0);

    // Regression (#3849): on upstream/main prior to this fix, wide 30x24 D6 and
    // L-shape D10 at 60 deg passed lanes <= 60 deg, hit 64.0 and 62.5 deg in
    // unstepped cleanup, and never stepped cleanup down or surfaced a warning.
    let wide = rect(0.0, 0.0, 30.0, 24.0);
    assert_cleanup_checked("wide 30x24 D6 (60 deg)", &wide, &p, 60.0);

    let l_shape = Region::new(vec![poly(&[
        (0.0, 0.0), (60.0, 0.0), (60.0, 30.0), (30.0, 30.0), (30.0, 60.0), (0.0, 60.0),
    ])])
    .expect("L-shape");
    assert_cleanup_checked("L-shape D10 (60 deg)", &l_shape, &PocketParams { tool_diameter: 10.0, ..p }, 60.0);
}

#[test]
fn cleanup_engagement_note_round_trips_for_2d_and_3d() {
    let note_2d = cleanup_engagement_note(102.5, Point3::new(1.25, -2.5, 0.0), 90.0);
    assert!(note_2d.starts_with(CLEANUP_ENGAGEMENT_NOTE_PREFIX), "{note_2d}");
    assert_eq!(cleanup_engagement_exceeded(&[note_2d.clone()]), Some((102.5, 90.0)));

    let note_3d = note_2d.replacen("2D Adaptive (Trochoidal):", "3D Adaptive (Trochoidal):", 1);
    assert_eq!(cleanup_engagement_exceeded(&[note_3d]), Some((102.5, 90.0)));
    assert_eq!(cleanup_engagement_exceeded(&["2D Adaptive (Trochoidal): 16 corner/wall clean-up pass(es) emitted".into()]), None);
}
