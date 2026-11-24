## #3884 - Plain Outside pocket: Arc with no air room is refused; each stand-in entry and a selected entry is a warning

**Decided** (the coordinator's review decision on PR #3899, from the
maintainer's rule on #3741: "an Outside pass never uses a helix or ramp
entry; the tool goes down in air outside the stock").

1. **Arc on an Outside ring pass with no air room is refused by name.** A
   plain Outside pocket with no stock known (or not round a part, or with
   the air room filled by fixtures on the avoid route) has no air room
   (#3746). The selected Arc entry then has no entry in air, so the
   operation is refused with `OUTSIDE_AIR_REFUSAL`: "entry refused: no air
   room for an Outside entry: set a stock, or choose an Inside start or a
   Helix, Ramp or Plunge entry". No automatic ramp or helix is posted.
   The check runs before the ladder (`entry_warn::refuse_outside_arc`,
   called by `pocket.rs` and `pocket_avoid.rs`), for the ring strategies
   (Waterline, Offset rings) and not for Zigzag or Adaptive.
2. **A selected Helix, Ramp or Plunge runs, with its one warning (#3748).**
   The advice note (`EntryNotRecommended`) is written on each Inside or
   Outside ring pass, with or without an air room: the air room is not the
   gate.
3. **A selected Plunge into uncut stock.** A start-side pass link that
   plunges into a pass start in material writes `PLUNGE_INTO_STOCK_NOTE`.
   The op posts `Warning::PlungeIntoStock` only where it posts no
   `EntryNotRecommended` for the Plunge, so the Plunge is named once.
4. **Stand-in entries that remain** (Inside with no air room; a restart on
   a ladder with an air room where no air lead fits) write the marker
   `ENTRY_STAND_IN_NOTE`, and the op posts one advisory
   `Warning::EntryStandIn { at, entries }`. `entry_status_lines` shows each
   stand-in note as a `warning:` line.

**Why.** Prime Directive: an entry that the user did not select and that
cuts into stock is refused, or it is visible. A warning on a ramp that
still cuts into stock on an Outside pass breaks the #3741 rule. The
refusal names the three fixes a user can make.

**Dropped.**
- The first form of this PR: keep the ramp stand-in on the Outside pass
  and post a warning. The review held that it breaks the #3741 rule.
- Refuse Arc on Inside with no air room: the #3741 rule names Outside
  only. Inside keeps its named stand-in with the warning.
- Make the warnings block the post: the entries are verified, and the
  selected entry is the user's.
- Warn on the stand-ins of Auto ladders: out of scope. Their notes are
  warning lines on the op page, but they post no operation warning.

**Cost to change later.** The rule is in `machining2d/entry_warn.rs`
(the refusal, the note markers, the stand-in tails and
`entry_warnings`). The tails match the note texts of `descent_dive`,
`descent` and `air_entry`; a new stand-in note adds its tail there.
