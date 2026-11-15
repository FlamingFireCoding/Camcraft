## #1626 - Skip a repeated Tormach tool change

**Decided:** Remember the tool for which this program last emitted M6.
The first tool change remains unconditional because the post cannot know
which tool is already in the spindle. A consecutive request for that same
tool skips the pre-change retract and its G30, and skips M6. It keeps the
tool note, G43 compensation, the machine-Z lift, and the clearance move
before the first X/Y traverse. A different tool, including a return to an
earlier tool, still requires a tool change.

**Why:** G30 can lower Z from the top of travel to the operator's stored
tool-change position. Repeating that trip for the tool this program
already loaded adds motion without changing the tool. Tracking emitted M6
commands avoids relying on PathPilot to optimize a same-tool M6.

**Public-document reading:** Tormach's [post-processor guidelines](https://knowledgebase.tormach.com/pcnc-1100/pathpilot-post-processor-guidelines)
describe M6 as beginning a tool-change sequence, and its [770MX tools page](https://knowledgebase.tormach.com/770mx/pathpilot-tools-and-features-770mx)
says Txx M06 loads the specified tool. These readings establish no
guaranteed same-tool exception, so this policy does not rely on one.

**Considered and dropped:** Inferring the tool loaded before the program
starts would guess at controller state. Skipping compensation or the lift
would remove the established protection before the next traverse. Keeping
a set of every tool previously used would incorrectly suppress a change
back to an earlier tool after another tool was loaded.

**Cost to change later:** A controller-aware policy would need an explicit
source of trusted spindle state and tests for the initial tool change.
Changing compensation or clearance behavior would need separate safety
evidence and pinning tests. The current policy needs only one remembered
tool number per posted program.

**Review decision (PR #1741):** An emitted M0 or M1 makes the loaded tool
unknown, including a setup-change stop or a probe-result check. The next
section emits its tool change even when its tool number is unchanged.
An operator may remove or exchange the tool during a stop; the post cannot
trust the earlier M6 after control returns. Optional stops use the same
conservative rule because the post cannot know whether the control will
honor M1. Continuing to skip M6 after a stop was rejected because it could
apply G43 to an empty spindle. Changing this later would require trusted
controller spindle state. The `tormach_stop_tool` pins cover program stops,
optional stops, setup changes and probe checks.
