//! The machine editor's axis chain (#2756): an axis part is added to or
//! removed from `kinematics.default.parts` through `MachineDraft`, every
//! key the editor does not know is kept, and the parsed machine follows
//! the edit (rotary `parent`, `chain_order` and `turns_part`).
//!
//! Fixture law (clean-room): the shipped grblHAL mill plus our own
//! values in the `.mch` layout of `assets/machines/AGENTS.md`.
//! Provenance: our own tests, specced from the issue's "Done when".
//!
//! Gate map (M4.4): covers app::machine, post::machine::draft, app::machine, post::machine::chain.

use std::path::PathBuf;

use camcraft_app::machine::chain::{Place, Step};
use camcraft_app::machine::draft::MachineDraft;
use camcraft_app::machine::library::MachineSource;
use camcraft_app::machine::validate::validate_text;
use camcraft_app::machine::{parse_mch, AxisLimits};
use serde_json::{json, Value};

/// The shipped 3-axis grblHAL mill: Z carries Y carries X on the
/// table, and the head sits beside Z.
fn grblhal() -> MachineDraft {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines/grblhal-3axis.mch");
    let mut m = MachineDraft::open(&MachineSource::File(p)).expect("opens");
    // Keys the editor knows nothing about, at the top and on an axis.
    m.value["x_vendor_block"] = json!({ "probe": "OMP40" });
    m.value["kinematics"]["default"]["parts"][0]["parts"][0]["parts"][0]["parts"][0]["note"] = json!("gib adjusted 2026");
    m
}

fn axes(m: &MachineDraft) -> Vec<AxisLimits> {
    parse_mch(&m.text()).expect("parses").axes
}

fn axis(m: &MachineDraft, name: &str) -> AxisLimits {
    axes(m).into_iter().find(|a| a.name == name).unwrap_or_else(|| panic!("no axis {name}"))
}

fn names(m: &MachineDraft) -> Vec<String> {
    axes(m).into_iter().map(|a| a.name).collect()
}

fn unknown_keys_kept(v: &Value) {
    assert_eq!(v.pointer("/x_vendor_block/probe"), Some(&json!("OMP40")));
    let x = v.pointer("/kinematics/default/parts/0/parts/0/parts/0/parts/0").expect("X");
    assert_eq!(x.get("name"), Some(&json!("X")));
    assert_eq!(x.get("note"), Some(&json!("gib adjusted 2026")));
}

#[test]
fn an_a_about_minus_x_on_the_table_is_added_and_turns_the_work() {
    let mut m = grblhal();
    let at = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds");
    assert!(m.dirty);
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept(&m.value);

    // On the table part, beside Z and before the head.
    assert_eq!(at, "/kinematics/default/parts/0/parts/1");
    let part = m.value.pointer(&at).expect("the pointer names the new part");
    assert_eq!(part.get("type"), Some(&json!("rotary")));
    assert_eq!(names(&m), ["X", "Y", "Z", "A"]);
    let a = axis(&m, "A");
    assert!(a.rotary);
    assert_eq!(a.axis, Some([-1.0, 0.0, 0.0]));
    assert_eq!(a.turns_part, Some(true), "a rotary beside the head turns the work");
    assert_eq!(a.chain_order, Some(0));
    assert_eq!(a.parent, None);
}

#[test]
fn a_b_that_carries_the_head_tilts_the_tool() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept(&m.value);

    let b = axis(&m, "B");
    assert_eq!(b.turns_part, Some(false), "the head now rides on B");
    // The head and its coolants moved inside B, unchanged.
    let head = m.value.pointer("/kinematics/default/parts/0/parts/1/parts/0").expect("head under B");
    assert_eq!(head.get("type"), Some(&json!("head")));
    assert_eq!(head.pointer("/tool_station/coolants/1/mCode"), Some(&json!("M7")));
}

#[test]
fn a_rotary_added_after_one_on_the_table_is_its_child_when_it_carries_the_head() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::CarriesHead, "A", [1.0, 0.0, 0.0]).expect("adds A");
    m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds B");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    // The head rides on B, which rides on A: A comes first in the chain.
    let (a, b) = (axis(&m, "A"), axis(&m, "B"));
    assert_eq!((a.chain_order, a.parent), (Some(0), None));
    assert_eq!((b.chain_order, b.parent.as_deref()), (Some(1), Some("A")));
}

