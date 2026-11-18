## #2523 - Keep example NC programs at LF

**Decision:** Use `examples/**/*.nc text eol=lf` for all example programs.
Replace the separate rule for the first 4-axis part.
Test a fresh Git checkout with `core.autocrlf=true` in an isolated repository.
The test copies the real attributes and all tracked example NC programs.

**Why:** The post writes LF bytes. The example tests compare those bytes
with the shipped programs. Git must preserve LF on Windows as well as Linux.
The glob covers programs directly in `examples/` and inside the catalog.
All four programs already have LF in the Git index, so no renormalization is needed.
The 4-axis tests read the already pinned program. The site tests read only
that program and use text reads.

**Dropped:** Separate rules for each program need updates for new examples.
A repository-wide `*.nc` rule or global line-ending rule changes files
outside this issue. Changing the byte comparisons would hide checkout errors.

**Cost to change later:** Edit one attribute rule and its checkout test.
No posted program or golden changes.
