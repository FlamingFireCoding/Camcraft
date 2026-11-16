//! Viewport highlight for "Do not touch" protected faces and edges (#2055).
//! Provenance: our own overlay projection mirroring `align_pick_viz`; no vendor material.

use std::collections::HashSet;

use camcraft_app::geometry::{Point3, feature, transform_point};
use camcraft_app::model;
use camcraft_app::operation::{self, FeatureSel};

use crate::interact::{feature_segments, project_uv};
use crate::state::State;

/// Semi-transparent fill opacity for protected faces and patches (0..255).
pub const AVOID_FACE_ALPHA: u8 = 88;
/// Stroke width (px) for protected edges.
pub const AVOID_EDGE_PX: f32 = 3.5;

/// Screen-space geometry of `"Do not touch"` items on the currently selected operation.
#[derive(Debug, Default, PartialEq)]
pub struct AvoidMarks {
    pub faces: Vec<[[f32; 2]; 3]>,
    pub edges: Vec<([f32; 2], [f32; 2])>,
}

/// Collect viewport-uv triangles and edge segments for the open operation's `"Do not touch"` list.
#[must_use]
pub fn avoid_marks(state: &State, aspect: f64) -> AvoidMarks {
    let mut out = AvoidMarks::default();
    if state.op_ui.name.is_empty()
        || !operation::do_not_touch_applies(state.op_ui.params.kind())
        || state.op_ui.area.do_not_touch.picks.is_empty()
    {
        return out;
    }
    let mvp = state.cam.mvp(aspect);
    for g in &state.op_ui.area.do_not_touch.picks {
        if state.hidden_in_view(&g.entity) {
            continue;
        }
        let Some(e) = model::find_by_name(&state.world, &g.entity) else {
            continue;
        };
        let Some(features) = model::features_of(&state.world, &e) else {
            continue;
        };
        let loc = camcraft_app::setup_align::view_pose(&state.world, &e).unwrap_or_default();
        let uv = |p: Point3| {
            let w = transform_point(p, loc.pos, loc.rot);
            project_uv(&mvp, [w.x, w.y, w.z])
        };
        match g.feature {
            FeatureSel::Face(fid) => {
                if let Some(face) = features.faces.iter().find(|f| f.id == fid)
                    && let Some(tris) = model::facets_of(&state.world, &e)
                {
                    for t in face.facets.iter().filter_map(|&i| tris.get(i as usize)) {
                        if let (Some(a), Some(b), Some(c)) = (uv(t.a), uv(t.b), uv(t.c)) {
                            out.faces.push([a, b, c]);
                        }
                    }
                }
                feature_segments(state, &g.entity, g.feature, aspect, &mut out.edges);
            }
            FeatureSel::Patch(pid) => {
                if let Some(patch) = feature::patches(&features).into_iter().find(|p| p.id == pid)
                    && let Some(tris) = model::facets_of(&state.world, &e)
                {
                    let fids: HashSet<u64> = patch.faces.into_iter().collect();
                    for face in features.faces.iter().filter(|f| fids.contains(&f.id)) {
                        for t in face.facets.iter().filter_map(|&i| tris.get(i as usize)) {
                            if let (Some(a), Some(b), Some(c)) = (uv(t.a), uv(t.b), uv(t.c)) {
                                out.faces.push([a, b, c]);
                            }
                        }
                    }
                }
                feature_segments(state, &g.entity, g.feature, aspect, &mut out.edges);
            }
            FeatureSel::Chain(_) => {
                feature_segments(state, &g.entity, g.feature, aspect, &mut out.edges);
            }
            FeatureSel::Vertex(_) => {}
        }
    }
    out
}

/// Paint protected faces and edges over the viewport image `rect`.
pub fn paint(painter: &egui::Painter, rect: egui::Rect, state: &State, aspect: f64) {
    let m = avoid_marks(state, aspect);
    if m == AvoidMarks::default() {
        return;
    }
    let to_px = |uv: [f32; 2]| egui::pos2(rect.min.x + uv[0] * rect.width(), rect.min.y + uv[1] * rect.height());
    let rgb = super::egui_app::ground_ink::overlay(super::egui_app::ground_ink::Overlay::Cut);
    let r = (rgb[0] * 255.0).clamp(0.0, 255.0) as u8;
    let g = (rgb[1] * 255.0).clamp(0.0, 255.0) as u8;
    let b = (rgb[2] * 255.0).clamp(0.0, 255.0) as u8;
    if !m.faces.is_empty() {
        let fill = egui::Color32::from_rgba_unmultiplied(r, g, b, AVOID_FACE_ALPHA);
        let mut mesh = egui::Mesh::default();
        for tri in &m.faces {
            let base = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
            for uv in tri {
                mesh.colored_vertex(to_px(*uv), fill);
            }
            mesh.add_triangle(base, base.saturating_add(1), base.saturating_add(2));
        }
        painter.add(egui::Shape::mesh(mesh));
    }
    let ink = egui::Color32::from_rgb(r, g, b);
    for (a, bb) in &m.edges {
        painter.line_segment([to_px(*a), to_px(*bb)], egui::Stroke::new(AVOID_EDGE_PX, ink));
    }
}
