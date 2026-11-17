## #2307 - Repair the regression suite Gate maps

**Chosen:** Name `sim::verify` in the fig8 suite header and add
`app::posting` and `app::posting_files` to the shipped-file suite header.
Pin quick-gate selection for all three module paths in xtask.

**Why:** The existing gate-map law rejects stale owners and missing maps.
The selection pins also prove that changes to the covered modules select
these regression suites. Production code and golden files stay unchanged.

**Dropped:** Weakening the scanner would hide incorrect test coverage.
Changing test bodies would not repair the metadata that caused the failure.

**Cost to change later:** If module ownership changes, update these headers
and the matching selection pins together.
