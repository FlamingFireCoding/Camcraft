## #1496 - Keep Sodick taper refused until its stream mapping is confirmed

**Decided:** Keep the Sodick post's taper refusal. Also refuse every
`WireAngle` record, including zero and records without `WireTaper`, rather
than silently post an angle-word move as a vertical cut. No taper words
or new tapered golden are justified by the evidence available here.

**Public evidence:** Sodick Inc.'s January 2012 *Wire EDM Machine
Operation Training*, LN1W / LN2W, printed pages 66-70, describes G51 as
upper-guide movement left of travel, G52 as right, A as degrees, and G50
as cancellation that returns the wire upright during the next XY move.
It describes T-Limit, T-Upper and T-Lower as guide settings, TP as the
table-to-program distance, and TN as the opposite plane's height.
[Public copy](https://fab.cba.mit.edu/content/tools/sodick_edm/sodick_user_manual_small.pdf).
This is a newer control than this post's Mark 21 / EX21 target; it does
not establish compatibility with that target. The existing post cites
the 1998 manual; attempts to retrieve its taper pages during this work
failed. No other CAM product's post or output was used.

**Why:** The repository's `guided` path in
`crates/app/src/posting_wire.rs` sends lower-guide XY and upper-guide
offsets. Replacing these vectors with the nominal `WireTaper` angle
would discard the explicit guide motion without proving equivalent
corner, lead-in and cancellation interpolation. Its `angled` path
instead sends programmed top-plane XY and signed `WireAngle` values.
`angle_lean` computes `WirePlane` as top Z minus work-origin Z. That
work-relative coordinate does not establish a distance from the Sodick
table. These are conclusions from our stream implementation, not claims
about undocumented controller behavior.

**Still unconfirmed for Mark 21 / EX21:** Exact TP/TN initialization and
its correspondence to machine guide calibration and the work datum;
corner interpolation and lead-in/cancellation equivalence; and the
signed angle mapping in that control's coordinate frame. The newer
manual confirms the left/right convention on its own controls, so the
direction words themselves are not generally unknown.

**Dropped:** Copying `WireTaper` into A while retaining lower-guide XY;
treating origin-relative `WirePlane` as table-relative TP; or inventing
machine code templates as evidence that Sodick accepts the mapping.

**Cost to change later:** Obtain public Mark 21 / EX21 documentation or
machine-owner confirmation for the remaining mappings, verify the
corner and transition behavior against the generated wire path, then
add a tapered punch through the app's post gate and its sourced golden.
