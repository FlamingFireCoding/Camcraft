## #3236 - wire-iso refuses a wire number that is not finite

**Decided:** the generic ISO wire post (`assets/posts/wire-iso.cps`)
refuses by name a `PARAM WireTaper`, `WirePlane`, `WireAngle`, `WireU`
or `WireV` that the stream states but that is not a finite number
(`null`, `nan`, `inf`). After that refusal the post writes no cut of the
section.

**Why:** the issue said that the post writes `UNaN` or `ANaN`. A test
showed a different failure. The app writes a non-finite `f64` as JSON
`null`, and the stream parser reads `nan` and `inf` as text. The script
skipped each such param without a word, so a leaning move posted as a
plain `G1 X Y` with no U V and no angle word. That is a silent wrong
cut. The refusal blocks the NC file in the app (`posting.rs`, a
`message()` is a blocking warning).

**Dropped:**
- A check on the Rust side only (`posting_wire.rs`). The script is the
  guard for every stream, also a hand-written one that `post.exe` reads.
- A refusal of the one move only. A section with one unreadable lean is
  not a program to run, so no later cut of it is written either.

**Cost to change later:** low. One list (`WIRE_NUMBERS`) and one guard
in `cutOk` of the script.
