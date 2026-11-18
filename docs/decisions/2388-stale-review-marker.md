## #2388 - Warn when the latest collaborator review marker is stale

**Chosen:** Compare the fetched PR head with the newest dated collaborator
`pr-reviewer` marker from both issue comments and pull-request reviews.
Read every page and count only users with push permission on the target
repository. Ignore unpublished reviews with no date. Accept a matching
abbreviated SHA. Print stale-marker warnings on stderr so JSON output stays
usable.

**Why:** A merge-ready label can outlive the reviewed head. The merge planner
needs to show that mismatch before the merging session acts. Markers appear
on two API endpoints, so either endpoint alone can miss the latest review.
The fetched head is the same commit the planner simulates.

**Dropped:** Automatically refusing the merge or running a delta review.
This issue asks for a warning; the merging session decides what review is
needed. Also dropped reading only the review endpoint or trusting every
comment author, since both would contradict the repository review rules.

**Verification limits:** The CLI pins use Unix shell stubs for git and gh.
They exercise endpoint selection, pagination arguments, marker ordering,
collaborator filtering, SHA comparison, and stderr output without network
access. They do not exercise GitHub authentication or Windows process
launching.

**Cost to change later:** The marker reader is an xtask-only module with no
new dependencies. A future merge refusal or structured JSON warning can use
the same facts but would need its own behavior and tests.
