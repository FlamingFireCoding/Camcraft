## #3447 - setups keep their creation order

**What was decided.** `SetupDef` has a new field `order: Option<u32>`.
`CreateSetup` writes one past the highest stored order (0 for the first
setup). `setup::setups` sorts by this field. Setups without the field
(every earlier file) sort first, by name, as before. The project file
saves the field as an additive `order` key on the setup row, with no
version step. `get_job`, `add_setup`, the operation order and the post
all read `setup::setups`, so they all follow the new order.

**Why.** The issue asks that a setup added second never posts first. A
setup made with `datum_from` takes its datum and its stock from the
source; it must come after it. The operation fix of #3022 uses the same
pattern: a stored position, and the name order only for older data.

**Considered and dropped.**
- Sort by the entity id (creation order without a stored field). Dropped:
  a project load creates the entities again in file order, and undo of
  `DeleteSetup` creates a new entity. The order would not survive.
- Give each older setup an order at load time. Dropped: it changes the
  saved bytes of every older file without a user action. Name order for
  these files is the old behaviour, so nothing changes for them.
- Add the reorder command in this PR. Dropped to keep the fix small. It
  is filed as #3484 (command, MCP tool, refusal when a setup would post
  before its datum source).

**Test change.** `setup_delete_order.rs` pinned the old law: its fixture
created `B_Middle` last and expected it in the middle by name. The
fixture now creates the setups in A, B, C order. The test still pins
that undo of `DeleteSetup` gives the setup back its old place.

**Cost to change later.** Low. The field is optional; a reorder command
writes it. A file that a newer build saves opens in an older build; the
older build ignores the key and uses name order.
