## #1335 - Infer a user's wire post from its manifest

**Decided:** Reuse `Script::voices_wire_edm()` in the posting machine-kind
door after reading an explicit `serves` declaration. When `serves` is absent,
`wireEdm = true` proves only `Kind::WireEdm`. An explicit `serves` declaration
continues to determine the permitted kinds.

**Why:** The engine already reads the strict boolean manifest flag. Using that
same reading lets a user's wire dialect reach a wire EDM without admitting it
to milling, turning, plasma, or sinker machines. It preserves the existing
explicit restriction and the named refusal for a cutting script on a wire EDM.

**Considered and dropped:** Parsing the flag again in the app would duplicate
the engine's manifest contract. Treating the flag as permission for all EDMs
or other machine kinds would claim compatibility the script did not declare.
Inferring sinker support without a matching manifest contract is outside this
wire-only change.

**Cost to change later:** Another inferred machine kind needs a local change
to the posting door, a corresponding engine manifest contract, and acceptance
and refusal pins. Explicit `serves` declarations need no migration.
