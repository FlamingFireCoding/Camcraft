## #1647 - Retract and identify mill setup changes

**Decided:** Haas, Mazak, Doosan DNM, Sinumerik, Heidenhain, grblHAL and
TinyG read `PARAM SetupChange`, sanitize the next setup's name with their
existing comment cleaner, and retract before naming that setup and writing
its M0 or M1. The setup marker resets with the after-operation request.
An ordinary operation stop keeps its existing output.

**Review fix:** The seven posts use `null` for an absent setup marker,
separate from its cleaned display text. A valid name such as `()` cleans
to an empty string; that still triggers the dialect's retract and displays
`UNNAMED SETUP` before M0 or M1. The marker still resets after the section.
Refusing these names was considered and dropped because a valid setup can
retain its safe retract and operator stop with replacement display text.
Changing that replacement later requires updating the posts and their
empty-name pins, without changing their retract policies.

**Why:** The app already sends the marker beside `AfterOperation` at a setup
boundary. Reading that marker in each dialect extends the existing Fanuc
and Tormach behavior without changing the stream or the app's stop policy.
The next section must state its own request, so a stale marker cannot turn
an ordinary stop into another setup-change stop.

**Retracts:** Haas and Mazak use their existing Z reference return,
`G91 G28 Z0`, followed by `G90`. Doosan uses the same return and cancels
active TCPM first, as its home path already does. Sinumerik uses its
existing `safeZ()` validation and machine-coordinate Z move. grblHAL and
TinyG reuse `retract()`, including its machine-safe-height or work-offset
clearance choice and its existing named refusals.

Heidenhain has no trusted machine-referenced home in this post. Its
setup-change move uses the current section's work-offset `ClearancePlane`
only when it is finite and above a finite `StockZTop`. Missing or unsafe
heights produce a refusal naming the operation. The lift preserves any
higher current Z, and the stock-top check resets for each section. This
avoids inventing a G53 home (G53 has another meaning in this dialect) or assuming a builder's
M91 datum. This clearance move is not a machine-home promise.

**Not applicable to the mill Z-lift contract:** `fanuc-lathe.cps`,
`tormach-lathe.cps` and `wire-iso.cps` keep their current stop behavior.
A lathe's Z direction runs along the spindle, not vertically clear of
the part. Fanuc's existing home path clears X before Z, while Tormach's
G30 path uses the stock envelope to order departure moves. A generic
wire EDM mounts no milling tool and finishes by cutting the wire; it
declares no trusted home position. Applying the mill's Z lift to these
posts would claim a clearance their motion models do not establish.
The app can send these posts `SetupChange`; this is a scope decision,
not an assertion that their setup boundaries are unreachable. Their
stops remain, but this fix adds no new setup-change notification to them.

**Considered and dropped:** A shared guessed retract would erase the
dialects' different coordinate and homing rules. Changing every M0/M1
would also alter stops unrelated to reclamping. Adding a new stream
record or changing app parameters is unnecessary because the existing
marker already distinguishes the boundary. Lathe and wire setup-change
handling needs its own verified departure contract rather than a copied
mill Z move.

**Cost to change later:** A different retract policy changes the relevant
post and its setup-change pins. Extending the same operator message to
lathe or wire posts requires tests of their own departure and wire state;
it does not require changing the existing `SetupChange` record.
