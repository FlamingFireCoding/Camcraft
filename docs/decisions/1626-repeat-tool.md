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
