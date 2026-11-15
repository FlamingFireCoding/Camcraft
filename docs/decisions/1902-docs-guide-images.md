## #1902 - Reproducible off-screen User Guide screenshots via ShellUi + View3D

- **What was decided:**
  1. Render all 13 User Guide screenshots off-screen from the real `ShellUi` + `View3D` egui harness using `egui_kittest::Harness` (at `1200x760`, quantized to an indexed PNG palette and verified under `5_000..=650_000` bytes and `1000..=1400` px wide), driven by `cargo xtask docs-images` (and `--check` for validation).
  2. Commit three lightweight clean-room parametric demo projects alongside `examples/first-part.camproj`: `examples/guide-workholding.camproj`, `examples/guide-3d.camproj`, and `examples/guide-multiaxis.camproj`, plus a shared CSS `.guide-pair` side-by-side layout rule in `site/static/style.css`.
  3. Keep UI/harness generation inside `crates/ui/src/egui_app/tests/` (`guide_projects.rs`, `guide_images.rs`) behind explicit `#[ignore]` rendering tests invoked via `CAMCRAFT_GUIDE_IMAGES_OUT` and `CAMCRAFT_GUIDE_PROJECTS_OUT`, while enforcing manifest/PNG/Markdown invariants in non-ignored unit and `xtask` integration tests.

- **Why:**
  - Using the real `ShellUi` + `View3D` off-screen software renderer (already active in `camcraft-ui` snapshot tests without X11/Wayland) produces authentic, drift-proof screenshots that reflect real UI typography, theme tokens, toolpaths, stock/vise geometry, and collision state rather than hand-taken or external screenshots.
  - Separating fast verification (`cargo xtask docs-images --check` and `demo_projects_build_and_load_with_valid_paths`) from image regeneration (`#[ignore]`) keeps normal `cargo test` fast while giving contributors a single repeatable regeneration command.
  - Parameterizing `examples/guide-*.camproj` strictly from clean-room primitives and `camcraft_app::commands` guarantees zero third-party geometry or vendor trademarks.

- **Considered and dropped:**
  - Launching the native windowed binary against Xvfb/Wayland for screenshots: fragile across headless CI and contributor environments, non-deterministic across GPU drivers.
  - Shipping raw unquantized RGBA PNGs or higher-resolution captures: bloated git diffs (> 1 MB total vs ~1.4 MB / ~60 KB–220 KB per image after quantization) without visual improvement at documentation column width.
  - Storing synthetic SVG mockups for UI views instead of rendered screenshots: misses actual panel/viewport behavior and drifts silently when widgets change.

- **Cost to change later:** Low — the manifest (`MANIFEST` and `DEMO_PROJECTS` in `xtask/src/docs_images.rs`) and scene camera/parameter recipes (`crates/ui/src/egui_app/tests/guide_images.rs`) are declarative tables that can be re-rendered anytime with `cargo xtask docs-images`.


### PR #2078 review correction: identify unsafe multi-axis examples

- **What was decided:** Keep the existing Indexed Face1 and Swarf1 images as
  explicitly unsafe examples. Add adjacent warning counts, explanations and
  required setup corrections, and identify the unsafe state in their alt text.
- **Why:** The review permits an explained unsafe example. The images show the
  actual demo warning state, so the guide must not present them as verified
  machining recipes. Require regeneration and verification after corrections;
  a stock simulation result of "not judged" cannot establish safety.
- **Considered and dropped:** Changing demo geometry or toolpath settings to
  make these images normal examples would require proving a different demo
  safe and replacing images. The narrow review fix instead documents the
  existing unsafe state without changing geometry, safety checks or images.
- **Cost to change later:** Low. Replace the examples and their notes together
  only after regenerated operations have been independently verified.
