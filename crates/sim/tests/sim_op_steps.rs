//! #2608: the operation steps of the simulation timeline. "Next" from
//! inside an operation goes to the start of the next one; "previous"
//! goes to the start of the current one, then to the one before it. The
//! first and the last operation have no step past them, and an operation
//! with no moves is not a stop.
//! Provenance: our own tests over our own timeline.
//! Gate map: covers camcraft_sim::sim (sim_steps).

use camcraft_sim::cl::{Move, MoveKind, Toolpath};
use camcraft_sim::geometry::Point3;
use camcraft_sim::sim::{Script, Traverse, script};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// A short path: a rapid in, a plunge and one cut, `x` mm along X.
fn path(x: f64) -> Toolpath {
    let mut t = Toolpath::default();
    t.moves.push(Move::new(MoveKind::Rapid, p(x, 0.0, 10.0), p(x, 0.0, 5.0), None));
    t.moves.push(Move::new(MoveKind::Plunge, p(x, 0.0, 5.0), p(x, 0.0, 0.0), Some(300.0)));
    t.moves.push(Move::new(MoveKind::Feed, p(x, 0.0, 0.0), p(x + 20.0, 0.0, 0.0), Some(600.0)));
    t
}

/// Three operations; the middle path of `with_empty` has no moves.
fn three(with_empty: bool) -> Script {
    let (a, b, c, e) = (path(0.0), path(40.0), path(80.0), Toolpath::default());
    let mut ops: Vec<(&str, &Toolpath)> = vec![("Face1", &a), ("Contour2", &b)];
    if with_empty {
        ops.push(("Empty", &e));
    }
    ops.push(("Pocket3", &c));
    script(ops, &Traverse::default())
}

fn near(a: Option<f64>, b: f64) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-9)
}

#[test]
fn each_operation_has_one_start_in_program_order() {
    let sc = three(false);
    let starts = sc.op_starts();
    assert_eq!(starts.iter().map(|&(op, _)| sc.ops[op].as_str()).collect::<Vec<_>>(), ["Face1", "Contour2", "Pocket3"]);
    assert_eq!(starts[0].1, 0.0, "the first operation starts at the start");
    assert!(starts[0].1 < starts[1].1 && starts[1].1 < starts[2].1, "the starts go forward: {starts:?}");
    for &(op, s) in &starts {
        assert_eq!(sc.at(s).op, Some(op), "the playhead at a start is in that operation");
    }
}

#[test]
fn next_from_inside_the_first_operation_goes_to_the_start_of_the_second() {
    let sc = three(false);
    let s = sc.op_starts();
    let inside = (s[0].1 + s[1].1) * 0.5;
    assert!(near(sc.next_op_start(inside), s[1].1));
    assert!(near(sc.next_op_start(0.0), s[1].1), "from the very start too");
    assert_eq!(sc.op_starting_at(s[1].1), Some("Contour2"));
    // The step lands on the start: a second "next" goes on.
    assert!(near(sc.next_op_start(s[1].1), s[2].1));
}

#[test]
fn previous_goes_to_the_current_start_then_to_the_one_before() {
    let sc = three(false);
    let s = sc.op_starts();
    let inside_second = (s[1].1 + s[2].1) * 0.5;
    assert!(near(sc.prev_op_start(inside_second), s[1].1), "first press: the start of the current operation");
    assert!(near(sc.prev_op_start(s[1].1), s[0].1), "second press: the start of the one before");
    assert!(near(sc.prev_op_start(sc.total), s[2].1), "at the end: the start of the last operation");
}

#[test]
fn the_first_and_the_last_operation_have_no_step_past_them() {
    let sc = three(false);
    let s = sc.op_starts();
    assert_eq!(sc.prev_op_start(0.0), None, "previous is off at the start of the first operation");
    assert_eq!(sc.next_op_start(s[2].1), None, "next is off in the last operation");
    assert_eq!(sc.next_op_start(sc.total), None, "next is off at the end");
    let empty = Script::default();
    assert_eq!((empty.next_op_start(0.0), empty.prev_op_start(0.0)), (None, None));
    assert!(empty.op_starts().is_empty());
}

#[test]
fn an_operation_with_no_moves_is_not_a_stop() {
    let sc = three(true);
    let names: Vec<&str> = sc.op_starts().iter().map(|&(op, _)| sc.ops[op].as_str()).collect();
    assert_eq!(names, ["Face1", "Contour2", "Pocket3"], "the empty path is no stop");
    let s = sc.op_starts();
    assert_eq!(sc.op_starting_at(sc.next_op_start(s[1].1).unwrap_or(f64::NAN)), Some("Pocket3"));
}

#[test]
fn a_non_finite_playhead_reads_as_the_end() {
    let sc = three(false);
    assert_eq!(sc.next_op_start(f64::NAN), None);
    assert!(near(sc.prev_op_start(f64::INFINITY), sc.op_starts()[2].1));
}
