## #1120 - Colour literals: a guard with exact counts first, then one PR for each family

**What was decided.** The first #1120 PR adds only the guard,
`crates/ui/tests/colour_literals.rs`. It walks `crates/ui/src`, counts the
colour literals in each file that is not a test file, and compares each
count with a row of `ALLOWED`. The count must be equal: more fails ("read a
theme token instead"), fewer also fails ("lower the row"), so the list only
shrinks. Each row says why: `keep:` for a colour that is not a choice at a
draw site, `move (#1120 <family>)` for a site that a later PR moves to
tokens. On `main` at f287a370c the list has 31 files and 162 literals.

The families that later PRs move, each with its tokens and its theme-file
keys (`theme_files::COLOR_NAMES`):
- viewport overlays (pick, avoid and handle inks, ground ink, view cube,
  viewport bar, generate overlay, drop hint, shell overlays);
- tool drawings (tool, holder and profile previews, tool library views);
- verdict palettes (`colorvision.rs`), moved into the token table with their
  values unchanged, as the issue says;
- G-code syntax colours, window chrome, status inks.

The `keep:` rows are the token tables, the contrast maths, a colour parsed
from a theme file, pixels read back from the GPU, a token with a disabled
alpha, and a colour that only measures text.

**Why.** On `main` there are about 95 literal sites to move, not the 51 of
the 2026-11-12 estimate. One PR for all of them would touch about 30 files
and conflict with most open UI PRs. The guard stops the count from growing
from the first merge, and each family PR is small and reviewable.

**Considered and dropped.**
- A guard that only refuses a file with no row. It lets the count inside a
  listed file grow without notice.
- A regex or a parse of the Rust source. A plain substring count over the
  lines, with line comments skipped, is enough for the forms that occur, and
  the guard stays in `std`.
- Moving the colour-vision maths and the contrast maths. They compute
  colours; they do not choose them.

**Cost to change later.** Small. The guard is one test file. A new literal
form is one line in `PATTERNS`, and the counts are printed by the failing
test, ready to paste.
