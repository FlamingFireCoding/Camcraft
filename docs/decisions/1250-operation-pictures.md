## #1250 - Add a path picture to each operation reference

**Decided:** Give each of the 44 current `OpType` variants a schematic
picture in its existing reference section. Keep the pictures in one SVG
asset with a named view for each kind. Describe each picture in its image
alternative text and SVG description. Keep the existing parameter and
refusal reference text.

**Why:** The earlier reference pages explain the operation families but
have no pictures. A small schematic makes the path pattern visible next
to the settings. The pictures come from CamCraft's operation descriptions
and general machining knowledge. They use no vendor help text or images.
The legend states that these are illustrations without scale, not generated
NC paths. The change does not alter cutting settings or generation.

**Considered and dropped:** App screenshots need model fixtures and can
make the reference depend on a specific UI version. Generated NC previews
would need a complete machine and tool setup for every family. Separate
image files would repeat the common drawing styles. The named SVG views
keep the diagrams together and let the coverage test check every kind.

**Cost to change later:** Replace an individual diagram and its alternative
text when the operation changes. If the operation enum gains a kind, add
its reference section, picture view and coverage mapping. Replacing the
schematics with generated previews can keep the same section locations.
