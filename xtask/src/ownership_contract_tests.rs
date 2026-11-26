//! Pin the dispatch instructions, not an atomic lock. Provenance: #3661 and its maintainer follow-ups.

const WORK: &str = include_str!("../../.claude/skills/work-issue/SKILL.md");
const REVIEW: &str = include_str!("../../.claude/skills/review-prs/SKILL.md");
const REFRESH: &str = include_str!("../../.claude/skills/resolve-conflicts/SKILL.md");
const MERGE: &str = include_str!("../../.claude/skills/merge-ready/SKILL.md");

#[test]
fn ownership_dispatch_paths_load_the_shared_guard() {
    for (name, skill) in [
        ("work", WORK),
        ("review", REVIEW),
        ("refresh", REFRESH),
        ("merge", MERGE),
    ] {
        assert!(
            skill.contains("../issue-claims/references/ownership.md"),
            "{name} must load the shared public ownership procedure before dispatch"
        );
        assert!(
            words(skill).contains("before dispatch"),
            "{name} must guard dispatch"
        );
    }
}

const GUARD: &str = include_str!("../../.claude/skills/issue-claims/references/ownership.md");

fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn ownership_record_and_lifecycle_contract() {
    let text = words(GUARD);
    for required in [
        "upstream issue thread for the canonical writer record",
        "upstream PR thread for the canonical review record",
        "Owner:",
        "Dispatcher:",
        "Issue:",
        "PR:",
        "Branch:",
        "Base:",
        "Starting head:",
        "Expected remote:",
        "Started:",
        "State: active",
        "Read it back by its comment URL",
        "Re-read the threads and remote head after posting and before dispatch",
        "4 hours",
        "48-hour",
        "An ended claim stays ended",
        "Releasing writer ownership after a push does not release the issue claim",
        "Age alone does not release a running review",
        "Ownership released: <record URL>",
        "both sessions STOP until one owner explicitly releases",
    ] {
        assert!(
            text.contains(required),
            "ownership procedure must retain: {required}"
        );
    }
}

#[test]
fn ownership_stop_and_non_atomic_limits_are_explicit() {
    let text = words(GUARD);
    for required in [
        "A live writer means stand down and name the owner and record URL",
        "A live reviewer means stand down and name the owner and record URL",
        "Failed or incomplete reads mean STOP",
        "If the remote head moved, STOP",
        "A rejected push is STOP",
        "Do not merge that unexpected head and retry",
        "Never force-push, including `--force-with-lease`",
        "not an atomic distributed lock",
        "plain push rejects non-fast-forward updates",
        "can accept a fast-forward after a race",
        "They test the published procedure, not execution by an agent or distributed exclusion",
    ] {
        assert!(
            text.contains(required),
            "ownership procedure must retain: {required}"
        );
    }
    let review = words(REVIEW);
    assert!(review.contains("Keep the local `.claude/pr-review.lock`"));
    assert!(review.contains("If the head moved, stop without a current-head verdict"));
}

#[test]
fn ownership_refresh_checks_precede_push_and_preserve_merge_validation() {
    for skill in [REFRESH, MERGE, GUARD] {
        let text = words(skill);
        assert!(
            text.contains("cargo check --tests"),
            "refresh must compile affected code"
        );
        assert!(
            text.contains("before push"),
            "refresh compilation must precede publication"
        );
    }
    let guard = words(GUARD);
    assert!(guard.contains("without a build or edit between them and the push"));
    assert!(guard.contains("Any later tree change invalidates those results"));
    let merge = words(MERGE);
    assert!(merge.contains("cargo xtask merge-plan --check <N> <N> ..."));
    assert!(merge.contains("Immediately before the first merge, read them again"));
    assert!(merge.contains("Never refresh a PR that is not next in the batch"));
}
