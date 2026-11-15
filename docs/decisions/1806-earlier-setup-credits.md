## #1806 - Credit declared cuts before retaining a setup's gouges

**Decided:** apply the existing edge-break and mark credits when the program
walk judges stock before a new block or a side-on turn. Read only operations
already cut, and only declarations belonging to the departing setup.

**Why:** the retained verdict compared intentional chamfers, deburrs,
engravings and traces against the sharp model. The final setup's credit
could not correct that earlier verdict. Reusing the existing laws preserves
their width, depth, target-part and remaining-floor limits. A deeper cut
continues to count at its excess below the declared finish surface.

Read the verdict with its matching display lattice while retaining the stock
for the next setup. Borrow a carried column body and read a field body once.
Keep the body's existing floor verdict; do not replace it with a top-only
judgment. The existing credit walk only credits a column's top verdict.

Stack on PR #1804 because it introduces the side-on retained-verdict path.
PR #1811 also edits `walk`, in the electrode-dispatch block, and edits other
methods of `Held`. Preserve both changes when the later branch refreshes.

**Considered and dropped:** dropping retained gouges would hide real damage.
Applying every setup's declarations could excuse a different block's cut.
Copying the whole held field just to get a lattice would copy unnecessary
field layers. Duplicating the credit laws would give the final and retained
verdicts separate definitions of the same intended cut.

Transforming an earlier setup's credits into the final silhouette is outside
this change. A full-perimeter chamfer visible from the new tool axis can
still be reported by that final silhouette. The side-on fixture isolates a
straight generated edge pass away from the new top, so it tests the retained
verdict. No golden file or posted motion changes.

**Cost to change later:** extending credits across setup frames needs a
separate geometric rule for transformed cuts and target surfaces, with pins
that distinguish intended cuts from real damage on the final silhouette.
The present helper can continue to judge and retain each departing setup.
