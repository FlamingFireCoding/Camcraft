//! The file editor's "Axis chain" group (issue #2756): one row per axis
//! part in chain order, with move up, move down and remove, a linear's
//! reversed direction (#373), a rotary's axis with a note when its
//! letter is not the RS-274 one, and a form that adds a linear or
//! rotary part on the table or under the head.
//!
//! Provenance: our own layout, specced from issue #2756; the edits live
//! in the app layer (`camcraft_app::machine::{chain, draft}`): this
//! file draws the rows and calls the draft on a click.

use egui::{DragValue, Label, RichText, TextEdit, Ui};

use camcraft_app::machine::chain::{suggested_letter, AxisRow, Place, Step};
use camcraft_app::machine::draft::MachineDraft;
use camcraft_app::machine::validate::Refusal;

use super::layout::group_open;
use super::theme::Tokens;

/// The directions a new rotary can turn about.
pub const DIRECTIONS: [(&str, [f64; 3]); 6] = [
    ("+X", [1.0, 0.0, 0.0]),
    ("-X", [-1.0, 0.0, 0.0]),
    ("+Y", [0.0, 1.0, 0.0]),
    ("-Y", [0.0, -1.0, 0.0]),
    ("+Z", [0.0, 0.0, 1.0]),
    ("-Z", [0.0, 0.0, -1.0]),
];

/// The "add axis" form's typed state.
pub struct AxisForm {
    /// The new axis's letter being typed.
    pub name: String,
    pub place: Place,
    /// An index into `DIRECTIONS`; a rotary only.
    pub direction: usize,
}

impl Default for AxisForm {
    fn default() -> Self {
        Self { name: String::new(), place: Place::OnTable, direction: 0 }
    }
}

/// What a click in the group asks of the draft.
pub(crate) enum Edit {
    Move(String, Step),
    Remove(String),
    /// A linear's `direction`: reversed (`-1`) or standard.
    Reverse(String, bool),
    /// A rotary's new `axis`.
    TurnAbout(String, [f64; 3]),
    Add,
}

/// The validator keys that belong on an axis row.
#[must_use]
pub fn row_keys(name: &str) -> [String; 2] {
    [name.to_owned(), format!("{name} axis")]
}

const TRADE_ABOVE: &str = "Trade places with the axis above";
const TRADE_BELOW: &str = "Trade places with the axis below";
const TRADE_KIND: &str = "An axis trades places only with one of its kind; to put it elsewhere, remove it and add it again";

fn row_words(r: &AxisRow) -> String {
    let kind = if r.rotary { "rotary" } else { "linear" };
    let moves = if r.carries_head { "moves the tool" } else { "moves the work" };
    format!("{} - {kind}, {moves}", r.name)
}

/// Draw the group. Returns the edit a click asks for; the caller
/// applies it after the frame's other fields (`apply`).
pub(crate) fn draw(
    ui: &mut Ui,
    draft: &MachineDraft,
    form: &mut AxisForm,
    refusals: &[Refusal],
    pal: &Tokens,
    error: egui::Color32,
) -> Option<Edit> {
    let rows = draft.axis_rows();
    let mut edit: Option<Edit> = None;
    group_open(ui, "Axis chain", None, true, |ui| {
        if rows.is_empty() {
            ui.label(RichText::new("no axis stated").color(pal.text_dim));
        }
        for (i, r) in rows.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                ui.add(Label::new(row_words(r)).wrap()).on_hover_text(format!("Part: {}", r.pointer));
                // A move trades only with a neighbour of the same kind.
                let same = |j: Option<usize>| j.and_then(|j| rows.get(j)).is_some_and(|n| n.rotary == r.rotary);
                let up = ui.add_enabled(same(i.checked_sub(1)), egui::Button::new(format!("Move {} up", r.name)));
                if up.on_hover_text(TRADE_ABOVE).on_disabled_hover_text(TRADE_KIND).clicked() {
                    edit = Some(Edit::Move(r.pointer.clone(), Step::Up));
                }
                let down = ui.add_enabled(same(Some(i + 1)), egui::Button::new(format!("Move {} down", r.name)));
                if down.on_hover_text(TRADE_BELOW).on_disabled_hover_text(TRADE_KIND).clicked() {
                    edit = Some(Edit::Move(r.pointer.clone(), Step::Down));
                }
                let remove = ui.button(format!("Remove {}", r.name));
                if remove.on_hover_text("Remove this axis; the parts it carries stay").clicked() {
                    edit = Some(Edit::Remove(r.pointer.clone()));
                }
            });
            if let Some(e) = row_fields(ui, r) {
                edit = Some(e);
            }
            for note in r.letter_note().into_iter().chain(r.offset_note()) {
                ui.add(Label::new(RichText::new(format!("Note: {note}")).color(pal.text_dim)).wrap());
            }
            let keys = row_keys(&r.name);
            for refusal in refusals.iter().filter(|x| keys.contains(&x.key)) {
                super::error_text::error_text(ui, error, &refusal.to_string());
            }
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let l = ui.label("New axis");
            let hint = DIRECTIONS.get(form.direction).and_then(|d| suggested_letter(d.1)).unwrap_or("A");
            ui.add(TextEdit::singleline(&mut form.name).id_salt("machine-axis-new").desired_width(40.0).hint_text(hint))
                .labelled_by(l.id)
                .on_hover_text("X, Y, Z, U, V or W for a linear; A, B or C for a rotary");
            ui.selectable_value(&mut form.place, Place::OnTable, "On the table")
                .on_hover_text("Beside the table's axes: a rotary here turns the work");
            ui.selectable_value(&mut form.place, Place::CarriesHead, "Carries the head")
                .on_hover_text("The head rides on the new axis: a rotary here tilts the tool");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Rotary turns about");
            for (i, (words, axis)) in DIRECTIONS.iter().enumerate() {
                if ui.selectable_value(&mut form.direction, i, *words).clicked() {
                    suggest(form, *axis);
                }
            }
        });
        if ui.button("Add axis").on_hover_text("Add the axis to the chain").clicked() {
            edit = Some(Edit::Add);
        }
    });
    edit
}

