//! #1122: form-seam guard tests — counts raw egui control calls per
//! page family so migrated families stay at zero and unmigrated ones do
//! not grow new raw calls.
//! Provenance: our own source-level guard over `crates/ui/src/egui_app`.

const RAW_PATTERNS: &[&str] = &[
    "ui.button(",
    "ui.small_button(",
    "ui.checkbox(",
    "ui.label(",
    "ui.add(",
    "ui.add_enabled(",
    "ui.add_sized(",
    "ui.radio_value(",
];

fn count_raw_calls(src: &str) -> usize {
    src.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| {
            RAW_PATTERNS
                .iter()
                .map(|pat| line.matches(pat).count())
                .sum::<usize>()
        })
        .sum()
}

struct Family<'a> {
    name: &'a str,
    max_raw: usize,
    sources: &'a [(&'a str, &'a str)],
}

const FAMILIES: &[Family<'_>] = &[
    Family {
        name: "holes",
        max_raw: 0,
        sources: &[
            ("holes_page.rs", include_str!("../holes_page.rs")),
            ("thread_page.rs", include_str!("../thread_page.rs")),
        ],
    },
    Family {
        name: "2D",
        max_raw: 4,
        sources: &[
            ("mill2d_page.rs", include_str!("../mill2d_page.rs")),
            ("mill2d_more_page.rs", include_str!("../mill2d_more_page.rs")),
            ("trochoid_page.rs", include_str!("../trochoid_page.rs")),
            ("pockets_page.rs", include_str!("../pockets_page.rs")),
            ("plasma_page.rs", include_str!("../plasma_page.rs")),
        ],
    },
    Family {
        name: "3D",
        max_raw: 42,
        sources: &[
            ("mill3d_page.rs", include_str!("../mill3d_page.rs")),
            ("approach_page.rs", include_str!("../approach_page.rs")),
            ("lead_page.rs", include_str!("../lead_page.rs")),
            ("heights_page.rs", include_str!("../heights_page.rs")),
            ("feeds_page.rs", include_str!("../feeds_page.rs")),
            ("geo_page.rs", include_str!("../geo_page.rs")),
            ("boundary.rs", include_str!("../boundary.rs")),
        ],
    },
    Family {
        name: "turning",
        max_raw: 3,
        sources: &[
            ("turning_page.rs", include_str!("../turning_page.rs")),
            ("turning_more_page.rs", include_str!("../turning_more_page.rs")),
            ("thread_infeed_page.rs", include_str!("../thread_infeed_page.rs")),
        ],
    },
    Family {
        name: "multi-axis",
        max_raw: 16,
        sources: &[
            ("multiaxis_page.rs", include_str!("../multiaxis_page.rs")),
            ("rotary_page.rs", include_str!("../rotary_page.rs")),
            ("probe_page.rs", include_str!("../probe_page.rs")),
            ("wire_page.rs", include_str!("../wire_page.rs")),
            ("sinker_page.rs", include_str!("../sinker_page.rs")),
        ],
    },
    Family {
        name: "setup and fixtures",
        max_raw: 245,
        sources: &[
            ("setup_pages.rs", include_str!("../setup_pages.rs")),
            ("setup_settings_page.rs", include_str!("../setup_settings_page.rs")),
            ("setup_post_page.rs", include_str!("../setup_post_page.rs")),
            ("setup_rotary_page.rs", include_str!("../setup_rotary_page.rs")),
            ("setup_wcs_pick.rs", include_str!("../setup_wcs_pick.rs")),
            ("setup_align_pick.rs", include_str!("../setup_align_pick.rs")),
            ("setup_axis_pick.rs", include_str!("../setup_axis_pick.rs")),
            ("setup_orient_pick.rs", include_str!("../setup_orient_pick.rs")),
            ("setup_plane_pick.rs", include_str!("../setup_plane_pick.rs")),
            ("setup_points_pick.rs", include_str!("../setup_points_pick.rs")),
            ("setup_relative_pick.rs", include_str!("../setup_relative_pick.rs")),
            ("setup_stock_pick.rs", include_str!("../setup_stock_pick.rs")),
            ("setup_workholding_row.rs", include_str!("../setup_workholding_row.rs")),
            ("stock_page.rs", include_str!("../stock_page.rs")),
            ("raw_stock_row.rs", include_str!("../raw_stock_row.rs")),
            ("wcs_page.rs", include_str!("../wcs_page.rs")),
            ("wcs_pick_row.rs", include_str!("../wcs_pick_row.rs")),
            ("fixtures_page.rs", include_str!("../fixtures_page.rs")),
            ("fixture_rows.rs", include_str!("../fixture_rows.rs")),
            ("fixture_plate_rows.rs", include_str!("../fixture_plate_rows.rs")),
            ("fixture_modvise_rows.rs", include_str!("../fixture_modvise_rows.rs")),
            ("fixture_accessory_rows.rs", include_str!("../fixture_accessory_rows.rs")),
            ("fixture_plate_dialog.rs", include_str!("../fixture_plate_dialog.rs")),
            ("fixture_vise_dialog.rs", include_str!("../fixture_vise_dialog.rs")),
            ("fixture_accessory_dialog.rs", include_str!("../fixture_accessory_dialog.rs")),
            ("fixture_library_dialog.rs", include_str!("../fixture_library_dialog.rs")),
            ("locate_form.rs", include_str!("../locate_form.rs")),
        ],
    },
    Family {
        name: "tools and machine",
        max_raw: 204,
        sources: &[
            ("tool_list_page.rs", include_str!("../tool_list_page.rs")),
            ("tool_cutter_page.rs", include_str!("../tool_cutter_page.rs")),
            ("tool_cutter_params.rs", include_str!("../tool_cutter_params.rs")),
            ("tool_body_page.rs", include_str!("../tool_body_page.rs")),
            ("tool_body_lathe_page.rs", include_str!("../tool_body_lathe_page.rs")),
            ("tool_profile_page.rs", include_str!("../tool_profile_page.rs")),
            ("tool_holder_picker.rs", include_str!("../tool_holder_picker.rs")),
            ("tool_catalog_rows.rs", include_str!("../tool_catalog_rows.rs")),
            ("tool_cutting_rows.rs", include_str!("../tool_cutting_rows.rs")),
            ("tool_orientation_row.rs", include_str!("../tool_orientation_row.rs")),
            ("tool_thread_mill_row.rs", include_str!("../tool_thread_mill_row.rs")),
            ("tool_turning_row.rs", include_str!("../tool_turning_row.rs")),
            ("machine_page.rs", include_str!("../machine_page.rs")),
            ("machine_editor.rs", include_str!("../machine_editor.rs")),
            ("machine_edm.rs", include_str!("../machine_edm.rs")),
            ("machine_edm_editor.rs", include_str!("../machine_edm_editor.rs")),
            ("machine_file_editor.rs", include_str!("../machine_file_editor.rs")),
            ("machine_grid.rs", include_str!("../machine_grid.rs")),
            ("machines_pane.rs", include_str!("../machines_pane.rs")),
            ("spindle_row.rs", include_str!("../spindle_row.rs")),
        ],
    },
];

