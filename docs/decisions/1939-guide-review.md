## #1939 - Review the guide after #1802

**Chosen.** Review every PR merged after #1802 through upstream/main
`cda3d589` and update the existing guide pages by job stage. Keep a
per-PR audit on the issue, including a reason for each change that needs
no additional guide text. Also close the explicitly requested earlier
Trochoidal parallel-lane documentation gap from #1788.

**Why.** The guide must describe shipped behavior and the limits that a
user must satisfy. The old text omitted the shipped ALC400G, body
electrodes, wider Bore clearing and aligned 3D operations. New refusal
and post behavior needs a remedy and its control limitations beside it.

**Dropped.** Do not document unmerged Sodick 4-axis or flushing parameters
as supported. Issues #1497 and #1499 remain open. Do not add screenshots:
the change needs factual reference text and the existing operation
schematics suffice. Do not regroup the guide again; #1908 already did it.
Do not stack on #1920 or #1921: they change different sections in the same
files. State the overlaps so the second merge refreshes against main.

**Validation choice.** Add focused guide-content pins in `site/tests/`,
the existing home for guide tests. Assert supported workflows and reject
the obsolete no-machine and rectangular-only descriptions. Build the
site to check internal links. No Rust crate or NC output changes.

**Cost to change later.** A later guide regrouping must move these tests'
page paths with the content. A later Sodick implementation must replace
the stated refusals only after its control meanings and output are
verified. The next release cut must file the v0.23.0 review; this worker
does not create a future-cut issue early.
