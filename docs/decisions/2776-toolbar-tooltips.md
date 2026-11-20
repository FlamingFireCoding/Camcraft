## #2776 - Explain the bottom toolbar buttons on hover

**What was decided:** Give each of the 13 buttons an original name and one
sentence that describes its action. Use an egui hover response with the
standard tooltip behavior. Honor the existing hover suppression during a
drag. Keep the existing button actions and layout.

**Why:** A user must know what an icon does before clicking it. The tooltip
explains each control without taking space from the model view.

**Considered and dropped:** Permanent text labels would need more width in
the compact toolbar. A custom popup would duplicate the standard egui
tooltip behavior and add maintenance work.

**Cost to change later:** Names and descriptions need local text edits in
the toolbar module and corresponding hover test expectations. No saved
project data or command interface depends on this wording.
