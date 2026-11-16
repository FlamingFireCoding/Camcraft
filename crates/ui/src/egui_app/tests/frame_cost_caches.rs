//! #2247-#2250: the caches behind the frame-cost pieces of the #2238
//! probe. Each cache gets the rule's two pins (`docs/decisions/
//! 2238-frame-cost-rule.md` rule 5): a still frame makes no new compute
//! (a counter, never a time), and a changed input - and its undo - gives
//! the answer a fresh compute gives.
//! Provenance: our own frame-cost law over our own shell; no vendor
//! material consulted.

use camcraft_app::geometry::feature;
use camcraft_app::operation::GeoSel;

use super::frame_cost_probe::{ADAPTIVE, adaptive_preview, fig8, run, top_chains};
use super::*;

fn chips(h: &Harness<'static, ShellUi>) -> Vec<(String, bool)> {
    h.state().state.geo_chips().into_iter().map(|c| (c.label, c.missing)).collect()
}

fn align(h: &Harness<'static, ShellUi>) -> String {
    let st = &h.state().state;
    format!("{:?}", camcraft_app::pick::align_321(&st.world, &st.op_ui.geo))
}

fn set_picks(h: &mut Harness<'static, ShellUi>, items: Vec<GeoSel>) {
    run(h, |st| st.pending.push_back(CommandKind::SetOperationGeometry { name: ADAPTIVE.into(), items }));
}

/// #2247: the welded-face grouping behind the chips, the face-origin
/// targets and the 3-2-1 check is made once per feature set.
#[test]
fn pick_chips_group_the_welded_faces_once_and_follow_the_picks() {
    let mut h = fig8();
    adaptive_preview(&mut h);
    let (chips0, align0) = (chips(&h), align(&h));
    assert_eq!(chips0.len(), 1, "one welded-face chip: {chips0:?}");

    // The memo answers what a fresh grouping answers, on fig8 itself.
    let f = {
        let st = &h.state().state;
        model::find_by_name(&st.world, "fig8").and_then(|e| model::features_of(&st.world, &e)).expect("features")
    };
    let memo: Vec<_> = model::patches(&f).iter().map(|p| (p.id, p.faces.clone())).collect();
    let fresh: Vec<_> = feature::patches(&f).into_iter().map(|p| (p.id, p.faces)).collect();
    assert_eq!(memo, fresh, "a hit is the fresh grouping");

    // Still frames group nothing.
    let before = model::patch_builds();
    h.run_steps(3);
    assert_eq!(model::patch_builds() - before, 0, "a still frame groups no welded face");
    assert_eq!(chips(&h), chips0);

    // A new pick is a new chip list; the welded-face chip stays as it was.
    let mut more = h.state().state.op_ui.geo.clone();
    more.push(top_chains(&h.state().state).into_iter().next().expect("a top chain"));
    set_picks(&mut h, more);
    let chips1 = chips(&h);
    assert_eq!(chips1.len(), 2, "the chain pick adds a chip: {chips1:?}");
    assert_eq!(chips1[0], chips0[0], "the welded-face chip reads the same");

    // The undo gives the earlier answer back.
    run(&mut h, |st| st.pending.push_back(CommandKind::Undo));
    assert_eq!(chips(&h), chips0, "after the undo the chips are the first ones");
    assert_eq!(align(&h), align0, "after the undo the 3-2-1 answer is the first one");
}
