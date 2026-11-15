## #1792 - Review the guide against shipped changes

**What was decided.** Review every first-parent merged PR from v0.20.0 to
the branch's upstream/main base. Map each PR to a guide page, or record why
it needs no guide change. Describe controls and refusals from CamCraft's
own source, tests and PR descriptions. Add a tools and tool libraries page.

**Why.** The guide has no tools reference. Several shipped commands,
parameters and refusals are missing from the existing pages. A complete PR
list makes the review scope clear and separates user changes from internal
test and process changes.

**Considered and dropped.** Do not document open PR behavior as shipped.
Those controls can change before merge. Do not add a recurring release
issue rule in this PR. The issue leaves that process decision to the
maintainer. Do not add a Rust crate test for prose. Use an external content
pin for the new tools page and its three entry links, then use Zola to
check page rendering and internal links. Keep test logs and the audit
evidence outside the branch.

**Cost to change later.** Another release review can use the same PR audit
method with the next tag range. A recurring issue rule needs a separate
process decision. Permanent content checks can replace the external pin if
the site gains its own test suite. No product code or file format changes.
