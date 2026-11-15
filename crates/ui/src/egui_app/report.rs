//! The Report door (#1696): one place to file a bug or ask for a feature.
//! It opens the repository's issue form in the browser with the title,
//! the body and the environment filled in, and it puts a picture of the
//! current document on the clipboard so the user can paste it in.
//!
//! Provenance: our own. GitHub's prefill query (`issues/new?title=…&body=
//! …&labels=…`) is its documented new-issue URL form; it applies only to a
//! blank issue template, so the repository also gained `blank-issue.md`.
//! The picture is the CPU raster (`camcraft_app::raster`) over the frame's
//! own scene, the same path the MCP `render_stock` picture uses. No vendor
//! material consulted.

use egui::{Button, ColorImage, Context, Id, Image, Response, TextEdit, Ui, Vec2, Window};

use camcraft_app::geometry::Point3;
use camcraft_app::raster::{Camera, Canvas, ViewDir};

use super::icons::{Icon, IconSet};
use super::world;
use crate::state::State;

/// The upstream repository's new-issue URL. The report always goes to
/// `NeoTech/Camcraft`, the same place the docs site points at.
pub(crate) const ISSUE_NEW_URL: &str = "https://github.com/NeoTech/Camcraft/issues/new";

/// The template that takes the prefill; GitHub's issue FORMS (`*.yml`)
/// ignore the `body`/`title` query, only a blank template takes it.
pub(crate) const BLANK_TEMPLATE: &str = "blank-issue.md";

/// The picture's size: readable in an issue, inside the raster's
/// accepted edges (16..=2048), quick to paint on a slow machine.
const SHOT_EDGE: [u32; 2] = [800, 600];

/// What the user files: a defect, or an enhancement. The two GitHub
/// labels are the repository's existing `bug` and `enhancement`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReportKind {
    Bug,
    Feature,
}

impl ReportKind {
    /// The window's button for this kind.
    #[must_use]
    pub(crate) fn button(self) -> &'static str {
        match self {
            Self::Bug => "Report a bug",
            Self::Feature => "Request a feature",
        }
    }

    /// The label GitHub puts on the new issue.
    #[must_use]
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Feature => "enhancement",
        }
    }
}

/// Percent-encode one query value. Only ASCII unreserved bytes pass
/// through; a space takes `+`, as GitHub's own query forms read.
pub(crate) fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The prefilled new-issue URL for one kind of report. GitHub opens the
/// blank template with the title and body already typed and the right
/// label ticked; the user adds the story and pastes the picture.
#[must_use]
pub(crate) fn issue_url(kind: ReportKind, title: &str, body: &str) -> String {
    format!(
        "{ISSUE_NEW_URL}?template={BLANK_TEMPLATE}&title={}&body={}&labels={}",
        url_encode(title),
        url_encode(body),
        url_encode(kind.label()),
    )
}

/// The body: the two questions a triage needs answered, then the plain
/// text the Debug info window reports (#157) - same fields, one source.
#[must_use]
pub(crate) fn report_body(state: &State) -> String {
    let mut out = String::new();
    out.push_str("<!-- CamCraft filled in the title, this text and the label. -->\n\n");
    out.push_str("**What happened**\n\n\n\n**Steps to reproduce**\n\n1. \n\n");
    out.push_str("---\n\n");
    out.push_str(&super::debug_info::report(state));
    out.push_str("\n\nScreenshot: press \"Copy screenshot\" in the app, then paste it into this description.");
    out
}

