---
name: start-orchestrator
description: Start the CamCraft agent team from a clean session - the coordinator (Fable), its four context scouts, the issue workers, and the link to the review session (SlaveSession). Carries the standing orders that the maintainer gave while the team ran (merge in batches without waiting for the review session, take over conflicting and idle PRs, delta review by a read-only Opus agent, ask only when two sides are exclusive). Use at the start of a session when the maintainer says "start the orchestrator", "start the team", or "work the issues", and after a restart of the main session.
---

# start-orchestrator - start the agent team

The rules are in AGENTS.md: "Autonomous Work Law", "Agent roles", "Issue Law",
"Pull Request Law". If this skill and AGENTS.md disagree, AGENTS.md is
correct. This skill has the start procedure and the standing orders.

The main session keeps the user. It does not orchestrate. It starts the
coordinator, relays, and asks the maintainer through the question dialog.

## 1. Before you start

1. `git fetch origin` and read `git log origin/main -5`. Do not pull in a
   checkout that another session uses.
2. Read `.claude/tmp/coordinator-handover.md` if it exists. It is
   machine-local and may be old. Check each fact at GitHub before you use it.
3. Check if a coordinator is already running (`ListAgents`). Never start a
   second one. Never start a coordinator while the old one is alive.
4. Check the open post-merge run (`gh run list --workflow post-merge.yml
   --limit 3`). Say if main is red and which issue tracks it.
5. Check the free disk space. One `target/` is 10-20 GB.

## 2. Start the coordinator

Start one agent of type `coordinator` (model Fable; if the model gives a 404,
start it with `model: opus` and say so). Its brief has:

- the handover file path, if it exists, and the line "check each fact at
  GitHub";
- the build cap: at most 3 local builds at `-j 6`, the review session's
  builds included;
- the standing orders of section 4;
- the rule that a maintainer decision goes to the main session as one line
  that starts with `ASK:`, and that the coordinator continues with other work.

The coordinator starts its own four `context-scout` agents (`scout-1` to
`scout-4`, Haiku) and its own workers. It cannot message the review session.
The main session relays between them.

## 3. The review session (SlaveSession)

SlaveSession is a separate local session. The maintainer starts it. This
skill cannot start it. It posts reviews, sets review labels and watches the
post-merge runs. It sends lines: `REVIEWED`, `RUN`, `TRIAGED`, `BLOCKED`.

- Relay each of its lines to the coordinator. Check each review id and head
  sha at GitHub first: it has quoted wrong ids before.
- Relay a coordinator line (`REQ review #N <sha>`, `REQ watch`, `REMIND`,
  `STATUS?`) to it with `SendMessage`.
- If no SlaveSession is running, the coordinator uses read-only Opus reviews
  and merges on them. Do not start a Claude PR-reviewer as a fallback unless
  the maintainer asks.

## 4. Standing orders (maintainer, 2026-11-18)

- **Do not wait for the review session.** The coordinator resolves
  conflicts, delta-reviews and merges in batches. SlaveSession is optional
  for refreshes and for the delta of a refresh.
- **Delta review.** A merge of main with a hand-resolved conflict (`DELTA`)
  or new commits get a read-only Opus review of the remerge-diff or of the
  new commits. A safety-floor PR gets that review on the same head.
- **Take over.** A conflicting PR or a `needs-rework` PR of a contributor is
  ours to fix when it is idle for more than 24 hours, or when the maintainer
  says "take them over". Drafts are included only on that word. Push to the
  author's branch only if `maintainerCanModify` is true.
- **P1 and P2 first.** Close and merge them. A contributor claim with no
  branch and no PR after 90 minutes is taken over when the maintainer says
  so.
- **Decide alone when both sides are kept.** A conflict resolution that keeps
  both sides and loses no refusal or safety rule is decided and recorded by
  the coordinator. It asks only when the two sides are exclusive.
- **Post-merge hold.** After a merge batch, no merge until the queued
  post-merge run ends, unless the maintainer lifts the hold. A push cancels or
  replaces the run.
- **UI PRs need before/after screenshots** in the body, unless the maintainer
  waives them for that PR.
- **Never merge:** `held-for-merge`, `needs-maintainer`, disputed PRs, a
  change to `.github/`, an unfinished draft that the maintainer did not hand
  over. Never `--admin`, never force-push, merge commits only.
- **No local test builds for the maintainer.** Do not build a PR branch for
  him unless he asks.
- **Terse.** One or two lines for each event. No tables unless asked. Say
  nothing when nothing changed. The maintainer's token budget is tight.
- **Private files.** The maintainer's own project files in
  `testdata/models/` (`fig8.camproj`, `testpart.sc`) are never committed,
  uploaded, pasted or quoted (#1749).

## 5. After a merge batch

1. Remove `in-review` and `claimed` from each closed issue.
2. If a process file changed: one `cargo xtask process-bump` in a worktree
   off the new main, one `chore(process): PROCESS_VERSION <n>` PR, merge it,
   and comment on #681 (AGENTS.md "Process changes bump `PROCESS_VERSION`").
3. Tell the coordinator to write `.claude/tmp/coordinator-handover.md` when
   the maintainer ends the session.

## 6. What this skill does not do

- It does not start SlaveSession.
- It does not start a schedule or a `/goal` loop. The maintainer sets them.
- It does not merge PR #2465 (the trial PR with `held-for-merge`). The
  maintainer decides that.
