## #2354 - Refuse unverified Sinumerik G73 dwell

**Decided:** Refuse a positive dwell on a Sinumerik PECK/G73 cycle with a
named, counted diagnostic before positioning or writing the cycle. Keep
zero-dwell G73 and the existing G83 dwell output unchanged. Other posts
with the same omission are tracked by #2356.

**Why:** The maintainer's 2026-11-17 decision was "Not verified: leave it"
for G73 P support. The control check covered G83 only. Refusing the dwell
prevents a silent change to the requested program without inventing an
unverified G73 word.

**Considered and dropped:** Posting G73 P based on the G83 check would
extend that evidence to a different cycle. Silently omitting the dwell
would retain the defect. Expanding the cycle or substituting another
cycle would change behavior beyond this refusal fix.

**Cost to change later:** Once G73 dwell support is verified, replace the
refusal with the verified output and update the refusal pins to check
that output. Keep the zero-dwell and G83 regression coverage.