#[test]
fn removing_the_added_a_gives_back_the_file_as_it_was() {
    let mut m = grblhal();
    let before = m.value.clone();
    let at = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds");
    m.remove_axis(&at).expect("removes");
    assert_eq!(m.value, before);
}

#[test]
fn removing_a_rotary_that_carries_the_head_puts_the_head_back() {
    let mut m = grblhal();
    let before = m.value.clone();
    let at = m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds");
    m.remove_axis(&at).expect("removes");
    assert_eq!(m.value, before);
}

#[test]
fn removing_z_keeps_the_axes_it_carried_in_its_place() {
    let mut m = grblhal();
    m.remove_axis("/kinematics/default/parts/0/parts/0").expect("removes Z");
    assert_eq!(names(&m), ["X", "Y"]);
    unknown_keys_kept_after_z_went(&m.value);
    // Z's controller row goes with it; X and Y keep theirs.
    let ctrl = m.value.pointer("/controller/synced_configuration/parts").expect("controller");
    assert!(ctrl.get("Z").is_none());
    assert!(ctrl.get("X").is_some() && ctrl.get("Y").is_some());
}

fn unknown_keys_kept_after_z_went(v: &Value) {
    assert_eq!(v.pointer("/x_vendor_block/probe"), Some(&json!("OMP40")));
    let x = v.pointer("/kinematics/default/parts/0/parts/0/parts/0").expect("X under Y");
    assert_eq!(x.get("note"), Some(&json!("gib adjusted 2026")));
}

#[test]
fn an_add_that_the_file_cannot_hold_is_refused_by_name_and_changes_nothing() {
    let mut m = grblhal();
    let before = m.value.clone();
    assert_eq!(m.add_linear_axis(Place::OnTable, "X"), Err("axis X: the machine already has an axis X".to_owned()));
    assert_eq!(
        m.add_rotary_axis(Place::OnTable, "Q", [1.0, 0.0, 0.0]),
        Err("axis Q: a rotary is named A, B or C".to_owned())
    );
    assert_eq!(
        m.add_rotary_axis(Place::OnTable, "A", [0.0, 0.0, 0.0]),
        Err("axis A: the axis direction is zero".to_owned())
    );
    assert_eq!(
        m.add_rotary_axis(Place::OnTable, "A", [f64::NAN, 0.0, 0.0]),
        Err("axis A: the axis direction must be three finite numbers".to_owned())
    );
    assert_eq!(m.value, before);
    assert!(!m.dirty);
}

#[test]
fn a_remove_that_names_no_axis_is_refused_by_name() {
    let mut m = grblhal();
    let before = m.value.clone();
    for bad in ["/kinematics/default/parts/0", "/kinematics/default/parts/0/parts/9", "/general", "nonsense"] {
        assert_eq!(m.remove_axis(bad), Err(format!("{bad}: not an axis part")), "{bad}");
    }
    assert_eq!(m.value, before);
}

/// A tilting head of our own: Z carries A carries B carries the head;
/// B states every #1962 key and a key the editor does not know.
fn tilting_head() -> MachineDraft {
    let mut m = grblhal();
    m.add_rotary_axis(Place::CarriesHead, "A", [1.0, 0.0, 0.0]).expect("adds A");
    let b = m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds B");
    let part = m.value.pointer_mut(&b).expect("B");
    part["pivot"] = json!([0.0, 0.0, 150.0]);
    part["offset"] = json!([0.0, 0.0, -40.0]);
    part["zero_offset"] = json!(0.5);
    part["rollover"] = json!(true);
    part["sign_convention"] = json!("tool_rh");
    part["note"] = json!("trunnion B, serial 42");
    m
}

fn trunnion_keys(m: &MachineDraft, name: &str) -> Value {
    let part = m.value.pointer(&pointer_of(m, name)).expect("part").clone();
    let keep = ["id", "pivot", "offset", "zero_offset", "rollover", "sign_convention", "note"];
    keep.iter().filter_map(|k| part.get(*k).map(|x| ((*k).to_owned(), x.clone()))).collect::<serde_json::Map<_, _>>().into()
}

