## #4149 - Mach3 post: extended work offsets with G59 P

**Decision:** Increase `workOffsetMax` to 255 in `assets/posts/mach3.cps`.
Format work offset numbers 1 to 6 as standard G-code words `G54` to `G59`.
Format work offset numbers 7 to 255 as extended G-code words `G59 P7` to `G59 P255`.
Refuse work offset numbers less than 1 or greater than 255 with an explicit error.
Update operation comment notes to display `G59 P<n>` for extended offsets.

**Why:** Mach3 supports 254 work coordinate systems. Offsets 1 to 6 map to `G54` through `G59`.
Offsets 7 and above use the `G59 P<n>` syntax up to `P255`.
The post previously stopped at `workOffsetMax = 6`. This prevented users from using extended fixtures.

**Considered and dropped:**
- Do not emit `G54.1 P<n>`. Fanuc controls use `G54.1 P<n>`, but standard Mach3 expects `G59 P<n>`.
- Do not wrap around or clamp offset numbers greater than 255. Refusing invalid values prevents machining collisions.
- Do not change engine behavior. `camcraft_post` already supports `MAX_BOUND = 255`.

**Cost to change later:** Minimal. `mach3.cps` isolates the formatting logic in helper functions.
Updating the syntax or range requires only local changes in `mach3.cps` and its unit tests.
