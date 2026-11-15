## #1917 - "Your first router part" guide page, AltMill demo project, and RichAuto DSP A11 notes

- **What was decided:**
  1. Placed the new walkthrough at `site/content/guide/getting-started/first-router-part.md` (`weight = 5`) and committed `examples/first-router-part.camproj` (`Sienci AltMill 4x4` + `grblHAL`, `140 x 100 x 12 mm` bracket plate in `200 x 160 x 12 mm` box sheet stock, corner `G54` at `(-100, -80, 12)`, `DR-0600` + `EM-0600-4FL`, dry `Drill1` -> `2D Pocket1` -> `2D Contour1` with 4 holding tabs).
  2. Chose 12 mm sheet stock thickness and ordered operations `Drill1` (`T1 DR-0600`) -> `2D Pocket1` (`T2 EM-0600-4FL`) -> `2D Contour1` (`T2 EM-0600-4FL`, 4× `8 x 3 mm` tabs) with `Coolant = Off` and `Ø6.2 mm` (`r = 3.1 mm`) 24-gon holes.
  3. Kept the RichAuto DSP A11 notes scoped to controller prep (`FAT16`/`FAT32`, `8.3` filenames, `G Code Setup` with `AbsCntr = Off`, `C.A.D Thickness` Z-puck zeroing, and single-tool `.nc` export) while explicitly noting that `richauto.cps` and Laguna/Grizzly machine profiles (`#1914`, `#1915`, `#1916`) are not on `main` yet.
  4. Added `guide/getting-started/first-router-part` to `test_sidebar_groups_follow_a_job_on_every_guide_page` in `site/tests/test_guide_navigation.py` rather than adding a synthetic entry to `MOVED`, and backed all checks with a Rust integration test in `crates/app/tests/first_router_part.rs`.

- **Why:**
  - `EM-0600-4FL` in `assets/tools/catalog.json` has `flute_len = 15.0 mm` and an `8.0 mm` shank (`shank_d > diameter`). Using `12 mm` sheet stock lets readers use the stock catalog cutter cleanly without triggering a shank-reach refusal or fabricating a custom bit first.
  - Drilling first while the sheet is rigid and grouping `2D Pocket1` and `2D Contour1` afterward uses only one manual tool change (`T1` -> `T2`), keeps perimeter holding tabs intact until the last pass, and splits cleanly into two single-tool files for manual-collet RichAuto DSP A11 routers.
  - Using `r = 3.1 mm` on a 24-gon hole ensures the inscribed chord midpoint (`3.1 * cos(pi/24) = 3.074 mm`) stays strictly outside the `3.0 mm` radius of `DR-0600` at breakthrough, avoiding polygonal facet grazes in stock verification.
  - Putting a new page in `MOVED` would have asserted a pre-#1903 `/guide/<slug>/` redirect alias that never existed.

- **What was considered and dropped:**
  - **18 mm sheet with custom extended-flute end mill:** dropped so the guide uses built-in catalog tools (`EM-0600-4FL`, `DR-0600`) out of the box; changing later only requires updating the stock height and pocket/contour depths.
  - **Cutting pocket -> drill -> contour:** dropped because it would add an extra `T2 -> T1 -> T2` tool swap on a manual collet router.
  - **Running `zola` and `python3` in CI here:** `zola` is not installed on PATH in this environment and `python3` is blocked by sandbox policy, so Zola link and sidebar checks are additionally pinned by `crates/app/tests/first_router_part.rs` until CI runs `test_guide_navigation.py` with Zola.

- **Cost to change later:** Low. All documentation is additive markdown under `site/content/guide/`, and `examples/first-router-part.camproj` can be regenerated directly from the command sequence in `crates/app/tests/first_router_part.rs` once `#1902` captures UI screenshots or `#1914`/`#1915`/`#1916` ship RichAuto/router profiles.

- **PR #1920 review corrections:** Use `XY→0` to set the A11 X/Y work origin, with the OEM manual sections 2.3 and 5.4.1 as the source. Dropped the `ORIGIN` instruction because `ORIGIN/OK` returns to the existing origin. State the demo's `G54 Z = -13 mm` drill breakthrough in the setup, drilling and clearance instructions. A contour-only `-12 mm` clearance check omits the drill's `1 mm` penetration below the stock.