/// The pointer of the axis part named `name`, depth-first.
fn pointer_of(m: &MachineDraft, name: &str) -> String {
    fn walk(v: &Value, at: &str, name: &str) -> Option<String> {
        v.pointer(at)?.as_array()?.iter().enumerate().find_map(|(i, p)| {
            let here = format!("{at}/{i}");
            if p.get("name").and_then(Value::as_str) == Some(name) {
                Some(here)
            } else {
                walk(v, &format!("{here}/parts"), name)
            }
        })
    }
    walk(&m.value, "/kinematics/default/parts", name).unwrap_or_else(|| panic!("no part {name}"))
}

#[test]
fn moving_a_rotary_up_changes_its_parent_and_chain_order_and_keeps_its_keys() {
    let mut m = tilting_head();
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    let keys = trunnion_keys(&m, "B");
    assert_eq!((axis(&m, "B").chain_order, axis(&m, "B").parent.as_deref()), (Some(1), Some("A")));

    let at = m.move_axis(&pointer_of(&m, "B"), Step::Up).expect("moves");
    assert_eq!(at, pointer_of(&m, "B"), "the move names where B is now");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    assert_eq!(trunnion_keys(&m, "B"), keys, "B's own keys move with it");
    let (a, b) = (axis(&m, "A"), axis(&m, "B"));
    assert_eq!((b.chain_order, b.parent), (Some(0), None));
    assert_eq!((a.chain_order, a.parent.as_deref()), (Some(1), Some("B")));
    assert_eq!(b.pivot, Some([0.0, 0.0, 150.0]));
    // The head still rides on the last rotary of the chain.
    assert_eq!(a.turns_part, Some(false));
    assert_eq!(b.turns_part, Some(false));
    unknown_keys_kept(&m.value);
}

#[test]
fn a_move_down_undoes_a_move_up() {
    let mut m = tilting_head();
    let before = m.value.clone();
    let at = m.move_axis(&pointer_of(&m, "B"), Step::Up).expect("up");
    m.move_axis(&at, Step::Down).expect("down");
    assert_eq!(m.value, before);
}

#[test]
fn an_a_on_the_table_is_added_moved_and_removed_and_the_file_validates_each_time() {
    let mut m = grblhal();
    let at = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds");
    // Depth-first the rows read Z, Y, X, A: up puts A in X's slot.
    let at = m.move_axis(&at, Step::Up).expect("moves");
    assert_eq!(at, "/kinematics/default/parts/0/parts/0/parts/0/parts/0");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept_on_x_wherever_it_is(&m);
    assert_eq!(axis(&m, "A").turns_part, Some(true), "no head below A: it still turns the work");
    m.remove_axis(&at).expect("removes");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept_on_x_wherever_it_is(&m);
    assert_eq!(names(&m), ["X", "Y", "Z"]);
}

fn unknown_keys_kept_on_x_wherever_it_is(m: &MachineDraft) {
    assert_eq!(m.value.pointer("/x_vendor_block/probe"), Some(&json!("OMP40")));
    let x = m.value.pointer(&pointer_of(m, "X")).expect("X");
    assert_eq!(x.get("note"), Some(&json!("gib adjusted 2026")));
}

#[test]
fn a_move_past_either_end_or_from_no_axis_is_refused_by_name() {
    let mut m = grblhal();
    let before = m.value.clone();
    let z = "/kinematics/default/parts/0/parts/0";
    let x = "/kinematics/default/parts/0/parts/0/parts/0/parts/0";
    assert_eq!(m.move_axis(z, Step::Up), Err("axis Z: already first in the chain".to_owned()));
    assert_eq!(m.move_axis(x, Step::Down), Err("axis X: already last in the chain".to_owned()));
    assert_eq!(m.move_axis("/general", Step::Up), Err("/general: not an axis part".to_owned()));
    assert_eq!(m.value, before);
    assert!(!m.dirty);
}

#[test]
fn a_linear_u_is_added_on_the_table() {
    let mut m = grblhal();
    m.add_linear_axis(Place::OnTable, "U").expect("adds");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    let u = axis(&m, "U");
    assert!(!u.rotary);
    assert_eq!(
        m.add_linear_axis(Place::OnTable, "A"),
        Err("axis A: a linear axis is named X, Y, Z, U, V or W".to_owned())
    );
}

#[test]
fn the_rows_follow_the_chain_and_say_what_carries_the_head() {
    let mut d = grblhal();
    d.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("A added");
    d.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("B added");
    let rows: Vec<(String, bool, bool)> =
        d.axis_rows().into_iter().map(|r| (r.name, r.rotary, r.carries_head)).collect();
    let row = |n: &str, rotary, head| (n.to_owned(), rotary, head);
    assert_eq!(rows, [row("Z", false, false), row("Y", false, false), row("X", false, false), row("A", true, false), row("B", true, true)]);
}

