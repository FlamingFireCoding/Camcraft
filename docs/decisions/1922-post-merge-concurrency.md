## #1922 - Keep the running post-merge gate

**Decided:** Set `cancel-in-progress: false`. Keep the existing concurrency
group, which separates main from other refs.

**Why:** A later merge must let the running gate finish and report its result.
GitHub keeps one pending run per group. A newer run replaces that pending run,
so a merge batch cannot create an unlimited queue.

**Dropped:** A separate group for each commit would create a long queue.
Automatic bisect after a failed gate is outside this issue's scope.

**Cost to change later:** Change one setting. This choice uses more runner time
and can delay the result for the latest commit by one gate run.

**Not exercised:** The local test checks the workflow setting. An actual merge
batch must confirm the Actions queue behaviour after this change merges.
