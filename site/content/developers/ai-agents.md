+++
title = "Contributing with AI agents"
description = "Getting started with Claude Code on CamCraft, the issue-to-merge flow, and what we learned running agents on several machines."
weight = 5
+++

Most CamCraft changes are written by AI coding agents (Claude Code, Codex
and others) that contributors run on their own machines. This page is the
practical side: how to start, how work flows from an issue to `main`, and
the gotchas we hit. The rules themselves live in
[`AGENTS.md`](https://github.com/NeoTech/Camcraft/blob/main/AGENTS.md); where
this page and `AGENTS.md` differ, `AGENTS.md` is right.

Questions and coordination happen on the
[CamCraft Discord](https://discord.gg/bBWjkAgX5).

## Your first hour

1. **Say hello** on the [CamCraft Discord](https://discord.gg/bBWjkAgX5): what
   machine you have (CNC or computer), and what you would like to work on.
2. **What you need:**
   - a GitHub account, and the [GitHub CLI](https://cli.github.com/) logged
     in (`gh auth login`) - agents use it to read issues and open PRs;
   - [Rust](https://rustup.rs/) stable, and `git`; on Linux the GUI build
     also needs `pkg-config`, `libwayland-dev`, `libxkbcommon-dev` and
     `libx11-dev` (Debian/Ubuntu names);
   - an AI coding agent. Claude Code needs a paid Claude plan or API credits;
     a Pro plan is enough to start, heavier use wants a Max plan. Other agents
     (Codex and the like) work too - they read the same `AGENTS.md`;
   - free disk: about 30 GB to start, more per extra worktree (see below).
3. **Fork, clone, build:** fork `NeoTech/Camcraft` on GitHub, clone your fork,
   add the original as `upstream`, and run `cargo xtask gate quick` once to
   warm the build (the first build takes a while). The `/fork-sync` skill
   keeps your fork's `main` and process files in step with the project.
4. **Find a first issue:** open, unclaimed bugs with a small effort label -
   [P3/P7 bugs, effort low or medium](https://github.com/NeoTech/Camcraft/issues?q=is%3Aopen+label%3Abug+-label%3Aclaimed+-label%3Ain-review+label%3Aeffort%3Alow%2Ceffort%3Amedium).
   Read the issue and its comments, then claim it with a comment.
5. **Ask your agent to do it:** in Claude Code, `/work-issue <number>` runs the
   whole flow below. Read its draft PR before you mark it ready.

## Getting started with Claude Code

1. Install [Claude Code](https://claude.com/claude-code) and a Rust stable
   toolchain, then fork and clone the repository.
2. Start Claude Code in the clone. It reads `AGENTS.md` automatically, and the
   project skills in `.claude/skills/` are available as slash commands:
   - `/work-issue` - claim one issue, branch from the latest `main`, fix it
     with a pinning test, run the quick gate, open a draft PR.
   - `/gh-issues` - read, file and update issues the way the project expects.
   - `/review-prs` - review an open PR (collaborators).
   - `/gate` - run a test gate tier.
3. Optional: CamCraft has an MCP server (`camcraft-cli mcp`), so an agent can
   drive a CAM document headlessly to reproduce a bug or check a result
   ([AI and automation](@/guide/automate/ai.md)).

A good first task is a `P3` or `P7` issue with `effort:low` or
`effort:medium`. Bugs (`P1`-`P3`) are always the most welcome.

## From issue to `main`

1. **Pick and claim.** Open issues are the work list, ordered by priority
   (`P1` first; `P?` is paused, leave it alone). Claim an issue by
   commenting `claimed: <who> · <plan>` - a comment is a claim. Read the
   comments first: someone else's live claim means the issue is taken.
2. **Branch from the latest `main`**, one issue per branch, ideally one git
   worktree per issue.
3. **Fix with a pinning test** - a test that fails before the fix and passes
   after. Run the quick gate: `cargo xtask gate quick --base main`.
4. **Open a draft PR.** The body is checked by reviewers, so include it all
   the first time:
   - what changed, the readings (numbers, before and after), the pinning
     test names, and what was *not* exercised;
   - `Changelog:` with one line (or `none` for docs/tests);
   - `Fixes #N` (or `Refs #N` for partial work);
   - `Process change: yes` if you touched a process file (`AGENTS.md`,
     `.claude/`, `xtask/`, the gate scripts);
   - **before/after screenshots** for any visible UI change, as images on
     an images-only branch of your fork (never in the PR branch itself);
   - a reason for any new top-level test binary (or add the test to an
     existing suite).
5. **Mark it ready** when the gate is green and the body is complete.
6. **Review.** A collaborator's agent reviews it: a trial merge with `main`,
   the touched suites, clippy and the project laws. Changes that can affect
   posted G-code, collisions, simulation or geometry get the safety tier
   (stronger model, mutation checks). Findings come back as a PR comment;
   fix them on the same branch with a plain push (never force-push).
7. **Merge.** A clean review sets `merge-ready`, and the PR is merged by a
   maintainer or the merge bot. If `main` moved and your PR conflicts, merge
   `main` into your branch when asked - the project refreshes at merge time
   rather than continuously, so you do not have to chase every merge.

Missing paperwork (screenshots, `Process change:`, `Changelog:`) is the most
common reason for a second review round. Ten seconds on the body saves an
hour of re-review.

## Tips and gotchas

- **Disk.** Each worktree has its own `target/`: 10-25 GB for a normal
  build, and 150-200 GB once the release-mode heavy lanes have run in it.
  Remove a worktree once its PR merges (`git worktree remove`), and watch
  free space if you run several agents.
- **Compile cache.** [sccache](https://github.com/mozilla/sccache) makes a
  fresh worktree cheap: `cargo xtask gate` uses it automatically when it is on
  `PATH`. Never set `CARGO_TARGET_DIR` (it breaks the cache across worktrees
  and mixes crates between concurrent builds). If builds hang, an sccache
  server may be wedged: `sccache --stop-server` and build again.
- **One gate per checkout.** Do not pull or switch branches in a checkout
  while a gate runs there; give each concurrent job its own worktree.
- **Long commands.** Builds and gates take minutes. Run them in the
  background (or in a subagent) instead of blocking the session.
- **Failures that are not yours.** If a heavy test fails, check it on bare
  `main` first: a failure that also happens on `main` does not block your PR.
- **Windows.** The repository pins LF line endings (`.gitattributes`). GPU UI
  tests can be flaky on hybrid-GPU laptops (two graphics adapters).
- **macOS.** The default APFS volume is case-insensitive: tests that create
  files whose names differ only by case behave differently there.

## More than one computer

Agents scale well across machines if each one runs its own small job pool:

- **A queue per machine.** A text file of jobs (`work N`, `review N`,
  `fixup N`) and a maximum number of concurrent jobs; a loop starts the next
  job when a slot frees. A coordinator session feeds the queues over SSH.
- **One worktree and one `target/` per job.** Shared build directories
  corrupt each other.
- **Avoid duplicate work.** Before queuing a review or a fix, check that no
  other machine is already running one for the same PR and commit. Two
  reviewers in one worktree overwrite each other.
- **Match the job to the machine.** Big PRs (gates of 1,000+ test steps)
  belong on the fastest machines; small reviews run anywhere.
- **Pace the subscriptions.** Give each AI plan a weekly budget and spread
  it over the week, so a busy day does not exhaust it; send cheap work
  (triage, issue sweeps, research) to cheaper models.

## Using subagents

- Let **subagents** run gates, reviews and research while the main session
  keeps planning; their results come back as a short summary.
- **Never review your own change** with the agent that wrote it - an
  independent reviewer finds what the author's context hides.
- Use the **stronger model for safety-tier work** (toolpaths, posts,
  simulation, geometry) and cheaper ones for paperwork checks and triage.
- Keep subagent briefs specific: the issue or PR number, what to check, and
  what not to touch.

## Where to talk

- [CamCraft Discord](https://discord.gg/bBWjkAgX5) - questions, test
  machines, coordination.
- Process changes are announced on the pinned issue
  [#681](https://github.com/NeoTech/Camcraft/issues/681).
