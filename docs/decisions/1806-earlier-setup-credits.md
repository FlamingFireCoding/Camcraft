## #1806 - Credit declared cuts before retaining a setup's gouges

**Decided:** apply the existing edge-break and mark credits when the program
walk judges stock before a new block or a side-on turn. Read only operations
already cut, and only declarations belonging to the departing setup.

**Why:** the retained verdict compared intentional chamfers, deburrs,
engravings and traces against the sharp model. The final setup's credit
could not correct that earlier verdict. Reusing the existing laws preserves
their width, depth, target-part and remaining-floor limits. A deeper cut
continues to count at its excess below the declared finish surface.

Read the top verdict with its matching lattice while retaining the stock
for the next setup. Borrow a carried column body and read a field body once.
Apply credits to the top before combining it with the body's untouched
floor verdict. The combined per-cell value stores only the worse face, so
crediting it afterwards can erase a shallower real underside gouge beneath
a deeper valid top mark. Keep both faces' counts, depth and location through
the merge; do not replace the body's verdict with a top-only judgment.

Stack on PR #1804 because it introduces the side-on retained-verdict path.
PR #1811 also edits `walk`, in the electrode-dispatch block, and edits other
methods of `Held`. Preserve both changes when the later branch refreshes.

**Considered and dropped:** dropping retained gouges would hide real damage.
Applying every setup's declarations could excuse a different block's cut.
Copying the whole held field just to get a lattice would copy unnecessary
field layers. Duplicating the credit laws would give the final and retained
verdicts separate definitions of the same intended cut. Restoring underside
depth after crediting the combined verdict would require recovering face
information already lost by the merge; crediting first keeps that information
at its existing source.

Transforming an earlier setup's credits into the final silhouette is outside
this change. A full-perimeter chamfer visible from the new tool axis can
still be reported by that final silhouette. The side-on fixture isolates a
straight generated edge pass away from the new top, so it tests the retained
verdict. No golden file or posted motion changes.

**Cost to change later:** extending credits across setup frames needs a
separate geometric rule for transformed cuts and target surfaces, with pins
that distinguish intended cuts from real damage on the final silhouette.
The present helper can continue to judge and retain each departing setup.
