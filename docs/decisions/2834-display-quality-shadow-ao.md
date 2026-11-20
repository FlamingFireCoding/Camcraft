## #2834 - Display menu: quality preset, ground shadow, ambient occlusion, Look at a work plane

**What was decided**

- The Graphics group of the Display menu has four new rows: the presets
  "Performance" and "Quality" (one choice), and the switches "Ground shadow"
  and "Ambient occlusion". The Look at group has a new entry, "Look at a
  work plane".
- A preset is a set of values for the two switches. "Performance" sets the
  two off. "Quality" sets the two on. No preset is stored. The active preset
  is the preset whose values the switches have now. A switch changed by hand
  leaves no preset active, and the menu cannot show a preset that the view
  does not have.
- The two switches are view state: direct field writes, no command. They are
  saved in `view.json` (`ViewPrefs::ground_shadow`, `ambient_occlusion`).
  An old file without the keys loads them off. The default is off, so the
  view and the pixel snapshots of #2818 do not change.
- Ground shadow: the model bodies are projected straight down on the floor
  (`floor::floor_z`). The footprint is sampled on a grid of 96 nodes at
  most, blurred two times, and drawn as blended triangles with the shade on
  each vertex. The pass is `Pipe::GroundShadow`: after the model, before the
  translucent stock.
- Ambient occlusion: one value for each model vertex, baked on the CPU. The
  value is the share of ten short rays over the face that meet the body
  again, marched through a surface voxel grid of 48 cells. The pass is
  `Pipe::FacesOccluded`: the face shader with the value on its light. The
  values are a second vertex buffer, so the model vertex buffer and the
  other pipelines do not change.
- Caches (`shade_cache`, held on `State::frame_cache`): the shadow keys on
  the facet allocation and the shown pose of each body and on the floor
  height. The occlusion keys on the facet allocation of one body, so a moved
  body keeps its values. A switch that is off builds nothing. Each cache has
  a counter test.
- Wireframe draws no lit face and no shadow. The two switches are disabled
  there, keep their values, and their tooltip says why.
- No entry is hidden on wasm. The two passes are vertex buffers and alpha
  blending in the one wgpu pass that the two shells share. They use no
  texture and no second render target.
- "Look at a work plane" arms one click on an origin axis in the view. The
  view turns to face the plane that the axis is normal to: Z gives XY, X
  gives YZ, Y gives ZX. A click on the origin point gives the XY plane. The
  origins are the work offsets (WCS) and the world origin. The reversed axes
  of a WCS (#1720) are applied. The view does not pan or zoom. A miss
  changes nothing and says so.
- `world_verts`, `world_verts_by` and the new `model_src` moved from
  `egui_app/world.rs` to `egui_app/world_verts.rs`. `world.rs` was at 392
  lines and the scene shade data put it past 400.

**Why**

- The issue asks for data that is calculated one time for each geometry
  change and cached. Baked vertex values and a baked shadow mesh are that
  data. A still frame adds one blended draw and one extra vertex attribute.
- The issue asks that the wasm build uses the same settings. Plain vertex
  buffers work on each backend, so the menu says nothing false there.
- The light of the shadow is at the zenith. The first version used the key
  light of the face shader. That light is behind the camera in the house
  view, so the shadow was behind the model and nobody saw it. The straight
  shadow shows as a soft rim from each view direction.
- A derived preset has no state that can disagree with the switches.

**Considered and dropped**

- Screen-space ambient occlusion (a depth texture and a second pass). It
  costs work on each frame, it needs a sampled depth target, and it is not
  "data calculated one time for each geometry change".
- A 3D occupancy texture sampled in the fragment shader. It gives a value
  for each pixel, but it needs a texture binding in the shared pass and one
  volume for all bodies in world space, built again on each pose change.
- A shadow map (a depth pass from the light). It needs a second render
  target and a sampled depth texture. The projected mesh needs neither.
- Occlusion between bodies (a part in a vise). The values are for one body
  alone, so that a moved body keeps them.
- A stored preset name in `view.json`. See "Why".
- Construction planes. The document has no construction plane entity. The
  world origin planes and the WCS planes are the reference planes that
  exist. A construction plane type can add its axes to `axis_marks` later.

**Limits**

- The occlusion is a value for each vertex. On a large flat triangle the
  shade is a linear ramp between its corners, so a coarse CAD face shows a
  soft diagonal ramp and not a narrow dark line in the corner. A fine mesh
  shows the corner well.
- The first frame after "Ambient occlusion" goes on bakes each body on the
  UI thread. The cost grows with the triangle count. It was not measured on
  a large model.
- The shadow is black with 60% alpha at most. It is faint on the dark
  viewport ground and strong on a light ground.
- The shadow of the simulation tool body is not drawn: the tool moves on
  each playback frame and would build the shadow again on each frame.

**Cost to change later**

- The two passes are behind `Pipe::FacesOccluded` and `Pipe::GroundShadow`
  and the `Shade` data. A different occlusion source fills the same
  `Shade::occlusion` values or replaces the one pipeline. The preset list is
  `QualityPreset::ALL`; a new switch adds a field to `Effects`.
