## #1674 - Keep the entry ramp angle independent of the links

**Decided:** An Inside Contour Ramp entry with no explicit entry angle uses
`DEFAULT_RAMP_ANGLE` (3 degrees). The shared descent ramp fallback uses the
explicit approach angle when set, otherwise the same 3 degree default.
Neither path takes the link ramp angle. Explicit entry angles and explicit
level transition angles keep their own behavior. The shared fallback also
applies this default to a helix that falls back to a ramp.

**Why:** Entry and link angles describe separate motions. A 45 degree link
setting must not silently steepen an entry whose angle was left unset.
Using the existing default fixes the affected generator paths without a
project schema change or a global change to legacy approach defaults.

**Considered and dropped:** Taking the smaller of the link angle and the
default would keep two separate settings coupled. Changing `Approach::DEFAULT`
globally, or adding schema and UI fields, would broaden this fix and change
other legacy entry behavior.

**Evidence:** The Contour, Circular and Bore pins failed before the fix.
All five `entry_ramp_angle` tests pass after it, including an explicit
5 degree entry and a separate 45 degree transition. Restoring each old
production file independently makes its affected pins fail; restoring the
fix makes them pass. The quick gate and motion clippy checks pass.

**Cost to change later:** Small generator and pin changes. No saved-project
migration is needed.
