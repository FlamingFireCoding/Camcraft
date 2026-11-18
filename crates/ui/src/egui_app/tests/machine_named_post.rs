//! #2374: a machine whose file names its post (`controller.post`) is
//! offered that post on the setup's Post page, in the same banner as the
//! kind offer and through the same `SelectPost`, so undo puts the
//! previous post back. The mount itself never changes the post.
//! Provenance: our own test over our own shipped machine file.
use super::*;
use camcraft_app::posting;
use camcraft_app::schema::PostRef;

const LINE: &str = "the machine names grblHAL as its post; switch to it?";

fn post(harness: &Harness<'static, ShellUi>) -> PostRef {
    posting::post_of(&harness.state().state.world)
}

#[test]
fn the_grblhal_machine_offers_its_own_post_and_the_switch_rides_undo() {
    let mut harness = shell_harness();
    let pending = |h: &mut Harness<'static, ShellUi>, c: CommandKind| h.state_mut().state.pending.push_back(c);
    pending(&mut harness, CommandKind::CreateSetup { name: None, datum_from: None });
    harness.run_steps(2);
    crate::actions::target(
        &mut harness.state_mut().state,
        UiTarget::SetupNode("Setup1".into(), crate::state::SetupPage::Post),
    );
    harness.run();
    assert!(harness.query_by_label_contains(LINE).is_none(), "no machine named yet - no offer");
    pending(&mut harness, CommandKind::LoadBuiltinMachine("grblHAL 3-axis".to_owned()));
    harness.run();
    harness.run();
    assert_eq!(post(&harness), PostRef::Builtin, "the mount never re-points the post by itself");
    assert!(
        harness.query_by_label_contains(LINE).is_some(),
        "the offer names the machine's post (status: {})",
        harness.state().state.status
    );
    harness.get_by_label("Switch post").scroll_to_me();
    harness.run();
    harness.get_by_label("Switch post").click();
    harness.run();
    harness.run();
    assert_eq!(post(&harness), PostRef::Named("grblHAL".to_owned()), "taken through SelectPost");
    assert!(harness.query_by_label_contains(LINE).is_none(), "the machine's post retires the offer");
    pending(&mut harness, CommandKind::Undo);
    harness.run();
    harness.run();
    assert_eq!(post(&harness), PostRef::Builtin, "undo restores Fanuc");
    assert!(harness.query_by_label_contains(LINE).is_some(), "and the offer stands again");
}