#[test]
fn a_linear_is_reversed_and_set_back_and_its_other_keys_stay() {
    use camcraft_app::machine::AxisDirection;
    let mut m = grblhal();
    let x = pointer_of(&m, "X");
    m.set_axis_reversed(&x, true).expect("X reversed");
    assert_eq!(m.value.pointer(&format!("{x}/direction")), Some(&json!(-1)));
    assert_eq!(axis(&m, "X").direction, AxisDirection::Inverted);
    assert!(m.axis_rows().iter().any(|r| r.name == "X" && r.reversed));
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());

    // Back to standard: unstated is the file's way of saying standard.
    m.set_axis_reversed(&x, false).expect("X standard");
    assert_eq!(m.value.pointer(&format!("{x}/direction")), None);
    assert_eq!(axis(&m, "X").direction, AxisDirection::Standard);
    unknown_keys_kept(&m.value);
}

#[test]
fn a_rotary_cannot_be_reversed_and_a_linear_has_no_axis() {
    let mut m = grblhal();
    let a = m.add_rotary_axis(Place::OnTable, "A", [1.0, 0.0, 0.0]).expect("A added");
    m.dirty = false;
    assert_eq!(m.set_axis_reversed(&a, true), Err("axis A: a rotary is not reversed; turn its axis instead".to_owned()));
    let x = pointer_of(&m, "X");
    assert_eq!(m.set_rotary_axis(&x, [0.0, 1.0, 0.0]), Err("axis X: a linear axis has no rotary axis".to_owned()));
    assert!(!m.dirty, "a refused edit changes nothing");
}

#[test]
fn a_rotary_turns_about_a_new_axis_and_keeps_its_trunnion_keys() {
    let mut m = tilting_head();
    let before = trunnion_keys(&m, "B");
    let b = pointer_of(&m, "B");
    m.set_rotary_axis(&b, [0.0, 0.0, -1.0]).expect("B about -Z");
    assert_eq!(axis(&m, "B").axis, Some([0.0, 0.0, -1.0]));
    assert_eq!(trunnion_keys(&m, "B"), before);
    assert_eq!(
        m.set_rotary_axis(&b, [0.0, 0.0, 0.0]),
        Err("axis B: the axis direction is zero".to_owned())
    );
    assert_eq!(
        m.set_rotary_axis(&b, [f64::NAN, 0.0, 1.0]),
        Err("axis B: the axis direction must be three finite numbers".to_owned())
    );
    assert_eq!(axis(&m, "B").axis, Some([0.0, 0.0, -1.0]), "a refused axis changes nothing");
}

#[test]
fn the_letter_follows_the_axis_by_rs274() {
    use camcraft_app::machine::chain::suggested_letter;
    assert_eq!(suggested_letter([1.0, 0.0, 0.0]), Some("A"));
    assert_eq!(suggested_letter([-1.0, 0.0, 0.0]), Some("A"));
    assert_eq!(suggested_letter([0.0, -2.0, 0.0]), Some("B"));
    assert_eq!(suggested_letter([0.0, 0.0, 1.0]), Some("C"));
    assert_eq!(suggested_letter([1.0, 1.0, 0.0]), None, "no letter for a tilted axis");
    assert_eq!(suggested_letter([0.0, 0.0, 0.0]), None);
}

#[test]
fn a_row_whose_letter_and_axis_disagree_says_so() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::OnTable, "A", [0.0, 1.0, 0.0]).expect("an A about Y is allowed");
    let a = m.axis_rows().into_iter().find(|r| r.name == "A").expect("A row");
    assert_eq!(a.axis, Some([0.0, 1.0, 0.0]));
    assert_eq!(a.letter_note().as_deref(), Some("A turns about Y: by RS-274 an axis about Y is B"));
    let a_ptr = a.pointer.clone();
    m.set_rotary_axis(&a_ptr, [-1.0, 0.0, 0.0]).expect("A about -X");
    let a = m.axis_rows().into_iter().find(|r| r.name == "A").expect("A row");
    assert_eq!(a.letter_note(), None);
    let x = m.axis_rows().into_iter().find(|r| r.name == "X").expect("X row");
    assert_eq!((x.axis, x.letter_note()), (None, None));
}
