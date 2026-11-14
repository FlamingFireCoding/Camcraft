//! The Tormach PathPilot (LinuxCNC) LATHE post's G76 pass count (#1619):
//! under R1 (LinuxCNC G76: "R1.0 selects constant depth") the control
//! cuts J, 2J, ... with the last pass clamped at K, so ceil(K/J) passes.
//! The posted grid words must make that count the record's own rung
//! count - the passes the stream laid and the verdict judged - in mm
//! and in inch, or the record is refused by name.
//!
//! Gate map (M4.4): covers app::posting (stream_records, post_setup on
//! the Tormach lathe post), post::post (engine on the PathPilot lathe
//! dialect: the G76 grid words).
//! Lane shape (M4.4): all tests here run in seconds.

mod tormach_lathe_support;

use camcraft_app::post::{cnc, cnc_thread};
use camcraft_app::posting;
use camcraft_app::schema::PostOptions;
use tormach_lathe_support::{g33s, g76, g76_words, post, program_with, thread_world};

fn with_rec(records: &[cnc::Record], f: impl Fn(&mut cnc_thread::ThreadRec)) -> Vec<cnc::Record> {
    records
        .iter()
        .cloned()
        .map(|r| match r {
            cnc::Record::Thread(Some(mut t)) => {
                f(&mut t);
                cnc::Record::Thread(Some(t))
            }
            other => other,
        })
        .collect()
}

/// A G76 word as a whole count of the control's grid steps.
fn steps(text: &str, c: char, inch: bool) -> i64 {
    let step = if inch { 0.0001 } else { 0.001 };
    let v = g76_words(text).iter().find(|(l, _)| *l == c).map_or(f64::NAN, |(_, v)| *v);
    (v / step).round() as i64
}

/// LinuxCNC's R1 ladder: J, 2J, ... clamped at K - ceil(K/J) passes.
fn r1_passes(text: &str, inch: bool) -> i64 {
    let (j, k) = (steps(text, 'J', inch), steps(text, 'K', inch));
    assert!(j >= 1 && k >= j, "J {j} K {k}: {text}");
    (k + j - 1) / j
}

#[test]
fn g76_words_cut_exactly_the_records_passes_in_mm_and_inch() {
    for inch in [false, true] {
        let (world, _history) = thread_world(inch);
        // The record's rungs: the passes the stream lays (one G33 each
        // without the cycle) and ceil(height / first depth).
        let laid = g33s(&program_with(&world, PostOptions::default())).len() as i64;
        let records = posting::stream_records(&world).expect("generated");
        let t = records
            .iter()
            .find_map(|r| match r {
                cnc::Record::Thread(Some(t)) => Some(t.clone()),
                _ => None,
            })
            .expect("a thread record");
        let rungs = (t.height / t.first_depth - 1e-9).ceil() as i64;
        assert_eq!((laid, rungs), (3, 3), "the fixture's ladder: 0.4, 0.8, 1.2");
        let text = program_with(&world, g76());
        assert_eq!(r1_passes(&text, inch), rungs, "inch {inch}: the control's ladder is not the record's: {text}");
        // The first cut is never a full grid step heavier than the
        // record's: inch J 0.0315 against the record's 0.031496.
        let j_mm = steps(&text, 'J', inch) as f64 * if inch { 0.00254 } else { 0.001 };
        let step = if inch { 0.00254 } else { 0.001 };
        assert!(j_mm - 2.0 * t.first_depth < step, "J {j_mm} mm a full step over 2 x {}: {text}", t.first_depth);
    }
}

#[test]
fn g76_refuses_by_name_when_no_grid_j_cuts_the_records_passes() {
    let why = "no first depth J on the control's grid";
    // mm: a crest 2 um over the height law (inside the lattice
    // allowance) makes K 2.402; J 0.8 would cut a fourth 0.002 pass,
    // J 0.801 is a full step over the record's first cut. Refused.
    let (world, _history) = thread_world(false);
    let records = posting::stream_records(&world).expect("generated");
    let (text, refused) = post(&world, &with_rec(&records, |t| t.major_diameter += 0.002), g76());
    assert!(refused.iter().any(|w| w.contains("G76") && w.contains(why)), "mm: {refused:?}");
    assert!(!text.contains("G76 P"), "mm: a refused cycle was written: {text}");
    // inch: 0.15 deep in rungs of 0.012 is 13 passes; K is 118 steps
    // and no whole J has 12J < 118 <= 13J. Refused.
    let (world, _history) = thread_world(true);
    let records = posting::stream_records(&world).expect("generated");
    let fine = with_rec(&records, |t| {
        t.height = 0.15;
        t.first_depth = 0.012;
        t.min_depth = 0.012;
        t.major_diameter = t.minor_diameter + 0.3;
    });
    let (text, refused) = post(&world, &fine, g76());
    assert!(refused.iter().any(|w| w.contains("G76") && w.contains(why)), "inch: {refused:?}");
    assert!(!text.contains("G76 P"), "inch: a refused cycle was written: {text}");
}
