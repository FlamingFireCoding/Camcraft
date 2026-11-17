## #2329 - Open docs search from the URL query

**Decided:** Read `q` with `URLSearchParams` when the search script loads.
If the parameter exists, copy it into the input and call the existing
`render` function. Leave the search unchanged when the parameter is absent.

**Why:** Links and form submissions use the same query format. The existing
renderer already handles results, empty queries and accessibility state.
The tests execute the inline script with the vendored search library.

**Dropped:** Manual URL decoding duplicates browser behaviour. A separate
search renderer adds code without a different requirement.

**Later change cost:** Low. Change the initialization block and its URL
tests if the query parameter or startup behaviour changes.
