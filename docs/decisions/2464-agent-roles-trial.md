# 2464 - agent roles trial: coordinator, workers, context scouts

The maintainer asked for the trial on 2026-11-18 (his words are on #2464).
These choices were taken unattended.

- **`effort:medium` goes to Opus, `effort:low` to Sonnet.** He said: Opus
  for "all code changes, actual work", Sonnet for "quick small
  understandable fixes". `effort:low` is the tier that the effort table
  describes as mechanical and one file. `effort:medium` has new logic in a
  module, thus it is "actual work". Dropped: Sonnet for `effort:medium`
  (cheaper, but it is not what he said).
- **`effort:max` goes from Fable to Opus.** Dropped: Fable for
  `effort:max` workers.
- **The coordinator runs on Opus.** The trial started it on Fable. On
  2026-11-18 the maintainer said: "so change the orchestrator to a Opus
  model." The reason is the token cost: each agent event wakes the
  coordinator with its full context. Dropped: Fable for the coordinator.
  No role uses Fable now.
- **A worker still reads the lines that it edits.** "None of these
  processes are actually doing the file handling" cannot be exact: an edit
  needs the exact text of its target. The rule is: a scout does each
  search and each first read; a worker reads only the ranges that the pack
  names and the lines that it changes.
- **A worker does not stop when no scout answers.** It starts one scout
  agent, and if that fails it searches itself and reports it. Dropped: a
  hard stop, because a stall is a bug (Autonomous Work Law).
- **Context packs are files in `.claude/tmp/context/`.** That path is
  machine-local and not committed. A file lets the coordinator hand one
  pack to a worker without a copy of its text in each brief.
- **Reviews, gate-runner and graph-updater do not change.** He named
  orchestration, code work and file reading only.
- **The PR does not edit `.claude/PROCESS_VERSION`** and has `Process
  change: yes` (Issue Law). It has `held-for-merge` from the start.

Not known at the start of the trial:
- if a subagent coordinator can start and message its own subagents in
  each client;
- if four idle Haiku agents cost less than the reads that they save.
The trial measures the two.
