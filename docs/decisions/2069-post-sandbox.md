## #2069 - Bound post scripts and provide preview inputs

**Decided:** Evaluate the host prelude, rotary helpers, and explanation
helpers before evaluating user source in a separate call. Name the sources
`host.js` and `user.cps`. Use Boa's own syntax positions and runtime stack
positions. User positions then start at line 1 and column 1 without a host
line offset.

**Why:** A script author must be able to find a reported error in their own
file. Subtracting a prelude line count would depend on host implementation
details and would not distinguish nested runtime frames. Separate evaluations
preserve the user source's positions and share the same JavaScript context.

**Decided:** Set Boa's loop limit to 1,000,000 iterations, recursion limit
to 64 calls, and stack limit to 4,096 entries before evaluating source.
Keep those limits for subsequent hook calls. Use the same engine limits
on desktop and wasm. Limit a post run to 100,000 output lines. Count normal
output helpers cumulatively across hooks. Check the output buffer length
in Rust before reporting its size or draining it, so direct buffer writes
cannot return an oversized program.

**Why:** Endless loops and excessive execution depth must produce a named
refusal. An output limit must cover the whole run, rather than restart at
each stream record. These are deterministic execution and output limits;
they are not a complete security boundary or a memory allocation limit.
Rejected wall-clock deadlines because they need platform-specific
interruption and can give different results on different machines.

**Decided:** Move the existing host prelude, hook names, boolean helpers,
and program identity into `js_host.rs`. Keep their existing public paths
through re-exports from `js`.

**Why:** This keeps the script loader below the approximately 400-line
module limit without changing existing callers or host behavior.

**Decided:** Expose `post::preview::BLANK_TEMPLATE`, `DEFAULT_VMC_JSON`,
and `SAMPLES`. The blank template declares the milling manifest and empty
functions for every engine hook. It deliberately emits no program text.
Each sample's `stream()` builds a deterministic `.cnc` stream from an
embedded committed golden. The four inputs cover facing, contouring,
pocketing, and drilling. Add setup, operation, tool, spindle, and clearance
records. Derive missing canned-cycle feeds from their hole chains before
normalizing record order. Embed the existing VMC machine definition.

**Why:** Preview callers on desktop and wasm need representative inputs
without filesystem access or a dependency on app generators. Existing
project goldens supply the motion data. The metadata follows the same
program context used by the repository's legacy golden tests. Rejected
copied vendor templates and post code; the template is original and uses
only public RS-274 / ISO 6983 conventions and this project's hook contract.
No new Cargo dependency is required.

**Overlap:** Stack this branch on #468 because both changes touch
`Script::load`. Keep #468's continuous-threading capability. PR #1681
touches `post/mod.rs` in a different function; whichever merges second
refreshes. This issue adds no threading golden changes of its own.

**Cost to change later:** Adjust limits with the boundary pins and rerun
the post suites to establish compatibility. A stronger security boundary
would need separate work on allocation, host-global mutation, and
execution interruption. Change preview metadata or add a sample together
with its blank-template and shipped-post checks. A change to the embedded
goldens automatically updates preview motion on the next build.
