## #1700 - A stock copied from an incomplete model keeps naming its hole

**Decided:** a `FromModel` stock copy carries its source's hole
(`schema::MeshOpen`) itself, beside the fidelity-unknown sentinel it
already carried (#1056, #1610). Every path that lands the sentinel on a
copy lands the hole with it: `SetStock` and its undo, a setup clone
(`CreateSetup` with a datum), an undone `DeleteSetup`, `UpdateModel`'s
refresh of the copies and its undo, and the project loader. The refusal
(`model_fidelity::op_problem`) reads the hole off the entity that carries
the sentinel, so a copy's refusal still names its source and that hole
once the source is deleted. `mesh_open::hole_behind` is
`model_fidelity::chord_behind`'s twin, and `mesh_open::note_for` gives the
words for a hole already in hand.

**Why:** the copy's refusal read the hole from the source model by name
(`mesh_open::unknown_note`). With the source deleted that lookup found
nothing, and the refusal fell back to "saved before CamCraft recorded
mesh fidelity (project v58 or older)", which is the wrong reason for that
mesh. The post still refused it; only the words were wrong. The copy
already held the sentinel for exactly this case (#1056: "keeps it when
the model is deleted or renamed"), so the hole now rides the same way.

**File format:** the saver already wrote `mesh_open` for any entity that
has one. The loader now also accepts it on a `FromModel` stock marked
`fidelity_unknown`, and a hole without the mark stays corrupt. There is
no version bump: #1610's strict check (a hole on anything but a model is
corrupt) has not shipped in a cut yet, so no released reader refuses
these files, and a reader from before #1610 drops the key and keeps the
mark. Files without the key load unchanged.

**Considered and dropped:**
- Keeping a tombstone of the deleted source's hole (by name) in the
  world, which would have given a second source of truth.
- Looking up a copy of the deleted source by name at refusal time. That
  has more than one possible answer when two setups each copy the model,
  and it is wrong after a rename.

**Cost to change later:** the hole is one optional component on the copy,
written beside the sentinel at the six sites listed above. Pinned by
`crates/app/tests/stock_copy_hole.rs`.
