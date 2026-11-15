# 1829 - AGENTS.md: shorter, ASD-STE100, procedures in skills

**Chosen.** `AGENTS.md` keeps the rules and loses the procedures. It went
from 1240 lines (81 kB) to about 700 lines (40 kB). Five new skills under
`.claude/skills/` hold the procedures that one task uses: `gate`, `runners`,
`release-cut`, `fork-sync`, `issue-claims`. A table at the top of `AGENTS.md`
names the skill for each task.

**Why.** Each session and each subagent loads `AGENTS.md` in full. The
runner installation, the five cut steps and the fork sync were read by all
agents and used by few. The maintainer asked for a shorter file in
ASD-STE100, with functions moved into skills (issue 1829 has his words).

**Where each moved section is now.**

| Old section of `AGENTS.md` | New place |
|---|---|
| Gate flags (`--only`, `--failed`, `--dry-run`), quick selection, `SLOW_BINARIES`, heavy-lane steps, narrow ladder, `VisualRunner` | `gate` skill |
| Shared compile cache (sccache) | `gate` skill; one rule line stays |
| Local batch gate when the post-merge runner is down; hand-test build | `gate` skill |
| The cut runner: machine, registration, safety | `runners` skill; the trigger and approval rules stay |
| The post-merge gate: skip, report, issue rules | `runners` skill; a summary stays |
| The PR test runner: refusal list, labels, per-OS notes | `runners` skill; a summary stays |
| Billed minutes of one cut | `runners` skill |
| A cut goes PR review -> merge to main -> tag, steps 1-5 | `release-cut` skill; the four cut rules stay |
| Changelog row format | `release-cut` skill |
| Fork sync at cut tags; process-file sync for each session | `fork-sync` skill; the rule and the file list stay |
| Claim mirror, stale release, withdrawal, maintenance sweep, fixed comment texts | `issue-claims` skill; the liveness rule stays |
| `needs-research` steps | `issue-claims` skill; the rule stays |
| PR rules that were in "Issue Law", "Autonomous Work Law" and "Commits and Releases" | one new section, "Pull Request Law" |

**Kept at full strength.** Clean-Room Protocol, Prime Directive, Everything
Is a Command, Assets, Dependencies, Code shape, the priority table, the
effort table and safety floor, the lanes, the collaborator rule, the draft
rule, the gate tier table, the Actions trigger rules, the versioning table.
Each rule name that a skill, an agent or a code comment cites is still a
bold name or a heading in `AGENTS.md`.

**Dropped.**

- The dated quotations of the maintainer and the "superseded since" notes.
  They are in the git history (`git log -p -- AGENTS.md`; the last long form
  is at commit `b0a95adc9`).
- The "Was" column of the priority table (the old P0/P1/P2 scale).
- The history of the M4.x refactor phases in "Code shape" and "Testing".

**Considered and not done.**

- A target of 300-350 lines. The rule text alone does not go below about
  700 lines without the loss of a rule.
- A move of the claim rule into the existing `gh-issues` skill. That skill
  is generic `gh` mechanics; the claim law is ours, thus it has its own
  skill.
- Edits to the existing skills and agents. Their citations still resolve,
  so this PR does not touch them.

**For the maintainer to check.** These rules have new wording, and the
meaning must be the same: "Every PR opens as a draft" (the merge-time
refresh classes), "Only collaborators' reviews count" (the disagreement
case), "A claim must be live", and the four cut rules.