/// A row's own fields: a linear's reversed box, a rotary's axis as
/// ±X/±Y/±Z or three numbers.
fn row_fields(ui: &mut Ui, r: &AxisRow) -> Option<Edit> {
    let mut edit = None;
    ui.horizontal_wrapped(|ui| {
        if !r.rotary {
            let mut reversed = r.reversed;
            let b = ui.checkbox(&mut reversed, format!("{} reversed", r.name));
            if b.on_hover_text("direction -1: this slide's + moves the other way").changed() {
                edit = Some(Edit::Reverse(r.pointer.clone(), reversed));
            }
            return;
        }
        ui.label(format!("{} turns about", r.name));
        for (words, axis) in DIRECTIONS {
            let on = r.axis.is_some_and(|a| unit(a) == Some(axis));
            if ui.selectable_label(on, format!("{} about {words}", r.name)).clicked() && !on {
                edit = Some(Edit::TurnAbout(r.pointer.clone(), axis));
            }
        }
        let mut a = r.axis.unwrap_or([0.0; 3]);
        let mut changed = false;
        for (c, word) in a.iter_mut().zip(["x", "y", "z"]) {
            // Keyed by the part and component, so a note or refusal row
            // that comes or goes above does not take the focus mid-edit.
            let id = ("machine-axis-dir", &r.pointer, word);
            let d = ui.push_id(id, |ui| ui.add(DragValue::new(c).speed(0.01).max_decimals(4))).inner;
            changed |= d.on_hover_text(format!("{} axis {word}: any direction, not only ±X, ±Y, ±Z", r.name)).changed();
        }
        if changed {
            edit = Some(Edit::TurnAbout(r.pointer.clone(), a));
        }
    });
    edit
}

/// `axis` scaled to length one, if it has a length.
fn unit(axis: [f64; 3]) -> Option<[f64; 3]> {
    let len = axis.iter().map(|x| x * x).sum::<f64>().sqrt();
    (len.is_finite() && len > 0.0).then(|| axis.map(|x| x / len))
}

/// Picking the new rotary's direction suggests its RS-274 letter, unless
/// the user typed a linear letter.
fn suggest(form: &mut AxisForm, axis: [f64; 3]) {
    let typed = form.name.trim().to_ascii_uppercase();
    if let Some(l) = suggested_letter(axis)
        && matches!(typed.as_str(), "" | "A" | "B" | "C")
    {
        l.clone_into(&mut form.name);
    }
}

/// Apply one edit to the draft. A refusal changes nothing and comes
/// back by name for the status line.
pub(crate) fn apply(draft: &mut MachineDraft, form: &mut AxisForm, edit: Edit) -> Result<(), String> {
    match edit {
        Edit::Move(at, step) => draft.move_axis(&at, step).map(|_| ()),
        Edit::Remove(at) => draft.remove_axis(&at),
        Edit::Reverse(at, reversed) => draft.set_axis_reversed(&at, reversed),
        Edit::TurnAbout(at, axis) => draft.set_rotary_axis(&at, axis),
        Edit::Add => {
            let name = form.name.trim().to_ascii_uppercase();
            let r = if matches!(name.as_str(), "A" | "B" | "C") {
                let axis = DIRECTIONS.get(form.direction).map_or([1.0, 0.0, 0.0], |d| d.1);
                draft.add_rotary_axis(form.place, &name, axis)
            } else {
                draft.add_linear_axis(form.place, &name)
            };
            r.map(|_| form.name.clear())
        }
    }
}
