## #1135 - the empty sinker template joins the machine library

**Decided.** `assets/machines/sinker-edm.mch` ships in `posting::MACHINES`
as "Sinker EDM 3-axis (template)". Its `edm` section stays empty. The
sinker post "Sinker EDM (ISO)" is its only post.

**Why.** Every library machine needs a post. The post shipped with #542.
The user fills the codes from the control manual, and the post refuses by
name until then.

**Dropped.** Moving the test fixture (`crates/app/tests/edm/sinker-edm.mch`)
into the assets: about a dozen suites include it by path. The fixture stays
as a copy, and a test pins that it equals the shipped file. The wire
template stays a fixture: no issue asked for it.

**Cost to change.** Rename the picker label in one line of `posting.rs`.
