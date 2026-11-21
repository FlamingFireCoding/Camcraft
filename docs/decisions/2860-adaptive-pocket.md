## #2860 - Adaptive roughing and Pocket finishing

**What was decided:** Add an original U-shaped pocket with one open end.
Use 2D Adaptive before 2D Pocket through the existing commands.
Adaptive leaves a nominal 0.5 mm wall allowance and stops 0.5 mm above the floor.
Pocket uses zero wall allowance and reaches the model floor.
Keep the case-specific pin in a separate test file.

**Why:** The case demonstrates roughing followed by finishing with explicit stock allowances.
The existing commands can generate, verify, save, and post the example.
A separate test avoids extending the shared catalog test beyond its existing 431 lines and avoids overlaps with open PRs.

**Considered and dropped:** A closed pocket would not meet the open-pocket requirement.
Rest machining is not needed to demonstrate this sequence.
Algorithm changes are not needed because the existing APIs provide the required behavior.
Extending the shared catalog test would increase its length and create overlap with other work.

**Verification scope:** Verification checks for gouges, rather than complete stock removal.
The cutter leaves material in the model's sharp inside corners.
Adaptive also reports up to 0.71 mm of extra corner scallop beyond the nominal wall allowance.
The case README states these limits.

**Cost to change later:** Regenerate the mesh, project, and NC program.
Update the pinning test and the README to match the new geometry or allowances.
