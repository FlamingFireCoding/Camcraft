## #2378 - FreeBSD package for a tag whose freebsd job ran out of memory

**What was decided.** Two changes. The `freebsd` job of `release.yml` gets
`mem: 12288` (was 5120) and runs `cargo build -j 2`. A new workflow,
`.github/workflows/freebsd-backfill.yml`, has the trigger `workflow_dispatch`
only. Its input `tag` names an existing `vX.Y.0` tag. It builds the tagged
tree with the same VM steps, then attaches the archive to that tag's release.
It refuses a release that already has a FreeBSD archive. It then removes the
line "FreeBSD (tar.gz) build did not succeed" from the release body and
changes nothing else.

**Why.** The freebsd job of v0.23.0 was killed with exit 137 while it built
`boa_ast`. The probable cause is memory: the VM had 5 GB. The hosted
`ubuntu-24.04` runner has 16 GB, so 12 GB for the VM leaves room for the host.
`-j 2` lowers the peak of rustc. A tag is never moved, so the tag run cannot
repeat the build with a fixed workflow. A dispatch-only workflow builds the
tagged tree and adds the missing asset to the existing release.

**The build and the upload are two jobs.** The `build` job runs with
`contents: read`. Only the `attach` job has `contents: write`, and it runs no
code from the tag. GitHub sets permissions per job, not per step. A step
with write access inside the build job would give the built code a write
token. The archive passes between the jobs as a workflow artifact.

**Memory value.** The vmactions/freebsd-vm README gives the default
(6144 MB) and no maximum. 12288 MB is the value the maintainer approved.
It is not measured. A first run may still need more or less.

**Considered and dropped.**
- A third run of the tag workflow. The same kill happened on two runs, and
  a re-run uses the same workflow file from the tag.
- Moving the tag to a fixed commit. A tag is never moved (AGENTS.md).
- A local build by hand. The maintainer stopped local builds of PR branches.
- A `push`, `pull_request` or `schedule` trigger for the backfill. Each
  needs the maintainer's approval, and the backfill is started by hand.

**Cost to change later.** Changing the memory value edits one line in
`release.yml` and one in the backfill workflow. The backfill workflow copies
the VM steps of the `freebsd` job. A change to those steps must be made in
both files.
