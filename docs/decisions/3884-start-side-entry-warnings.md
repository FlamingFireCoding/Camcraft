## #3884 - Plain Outside pocket: each stand-in entry and a selected Plunge into stock is a warning

**Decided.** On a start-side ladder ("Pass starts at" Inside or Outside;
on the avoid route also Auto that resolves to a side round a part: the
ladders with the #3316 link law, `Motion::cleared_links`), the entry that the user did not select is never a note only.

1. **Arc with no air room uses a safe stand-in, named in a Warning.** A
   plain Outside pocket with no stock known has no air room (#3746), so the
   Arc entry cannot start in the air. Where a later pass start is in
   material, the ladder keeps its named stand-in (the ramp along the pass,
   a clear helix, the axial dive, or at last the plunge). The path also
   gets the marker note `ENTRY_STAND_IN_NOTE`, and the operation posts one
   advisory `Warning::EntryStandIn { at, entries }` that counts the named
   stand-ins. The op page shows each stand-in note as a `warning:` line.
2. **A selected Plunge into uncut stock is one Warning.** A start-side
   pass link that plunges into a pass start in material writes
   `PLUNGE_INTO_STOCK_NOTE` once. The operation posts one advisory
   `Warning::PlungeIntoStock`. Where the op already posts
   `EntryNotRecommended` for the selected Plunge (the air room exists), it
   does not post the second warning, so the Plunge is named once.
3. **`entry_status_lines`** shows each stand-in entry (`is_entry_stand_in`)
   as `warning: Entry: ... - fix: ...`. The two marker notes add no line:
   the op's warning is already a status line.

**Why.** The maintainer's rule on #3748: "if the selected entry is not
possible, the operation does not fall back silently. It refuses the entry
with a diagnostic, or it uses a safe entry and names it in a warning". A
refusal would refuse every plain Outside pocket with the default Arc entry
and no stock: a legal program refused. The ramp along the pass at the ramp
angle is a safe entry, and the path is verified like any other.

**Dropped.**
- Refuse the Arc entry on a plain Outside pocket with no air room: it
  refuses the default op of a common case.
- Post the `EntryNotRecommended` advice where no air room exists: its
  reason ("an Arc entry starts in the air outside the stock") is not true
  with no stock known, and the selected Arc there gets a stand-in too.
- Make the warnings block the post: the entries are verified, and the
  Plunge is the user's selection.
- Warn on the stand-ins of Auto ladders (no start side): out of the scope
  of #3884. Their notes are warning lines on the op page now, but they post
  no operation warning.

**Cost to change later.** The rule is in `machining2d/entry_warn.rs`
(the note markers, the stand-in tails and `entry_warnings`). The tails
match the note texts of `descent_dive`, `descent` and `air_entry`; a new
stand-in note adds its tail there.
