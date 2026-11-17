## #1848 - Name the repeat-depth checkbox through its caption

**Decided:** Associate each form checkbox with its existing caption through
`Response::labelled_by`. Clear the checkbox's empty direct AccessKit label
so assistive tools can resolve the caption. Add a structural assertion for
the Threading Passes tab's `Repeat each depth` checkbox, including its role
and enabled state.

**Why:** The checkbox had an empty accessible name. Its separate caption
was visible, but a query for a checkbox with that name could not find it.
The shared form helper owns this caption-and-control relationship.
The pin also checks keyboard activation. The existing caption-click test
now selects the caption by role because the checkbox has the same name.

**Considered and dropped:** An assertion for the caption alone would not
prove that the checkbox has a name or is enabled. A separate explicit name
on the checkbox would duplicate its existing caption. A page-specific
checkbox would bypass the shared form helper. Pixel snapshots would not
prove the accessible role or enabled state.

**Cost to change later:** The association is one shared helper change.
The structural pin stays in a separate small module and must change if
the page's control name, role or availability changes.