/// Paint the frame's own scene through the CPU raster. `None` when the
/// document has no triangles to frame (an empty scene fits no camera).
fn raster(state: &State) -> Option<Canvas> {
    let scene = world::collect(state);
    let solid: Vec<_> = scene.parts.iter().filter(|p| !p.fill).collect();
    let to_pt = |v: &[f32; 3]| Point3::new(f64::from(v[0]), f64::from(v[1]), f64::from(v[2]));
    let cam = Camera::fit(
        ViewDir::Iso,
        solid.iter().flat_map(|p| p.verts.iter()).map(to_pt),
        SHOT_EDGE[0],
        SHOT_EDGE[1],
    )?;
    let mut canvas = Canvas::new(cam, [0xF2, 0xF2, 0xF2]);
    for part in &solid {
        // The model reads grey, the stock steel-blue - the viewport's
        // two solid roles, flat-shaded by the camera's one light.
        let base = if part.stock { [0x9A, 0xA8, 0xB8] } else { [0xC8, 0xC8, 0xC8] };
        for tri in part.verts.as_chunks::<3>().0 {
            canvas.triangle(to_pt(&tri[0]), to_pt(&tri[1]), to_pt(&tri[2]), base);
        }
    }
    for line in &scene.overlay {
        canvas.overlay_line(
            Point3::new(f64::from(line[0]), f64::from(line[1]), f64::from(line[2])),
            Point3::new(f64::from(line[3]), f64::from(line[4]), f64::from(line[5])),
            [0x20, 0x20, 0x20],
        );
    }
    Some(canvas)
}

/// The document as an image for the clipboard, so the user pastes it
/// straight into the issue box (GitHub uploads a pasted picture).
pub(crate) fn screenshot_image(state: &State) -> Option<ColorImage> {
    let canvas = raster(state)?;
    Some(ColorImage::from_rgb(
        [usize::try_from(SHOT_EDGE[0]).ok()?, usize::try_from(SHOT_EDGE[1]).ok()?],
        canvas.rgb(),
    ))
}

/// The status strip's door: the glyph when the icon set is up, the words
/// when it is not (the Debug info door's degrade law). Both answer to
/// "Report an issue" - the name the issue forms promise.
pub(super) fn button(ui: &mut Ui, icons: Option<&IconSet>) -> Response {
    let resp = match icons.and_then(|s| s.texture(Icon::Report)) {
        Some(tex) => {
            let ink = ui.visuals().text_color();
            let glyph = Image::new(tex).fit_to_exact_size(Vec2::splat(16.0)).tint(ink);
            ui.add(Button::image(glyph).frame(false)).on_hover_text("Report an issue")
        }
        None => ui.small_button("Report an issue"),
    };
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Report an issue"));
    resp
}

/// The window: a title field, the two kinds, and the screenshot copy.
/// Allocates nothing while closed; closed by default on a fresh State,
/// the shell's window law.
pub(crate) fn window(ctx: &Context, state: &mut State) {
    if !state.report_open {
        return;
    }
    let id = Id::new("report-issue");
    let folded = ctx.data(|d| d.get_temp::<bool>(id.with("collapsed"))).unwrap_or(false);
    let mut open = true;
    let mut filed: Option<ReportKind> = None;
    let mut copy_shot = false;
    Window::new("Report an issue")
        .id(id)
        .title_bar(false)
        .resizable(!folded)
        .default_size([440.0, 180.0])
        .show(ctx, |ui| {
            if super::window_chrome::title_row(ui, id, "Report an issue", &mut open) {
                return;
            }
            ui.separator();
            ui.label("CamCraft fills the title, the description and the label into GitHub. Type a short title:");
            ui.add_sized(
                [ui.available_width(), 0.0],
                TextEdit::singleline(&mut state.report_title).hint_text("e.g. Contour leaves a witness line"),
            );
            ui.horizontal(|ui| {
                for kind in [ReportKind::Bug, ReportKind::Feature] {
                    if ui.button(kind.button()).clicked() {
                        filed = Some(kind);
                    }
                }
                if ui.button("Copy screenshot").clicked() {
                    copy_shot = true;
                }
            });
            ui.label("Before you file: check the open issues. A duplicate wastes both our times.");
        });
    if let Some(kind) = filed {
        let body = report_body(state);
        let url = issue_url(kind, &state.report_title, &body);
        ctx.open_url(egui::OpenUrl::new_tab(&url));
        state.status = format!("{}: camcraft filled the new issue - add the story and paste the picture", kind.button());
    }
    if copy_shot {
        match screenshot_image(state) {
            Some(img) => {
                ctx.copy_image(img);
                state.status = "screenshot copied to the clipboard - paste it into the issue description".into();
            }
            None => state.status = "no picture to copy: the document has no solid to draw".into(),
        }
    }
    state.report_open = open;
}
