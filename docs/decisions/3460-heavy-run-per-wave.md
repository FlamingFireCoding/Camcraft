## #3460 - One heavy run for a merge wave, on the trial tree of the wave

**The maintainer's words** (issue #3460, 2026-11-22, given in the
question dialog):

> A wave of PRs that the planner groups together gets ONE heavy run on a
> trial tree of the wave (current `main` plus each PR of the wave). The
> result is recorded on each PR of the wave (tree hash, lanes, counts). A
> red step is attributed to a PR, that PR leaves the wave, and the run
> starts again for the rest.
>
> Rules that stay:
> - "Not run" is never a pass.
> - A PR that merges alone keeps the rule of today.
> - The release tier at the cut stays the final check.
>
> [...] Safety-floor PRs: the wave run covers them only if the trial tree
> contains their final head.

**Decided.** AGENTS.md "Heavy lanes at review" has two forms. The PR
form is the rule of #2568: the author runs `cargo xtask gate heavy` on
the final head. The wave form is new: the merging session runs it one
time on the trial tree of wave 1, with `--covers <PR>:<head sha>` for
each PR. The tool refuses a sha that is not a commit, a head that is not
an ancestor of `HEAD`, and a working tree with changes. It prints
`covers: #<PR> <sha>` and `tree: <HEAD^{tree}>` in the plan header and
under the summary, and writes `covers.txt` (header, steps, counts, wall
time, verdict) to the log directory. The `merge-ready` skill (step 4a)
makes the trial tree, posts one `<!-- heavy-wave: <tree hash> -->`
comment on each PR of the wave, and drops a PR with a red step. A
reviewer accepts a wave record only if it names the current head of the
PR. A wave-2 PR merges alone and keeps the PR form.

**Why.** One run of the motion and app lanes takes 60 to 70 minutes, and
one heavy build runs at a time on a machine. A wave of 9 PRs took about
12 hours with one run for each PR. The trial tree contains each diff of
the wave, so one run covers each PR. The ancestor check makes the record
true: a tree that lacks a PR's final head cannot name it. The clean-tree
check makes the tree hash the tree that the run tested.

**Taken unattended.**
- The rule text says "does not merge while not run on its final head"
  in place of "not ready while not run", because the wave run comes at
  merge time. The PR form keeps "before the PR is flagged ready".
- The reviewer keeps its duty of #2568 (PR-reviewer step 5, pinned by
  `runner_review_requires_heavy_lanes_without_author_results`): with no
  author result and no wave record on the head, it runs the lanes, or
  reports "heavy lanes not run" as a blocker. A wave record that names
  the current head takes the place of the author result. A change that
  lets a review defer the lanes to the wave run is a new rule for the
  maintainer; it is not in this PR (follow-up #3467).
- The attribution of a red step: the PR whose diff the red test reads;
  when that is not clear, the last such PR in the planner's order.
- The tool does not make the trial tree and does not post the record.
  The merging session does, by the skill. This keeps `xtask` free of
  `gh` writes and of worktree side effects.
- `gate_args.rs` holds the moved `parse_gate`, so that `main.rs` stays
  under 400 lines.

**Dropped.**
- Heavy runs for each PR only (the rule of #2568 alone). It is correct,
  but it costs about 12 hours for a wave of 9 PRs.
- No heavy run before the merge, with the cut gate as the only check. A
  red heavy test would then be found only at the cut, after the merge,
  and "not run" would act as a pass.
- A trial tree built from `git merge-tree` objects only (as
  `merge-plan` does). The heavy tier diffs the working tree, so the run
  needs a checkout.

**Cost to change later.** Small. The flag is additive; without it,
`gate heavy` works as before. The comment form is a skill text.
