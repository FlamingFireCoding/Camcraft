## #1385 - Indexed work offsets for grblHAL and TinyG

**Decided:** Both posts declare `workOffsetIndex = true` and
`workOffsetMax = 6`. They convert the engine-validated index to G54-G59.
Their `workOffsetName` functions use the same conversion for display.

**Why:** The engine already validates the offset and refuses an unsupported
number by setup name. The posts can use that contract without duplicate
guards. Their supported offsets and posted programs stay the same.

**Tests:** Append both scripts to the existing `work_offset_index` table.
This keeps the indexes that other tests use. The existing tests check the
flag, display names, output words, legacy words, and maximum-offset refusal.

**Dropped:** Do not add extended offsets without verified control support.
Do not refactor the existing oversized scripts in this change. Both changes
would expand the scope beyond the offset contract.

**Cost to change later:** Extended offsets need public control documentation,
a new maximum, a word conversion, and tests. A script refactor needs separate
coverage to prove that the posted programs stay the same.
