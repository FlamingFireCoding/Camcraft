//! The file editor's "Axis chain" group (issue #2756): one row per axis
//! part in chain order, with move up, move down and remove, and a form
//! that adds a linear or rotary part on the table or under the head.
//!
//! Provenance: our own layout, specced from issue #2756; the edits live
//! in the app layer (`camcraft_app::machine::{chain, draft}`): this
//! file draws the rows and calls the draft on a click.

use egui::{Label, RichText, TextEdit, Ui};

use camcraft_app::machine::chain::{AxisRow, Place, Step};
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
    Add,
}

/// The validator keys that belong on an axis row.
#[must_use]
pub fn row_keys(name: &str) -> [String; 2] {
    [name.to_owned(), format!("{name} axis")]
}

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
                let up = ui.add_enabled(i > 0, egui::Button::new(format!("Move {} up", r.name)));
                if up.on_hover_text("Trade places with the axis above").clicked() {
                    edit = Some(Edit::Move(r.pointer.clone(), Step::Up));
                }
                let down = ui.add_enabled(i + 1 < rows.len(), egui::Button::new(format!("Move {} down", r.name)));
                if down.on_hover_text("Trade places with the axis below").clicked() {
                    edit = Some(Edit::Move(r.pointer.clone(), Step::Down));
                }
                let remove = ui.button(format!("Remove {}", r.name));
                if remove.on_hover_text("Remove this axis; the parts it carries stay").clicked() {
                    edit = Some(Edit::Remove(r.pointer.clone()));
                }
            });
            let keys = row_keys(&r.name);
            for refusal in refusals.iter().filter(|x| keys.contains(&x.key)) {
                super::error_text::error_text(ui, error, &refusal.to_string());
            }
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let l = ui.label("New axis");
            ui.add(TextEdit::singleline(&mut form.name).desired_width(40.0).hint_text("A"))
                .labelled_by(l.id)
                .on_hover_text("X, Y, Z, U, V or W for a linear; A, B or C for a rotary");
            ui.selectable_value(&mut form.place, Place::OnTable, "On the table")
                .on_hover_text("Beside the table's axes: a rotary here turns the work");
            ui.selectable_value(&mut form.place, Place::CarriesHead, "Carries the head")
                .on_hover_text("The head rides on the new axis: a rotary here tilts the tool");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Rotary turns about");
            for (i, (words, _)) in DIRECTIONS.iter().enumerate() {
                ui.selectable_value(&mut form.direction, i, *words);
            }
        });
        if ui.button("Add axis").on_hover_text("Add the axis to the chain").clicked() {
            edit = Some(Edit::Add);
        }
    });
    edit
}

/// Apply one edit to the draft. A refusal changes nothing and comes
/// back by name for the status line.
pub(crate) fn apply(draft: &mut MachineDraft, form: &mut AxisForm, edit: Edit) -> Result<(), String> {
    match edit {
        Edit::Move(at, step) => draft.move_axis(&at, step).map(|_| ()),
        Edit::Remove(at) => draft.remove_axis(&at),
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
