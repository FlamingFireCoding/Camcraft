//! The Report door (#1696): one place to file a bug or ask for a feature.
//! It opens the repository's issue form in the browser with the title,
//! the body and the environment filled in, and it puts a picture of the
//! current document on the clipboard so the user can paste it in.
//!
//! Provenance: our own. GitHub's prefill query (`issues/new?title=…&body=
//! …&labels=…`) is its documented new-issue URL form; it applies only to a
//! blank issue template, so the repository also gained `blank-issue.md`.
//! The picture is the CPU raster (`camcraft_app::raster`) over the frame's
//! own scene, the same path the MCP `render_stock` picture uses, with the
//! viewport floor's grid law drawn for the picture's own camera (#1696).
//! No vendor material consulted.

use egui::{Button, ColorImage, Context, Id, Image, Response, TextEdit, Ui, Vec2, Window};

use camcraft_app::geometry::Point3;
use camcraft_app::raster::{Camera, Canvas, ViewDir};

use super::icons::{Icon, IconSet};
use super::world;
use crate::interact;
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

/// The shot's ground: a light floor, readable in an issue. The grid
/// fades toward it at its rim, the viewport floor's law.
const SHOT_BG: [u8; 3] = [0xF2, 0xF2, 0xF2];

/// The floor's minor / major ink: a step off the ground so the floor
/// reads as a surface without competing with the part - the viewport's
/// `interact::GRID_MINOR`/`GRID_MAJOR` law mirrored for a light ground.
const GRID_MINOR_LIGHT: [f64; 3] = [218.0, 218.0, 222.0];
const GRID_MAJOR_LIGHT: [f64; 3] = [198.0, 198.0, 204.0];

/// The floor's wanted screen spacing, in pixels: the lines must read
/// without turning the picture into a mesh.
const SHOT_GRID_TARGET_PX: f64 = 40.0;

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

/// The floor's ink for the line at index `k` on one axis, `centre` the
/// line nearest the scene's centre: major on every fifth line, faded
/// toward the ground at the rim (cubed falloff) - the viewport grid's
/// `grid_ink` law with light-ground values.
fn grid_ink(k: i64, centre: i64) -> [u8; 3] {
    let base = if k.rem_euclid(interact::GRID_MAJOR_EVERY) == 0 { GRID_MAJOR_LIGHT } else { GRID_MINOR_LIGHT };
    let d = (k - centre).unsigned_abs() as f64 / interact::GRID_HALF_STEPS as f64;
    let fade = (1.0 - d * d * d).clamp(0.0, 1.0);
    let mut out = [0u8; 3];
    for (o, b) in out.iter_mut().zip(base) {
        *o = (f64::from(SHOT_BG[0]) + (b - f64::from(SHOT_BG[0])) * fade).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// The shot's ground grid (#1696, the maintainer's GUI finding): the
/// viewport floor's law - 1-2-5 steps snapped to round machine
/// coordinates, a major line on every fifth, faded at the rim, drawn
/// under the part so the part occludes it like a real table - but built
/// for the shot's own fitted camera, not the live one. The viewport's
/// grid (`interact::ground_grid_lines`) is spaced for the user's zoom
/// and centred on his camera target: through a fitted camera that
/// lands at the wrong scale, and the maintainer saw exactly that.
fn paint_grid(canvas: &mut Canvas, cam: &Camera, cx: f64, cy: f64) {
    if !(cx.is_finite() && cy.is_finite()) {
        return;
    }
    let step = interact::nice_step(SHOT_GRID_TARGET_PX / cam.scale());
    let reach = step * interact::GRID_HALF_STEPS as f64;
    // The same hang guard as the viewport grid: a centre far from the
    // origin at a fine step would stretch the line index range toward
    // the whole of i64. The floor is furniture; it stays out of that
    // frame (Prime Directive: no hang).
    if !(step.is_finite() && step > 0.0)
        || (cx / step).abs() > 1.0e12
        || (cy / step).abs() > 1.0e12
        || !reach.is_finite()
    {
        return;
    }
    // The line nearest the centre is the fade's centre; lines land on
    // round machine coordinates, so a re-framed camera keeps landing on
    // 0, 5, 10 ... on the major lines.
    let (cix, ciy) = ((cx / step).round() as i64, (cy / step).round() as i64);
    let k0x = f64::floor((cx - reach) / step) as i64;
    let k1x = f64::ceil((cx + reach) / step) as i64;
    let k0y = f64::floor((cy - reach) / step) as i64;
    let k1y = f64::ceil((cy + reach) / step) as i64;
    let at = |x: f64, y: f64| Point3::new(x, y, 0.0);
    // Lines of constant y run along X, and the other way round.
    // k stays inside centre +/- (GRID_HALF_STEPS + 1), and the guard
    // above caps centre/step at 1e12: k as f64 is exact below 2^53.
    for k in k0y..=k1y {
        canvas.ground_line(at(cx - reach, k as f64 * step), at(cx + reach, k as f64 * step), grid_ink(k, ciy));
    }
    for k in k0x..=k1x {
        canvas.ground_line(at(k as f64 * step, cy - reach), at(k as f64 * step, cy + reach), grid_ink(k, cix));
    }
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
    let mut canvas = Canvas::new(cam, SHOT_BG);
    // The floor first, the part over it: `ground_line` writes and tests
    // depth, so the part hides the floor and the floor can never paint
    // over the part (#1696). The same switch as the viewport: hiding the
    // world axes hides the floor too.
    if state.show_world_axes {
        let (mut x0, mut x1, mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for v in solid.iter().flat_map(|p| p.verts.iter()) {
            if v[0].is_finite() && v[1].is_finite() {
                x0 = x0.min(f64::from(v[0]));
                x1 = x1.max(f64::from(v[0]));
                y0 = y0.min(f64::from(v[1]));
                y1 = y1.max(f64::from(v[1]));
            }
        }
        if x0.is_finite() && y0.is_finite() {
            paint_grid(&mut canvas, &cam, (x0 + x1) * 0.5, (y0 + y1) * 0.5);
        }
    }
    for part in &solid {
        // The model reads grey, the stock steel-blue - the viewport's
        // two solid roles, flat-shaded by the camera's one light.
        let base = if part.stock { [0x9A, 0xA8, 0xB8] } else { [0xC8, 0xC8, 0xC8] };
        for tri in part.verts.as_chunks::<3>().0 {
            canvas.triangle(to_pt(&tri[0]), to_pt(&tri[1]), to_pt(&tri[2]), base);
        }
    }
    // The eye's overlay - the toolpath, the selection draft - over the
    // part: never lost behind it, the raster's overlay law. The ground
    // grid is not in this list: paint_grid already drew the floor for
    // this picture's own camera, and `collect(state)`'s copy of it was
    // framed for the live camera.
    for line in &interact::capped(interact::collect_overlay(state)) {
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