#[test]
fn pages_do_not_add_raw_egui_calls_outside_the_form_seam() {
    let mut violations = Vec::new();
    for fam in FAMILIES {
        let mut total = 0;
        let mut per_file = Vec::new();
        for (name, src) in fam.sources {
            let n = count_raw_calls(src);
            if n > 0 {
                per_file.push(format!("{name}={n}"));
            }
            total += n;
        }
        if total > fam.max_raw {
            violations.push(format!(
                "family '{}' has {total} raw ui calls (limit {}): {}",
                fam.name,
                fam.max_raw,
                per_file.join(", ")
            ));
        }
    }
    assert!(violations.is_empty(), "raw egui calls bypass the form seam:\n{}", violations.join("\n"));
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Choice {
    First,
    Second,
}

struct SeamProbe {
    hits: usize,
    small_hits: usize,
    choice: Choice,
    count: u32,
}

impl Default for SeamProbe {
    fn default() -> Self {
        Self { hits: 0, small_hits: 0, choice: Choice::Second, count: 1 }
    }
}

#[test]
fn form_seam_controls_preserve_labels_clicks_and_accessible_names() {
    use egui::accesskit::Role;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};

    let mut h = Harness::builder().with_size(egui::Vec2::new(480.0, 260.0)).build_ui_state(
        |ui, p: &mut SeamProbe| {
            super::list_row(ui, "1  X 10.000  Y 20.000");
            if super::small_button(ui, "Remove 1").clicked() {
                p.small_hits += 1;
            }
            if super::button(ui, "Detect holes").clicked() {
                p.hits += 1;
            }
            super::prop_row(ui, "Hand", |ui| {
                super::radio_pair(ui, &mut p.choice, (Choice::First, "Left hand"), (Choice::Second, "Right hand"));
            });
            super::prop_row(ui, "Spring passes", |ui| {
                super::super::fields::int_range_row(ui, &mut p.count, 0..=4);
            });
        },
        SeamProbe::default(),
    );
    h.run();

    assert!(h.query_by_label("1  X 10.000  Y 20.000").is_some());
    assert!(h.query_by_label("Left hand").is_some() && h.query_by_label("Right hand").is_some());
    let spin_named = h.root().children_recursive().any(|n| {
        let node = n.accesskit_node();
        node.role() == Role::SpinButton && node.labelled_by().next().is_some()
    });
    assert!(spin_named, "int_range_row inside prop_row must carry labelled_by");

    h.get_by_label("Detect holes").click();
    h.get_by_label("Remove 1").click();
    h.get_by_label("Left hand").click();
    h.run();

    assert_eq!(h.state().hits, 1);
    assert_eq!(h.state().small_hits, 1);
    assert_eq!(h.state().choice, Choice::First);
}

