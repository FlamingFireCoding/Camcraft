## #1844 - Refuse 3D Pocket depths without proved assembly clearance

**What was decided.** Choose the issue's refusal option. Before an upright
3D Pocket builds a path, compare its requested Bottom of Part Z with the
higher of the source material top and stock top. Refuse when a shank or
holder band wider than the cutter, with positive length, starts at or below
that top at the requested depth. Equality refuses too: the cavity pin shows
that contact with the wall top produces shank and rapid findings.

Read the same `Cutter` bands that verification uses. This includes modeled
holder bodies, necked shanks, and the existing enclosing-cylinder fallback.
Name the tool and the obstructing band. Tell the user to raise Bottom of
Part Z strictly above the stated threshold or use a longer-reaching tool.

**Why.** The ladder currently proves only cutter-radius clearance. A wider
assembly can touch the part even though its flutes clear every level. An
early named refusal meets the issue's requirement without weakening the
correct verification laws or returning a path that generation calls `ok`.

This check is deliberately conservative. It credits neither lateral room,
previously removed stock, nor stock to leave. A deep cut that has enough
lateral room can therefore refuse. The diagnostic states the lateral-room
and prior-stock limitations. Tilted jobs defer to their existing generation
rules; the existing tilted whole-model refusal keeps precedence.

Move the existing `run_job` orchestration into `generate/run_job.rs` and
keep its public re-export. This gives the preflight a small module and
shrinks the already oversized `generate.rs`. The move preserves error
mapping, logging, path notes, and the existing generation arms.

**Considered and dropped.** Widening each level for the shank and holder,
then reporting unreachable areas as rest, would preserve more usable cuts.
It also needs new proofs for entries, transitions, links, shells, and rest
passes. That algorithm is beyond this refusal fix. Deferring the warning
until verification leaves the generation result misleading. Changing the
verification laws would hide real contact, so that option was rejected.

**Cost to change later.** Replace this preflight when the ladder can prove
assembly clearance per level, with pins for both reachable and unreachable
regions and all connecting moves. Keep the actual tool-band interpretation
and the named refusal for cases the planner still cannot prove.

**Validation.** The sample and closed-cavity pins fail before the fix and
pass afterward. Restoring upstream generation, disabling either band's
check, or allowing equality makes the relevant pin fail; restoring the fix
makes it pass. Shallow depths, longer flutes, a flute-width shank, and a
0.001 mm axial gap remain accepted. The quick gate passed in 362 seconds,
and app all-targets clippy passed with warnings denied. No goldens,
verification laws, dependencies, or process files changed.
