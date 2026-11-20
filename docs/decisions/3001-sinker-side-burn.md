## #3001 - Post explicit Side Burn moves through the machine dialect

**Decided:** Add the optional `edm.codes.side` template to the shipped
sinker post and the machine editor's code list. Require `{x}`, `{y}`,
`{z}`, and `{f}` in the template. Fill them with the absolute endpoint
in the work offset and the feed, in the document's units. Keep all EDM
codes in the machine file. Add no production code for a real control.

**Why:** The generated Side Burn already states its downward plunge,
lateral feed, return feed, and rapid retract as explicit CL moves.
Post the plunge with `depth`. Post each lateral feed with `side`.
Preserve the direction and amount that the generator states. Accept only
feeds along one X or Y axis at constant Z. Refuse missing dialect data,
incomplete coordinates, invalid feed, and unsupported moves by name.
The existing straight burn keeps its current depth code and needs no
`side` entry. The tests use an invented dialect. No real EDM control was
exercised.

**Dropped:** Do not choose an implicit lateral code or reuse `depth` for
a lateral feed. Either choice can omit the required movement. Do not
replace explicit moves with a control orbit. Do not add general XYZ or
diagonal servo feeds; the generated Side Burn does not need them.

**Cost to change later:** A real control needs verified machine data from
its public manual and inspection of its NC output. Additional motion
types need a separate safety rule and pinning tests. Existing machine
files remain valid and refuse Side Burn until they state `side`.
