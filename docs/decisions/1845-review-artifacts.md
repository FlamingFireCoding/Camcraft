## #1845 - Remove the remaining committed PR review artifacts

**What was decided.** Remove the review artifact folders for
[PR #1744](https://github.com/NeoTech/Camcraft/pull/1744),
[PR #1699](https://github.com/NeoTech/Camcraft/pull/1699),
[PR #1710](https://github.com/NeoTech/Camcraft/pull/1710) and
[PR #1724](https://github.com/NeoTech/Camcraft/pull/1724).
Include their readme files and the script that checks the PR #1724 images.
The xtask guard now checks that these four folders are absent.

Move the PR #1724 evidence out of the current product tree.
This supersedes the retention decision in the #1122 entry.
Git history keeps the images, capture provenance and check script.
The original [capture commit](https://github.com/NeoTech/Camcraft/commit/486b03c91c8b1330f1e79ec47500ad68625fdd5a)
preserves that record. This cleanup does not upload new images.

Remove the optional boundary capture hook. Keep its structural assertions.
Keep the manual PR #1710 capture test, but write its output to
`camcraft-review-1710` under the operating system's temporary directory.

**Why.** Issue #1794 removed the precedent for keeping review evidence in
the product tree. These files are review artifacts, not product inputs.
PR screenshots belong in PR bodies, with images on the author's fork
in its images-only branch. The boundary hook accepted an arbitrary path
and could recreate a removed folder. The manual capture also wrote into
a removed folder. Its temporary output keeps the capture useful.

**Considered and dropped.** Keep a special exception for PR #1724:
its evidence remains available in Git history, so it needs no exception.
Keep the boundary hook with its arbitrary destination: this could recreate
the folders that the guard rejects. Delete the manual PR #1710 capture:
changing its output directory is smaller and preserves the capture tool.
A general ban on documentation images would reject valid product content.

**Cost to change later.** Small. The guard names four additional folders.
The manual capture destination is one line. Future review evidence can
use the images-only branch without restoring files to the product tree.
