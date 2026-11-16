## #2030 - Offer every machine kind in the Machines pane filter

**Decided:** Append Waterjet, Wire EDM and Sinker EDM to the private
`KINDS` array. Keep the first four entries in their existing order.
Use a structural test to check all seven kinds through the dropdown and
check the filtered machine rows.

**Why:** The filter must offer each kind that the machine library can
hold. The missing entries include Waterjet as well as the two EDM kinds.
The existing array controls both the filter and the New blank kind.
Appending entries preserves the existing filter indices.

**Dropped:** Adding only the EDM kinds would leave Waterjet unavailable.
A shared enum list or a wider API change is unnecessary for this fix.

**Cost to change later:** Add a filter entry and a test entry when a new
machine kind becomes available. A shared list can replace the private
array if more callers need the same complete list.
