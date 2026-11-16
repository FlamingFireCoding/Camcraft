## #1862 - A DXF profile on the stock of a setup, and the virtual model reference

The maintainer's words (issue 1862, 2026-11-15): "A DXF should be possible to
"place" on a stock and used as references for cutting - not be treated like a
model - thus for processes like wireedm, plasma, laser, etc etc.. we can just
make a sheet, cube, cylinder. place the DXF - and select operations accordingly.
The Adding DXF Profile, should thus live on "Stock page" and have a "virtual"
model reference, just so the operations pages gets happy about a model being
loaded.."

### What a profile is in the ECS

- **A `Sketch` component on a named entity** (`crates/app/src/schema.rs`),
  beside `Name` and `Location`. It holds `setup: Option<String>` (the
  declared-vs-effective law of `Operation::setup`) and `data:
  Arc<SketchData>`. The entity never has `Facets`, and `model::is_model` is
  false for it, so the part readers (`model::model_entities`) and each 3D
  solid tool do not see it.
- **`SketchData` is in `camcraft-geometry`** (`sketch_data`), not in `app`:
  it works only on domain values. It holds the primitives in millimetres, the
  layer names, the source units (the file's `$INSUNITS` and the unit applied),
  the join tolerance, the chains of issue 1861 and the import summary. Its
  fields are private and the two constructors make the chains, so the stored
  chains are always the chains of the stored primitives.
- **The `Location` of a profile is relative to the stock**, as the scope
  change says: the frame is the top face of the stock of the setup, at the
  stock origin, turned with the stock (`sketch::face_frame`). The stock origin
  is the origin of the stock entity (for a box and a cylinder: the centre in
  XY). The top is the box height, the cylinder height, or the highest facet Z
  of a stock made from a model. A profile has Z 0 and a turn about the face
  normal only. Because the placement is relative, a moved or resized stock
  carries the profile, and nothing must follow it by a second command. The
  stock is setup furniture, so a setup's part alignment does not move the
  profile.
- **Deleting the entity frees the drawing.** The component is a Rust-owned
  type; the last `Arc` holder drops the data
  (`deleting_the_entity_frees_its_drawing`).

### The commands

- **`ImportDxf::new(world, setup, path, options)`** reads, checks and chains
  the file at construction. `setup` is the new argument (`None`: the active
  setup). A setup without a stock is refused ("make a stock first"). A file
  with no unit and no unit from the caller is the typed
  `CommandError::DxfUnitsMissing`; it has the drawing extents, so the UI of
  issue 1863 can show them in each unit before it asks. A unit from the caller
  wins over the unit of the file (the argument is an override).
- **`MoveSketch` moves a profile; `MoveModel` refuses one by name.** The issue
  body said `MoveModel`. That was written before the scope change. `MoveModel`
  writes a world placement and carries the stock; a profile has a face
  placement (X, Y, angle) and rides the stock. Two verbs on one object would
  split the undo story (the "pages own their verbs" law of `MoveModel`), so
  the profile has its own command and the placement cannot leave the face.
  Dropped: a `MoveModel` that accepts a profile and silently drops Z and tilt.
- **A name from the file stem is made unique** (`plate`, `plate (2)`), so one
  drawing can be placed more than one time on a sheet. A stated name that is
  in use is refused.
- **`DeleteSetup` refuses a setup with a placed profile** (as it refuses one
  with operations). Dropped: deleting the profiles with the setup, which loses
  work without a word.
- The Z of the drawing (`Document::elevation`) is not used: the profile lies
  on the stock face.

### What the virtual model reference is

- **It is derived, not stored**: `sketch_model::of(world, setup)` is `Some`
  when the setup has one or more placed profiles and the document has no part
  (a model that is not a fixture). No component and no file key holds it, so
  it cannot disagree with the document. Dropped: a tag component or a
  placeholder model entity. A placeholder entity must be kept in step by each
  command that adds or removes a model or a profile, and a facet-less "model"
  in `model_entities` is the zero-triangle mesh that the issue forbids.
- **It has no facets.** It names profiles only.
- **An operation refers to a profile as `SrcRef::Entity(<profile name>)`.**

### Which checks read it

| Check | Reads | Result on a virtual reference |
|---|---|---|
| The source door of each operation command (`gates::valid_src`, `sketch_gate`: create, paste, retarget, retype, move to a setup) | `sketch_model::source_problem` | A profile is a source for the 2D milling, plasma and wire EDM sections, in its own setup. Each other kind refuses: "needs a solid model - '<name>' is a DXF profile, a virtual model reference with no solid". |
| The same door, 3D milling and multi-axis sections, each source | `sketch_model::of` | Refused: "a <kind> operation needs a solid model - the setup has only its stock and the DXF profile ..." |
| The model-height diagnostic (`operation::missing_source`, Top/Bottom of Model) | `sketch_model::of` | The text names the virtual reference and says to use a stock or fixed height. It is not "no model loaded". The height has no value: a reference with no solid has no top and no bottom. |
| The generation door (`operation::collect_job`) | `sketch_model::generation_problem` | Refused by name with what the operation is checked against (the stock and the chains). |
| `get_job` (MCP) | `sketch_model::of`, `checked_against` | Each setup has `model_reference` (`solid`, `virtual` with its profiles, or null); an operation on a profile has `checks_against`. |
| The operation pages (issue 1863) | `sketch_model::model_loaded` | True for a part or a virtual reference. |

Checks that need a solid part keep their refusal: part alignment and locate
("import a model first"), fit stock to models, the WCS box points of a model,
mesh repair, fixture tags, stock from a model.

### Why nothing passes silently

This PR does not make a toolpath from profile chains: that is issue 1864
(chain picks, side and direction per chain, stock and crossing checks without
a mesh). Until then generation of an operation on a profile is refused by name,
so no gouge, collision or verify check can pass because the model is virtual.
The operation has no toolpath, and the post gate refuses it.

### What the project file stores

- A profile is an entity row with `kind: "sketch"`. Its `translation` and
  `rotation` are the face placement. Its `sketch` key has: `setup`, `insunits`
  (the code the file stated, 0 for none), `unit` (the `$INSUNITS` code of the
  unit applied), `join_tolerance`, `layers`, `primitives` (mm, radians; each
  names its layer by index), and the import counts (`entities`, `vertices`,
  `skipped`).
- **The chains are not stored.** They are made again from the primitives at
  load, by the same `chain()` call as at import. Dropped: storing the chains.
  A stored chain can disagree with its primitives after a hand edit, and the
  chain identities of issue 1861 do not depend on the order of the input.
- **The virtual model reference is not stored.**
- Each profile is read and chained before any world write. A profile off its
  face, with an unknown unit code or with primitives that do not chain refuses
  the file by name.
- **`PROJECT_VERSION` 63 -> 64.** Main uses v63 for the through-hole
  `break_through` field from #1557. Profiles use the next version, v64.
  A v63 file keeps its breakthrough values and steps up unchanged. A v62
  file still loads through the earlier migration. An older reader refuses
  a v64 file before it can discard a profile. Dropped: sharing v63 with
  main, which lets a reader discard a field that controls tool travel.

### What it costs to change later

- A stored reference or stored chains need a new project version and a
  migration.
- If Top/Bottom of Model must have a value on a virtual reference (for
  example the stock top and the stock bottom), `operation::resolve_height`
  gets one arm; operations saved before that keep the refusal until then.
- A profile on a different stock face (a side, the bottom) needs a face
  choice on `Sketch` and in the file.

### PR #2134 review fixes

- **Reserve the stock source word without regard to letter case.** Explicit
  profile names such as `stock` and `STOCK` are refused. The same file stems
  use the existing `profile` fallback and unique-name handling. The MCP
  source reader treats these words as stock, so accepting them as profile
  names could silently select different geometry. Dropped: accepting
  ambiguous aliases or changing the source-reference format for this fix.
  An unambiguous source-reference format later needs coordinated changes to
  the source reader and its callers.
- **Validate summary counts in both `SketchData` constructors.** Entity and
  vertex counts must fit the default DXF import limits. The skipped counts
  are added with checked arithmetic, and their total must fit the entity
  limit. This check runs before the empty-drawing refusal. The validated
  total is retained for `profile_view`, so it never adds untrusted counts.
  Invalid stored summaries become the existing project error naming the
  profile before any world write. Dropped: unchecked addition, a panic, or
  saturating addition that hides invalid saved data. Changing these limits
  later requires changing the constructor validation and its pins; no new
  project field or version is needed for the derived total.

### PR #2134 merge refresh

- Keep both `rest_dto` and `sketch_dto` in the project module. They store
  separate data. Dropped: taking either module list alone, which loses the
  other feature's file types.
- The first refresh kept project version 63 because main still wrote
  version 62 after #1817. Its rest parameters were additive optional keys.
  That decision is superseded by the next refresh below.

### PR #2134 breakthrough format refresh

- Merge main's v63 breakthrough format and give profiles v64. Keep both
  project modules and all breakthrough fields. The v63-to-v64 migration
  preserves the document data. The v62 migration remains in the chain.
  Dropped: retaining version 63 for profiles, because two formats with one
  version let an older reader remove a stated breakthrough on save.
- Pin a v63 breakthrough file through load, save as v64, and load again.
  Pin the profile and rest payloads in the v64 round trip. These checks
  protect the stored travel limit and the profile data together.
