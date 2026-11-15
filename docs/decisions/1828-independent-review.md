## #1828 - Require independent review evidence before merge planning

**Decided:** The merge-ready skill reads every page of formal reviews,
issue comments and label events before it offers a PR. Collaborator status
alone does not make an author's markers an independent review. The skill
records the counted review's URL, author and head SHA. A review through the
author's account also needs a named different agent and a linked review
dispatch/result that establishes independence. The planner receives only
the explicit PR numbers retained by these checks.

**Why:** Author-only markers and an author-set label allowed PRs into a
merge batch without a counted independent review. Planning by label could
also restore PRs excluded during collection. Issue comments must remain a
valid review source because reviewer agents normally use them.

**Considered and dropped:** Requiring a different GitHub account would
reject independent agents that share an account. Trusting the label alone
would preserve the defect. Adding a review-eligibility engine to the merge
planner would expand this skill-focused fix into new runtime behavior.

**Validation limit:** Three structural instruction tests and worked
examples pin the collection, independence and filtered-planning rules.
They do not prove live GitHub classification or an agent's execution of the
workflow. Each test fails with the upstream skill and passes with the fix.

**Cost to change later:** Revise the skill, its examples and its three pins
if review evidence changes. An automated eligibility engine would need
separate handling and tests for review provenance and GitHub history.
