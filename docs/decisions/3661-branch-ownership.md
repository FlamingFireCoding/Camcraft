## #3661 - Public ownership records for dispatch

**Decided:** Use public GitHub comments as a cooperative ownership guard.
Keep the canonical writer record with the issue claim. Put the canonical
reviewer record on the PR. Link the records across the threads. Record the
unique worker, dispatcher, head repository and branch, PR, base and full
starting head. Re-read before dispatch and immediately before a push.
Keep the existing local review lock and claim lifetime rules.

**Why:** Both sessions can read the same public thread. The issue already
uses claim comments, including contributors without label permission.
One reference under `issue-claims` keeps the four dispatch skills aligned.
Explicit release separates branch writing from a review without withdrawing
an issue claim that remains live. Refreshes compile affected code before
push, including clean merges and incoming base changes.

**Considered and dropped:** A local `.claude/claims/issue-N` file does not
coordinate separate checkouts or machines. A tracked record would change
its own recorded head on each commit. A distributed lock service would need
new infrastructure and permissions that this process issue does not need.
Git force-with-lease would not stop duplicate dispatch and conflicts with
the no-force rule. No new service, dependency or application behavior ships.

**Guarantee:** This procedure tells cooperating sessions to stand down on
observed live owners or moved heads. It is not an atomic distributed lock.
Readback detects visible duplicate records but cannot prevent simultaneous
comment writes or a race after the last remote read. A plain push refuses
non-fast-forward updates, not every unexpected fast-forward. Tests pin the
instructions and skill links; they do not prove distributed exclusion.

**Cost to change later:** Replace the shared reference and its contract
tests. Keep existing comment URLs and claim endings readable. Any stronger
atomic guarantee needs a separate design and explicit authority for a
server-enforced mechanism.
