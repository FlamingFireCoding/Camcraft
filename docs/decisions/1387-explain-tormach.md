## #1387 - Explain G-code for the Tormach mill

**Decided:** declare `explains = "paren"` in `tormach.cps` and append the
shared `explain(a)` result in `block()`, after the executable words.
The helper supplies the comment guards and returns no text when Explain
is off. The default program therefore keeps its existing bytes.

The post supplies original wording for `G38.2` ("PROBE TOWARD CONTACT")
and `G59.1` through `G59.3` ("WORK OFFSET 7" through "WORK OFFSET 9").
Each entry carries a source note naming the relevant public LinuxCNC
G-code reference section. Other meanings come from the shared engine
table, whose provenance names RS-274 and the LinuxCNC reference.

**Why:** the existing helper already keeps comments inert, bounds their
length and preserves the executable words. The post only needs to opt in
and state the additional words it uses.

**Considered and dropped:** duplicating the engine dictionary would make
wording and guards drift between dialects. Rewriting the completed program
to add comments would create another parser and risk changing executable
text. Changing the engine is unnecessary for this dialect's opt-in.

The optional lathe change is deferred. Open PRs #1705, #1681, #518, #517
and #468 touch `tormach-lathe.cps`, and the lathe needs its own word
semantics. This change stays with the required mill post and its matrix.

**Cost to change later:** another mill word needs a table entry with a
public source note and a pin. The lathe can opt in separately after its
overlapping work settles, using its existing tool and feed-mode readings.
