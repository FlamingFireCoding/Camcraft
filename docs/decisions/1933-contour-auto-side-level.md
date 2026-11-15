## #1933 - Side Auto of a picked Contour reads the material at the ring's level

**Decided.** Side Auto of a 2D Contour on picked rings (chains, face loops,
wall patches) reads the part's section at the rings' own height, not the
full XY silhouette. The law is `machining2d::auto_side`
(`crates/motion/src/machining2d/contour_auto.rs`):

- The Z span of the picks is read in the setup frame (or along a tilted
  frame's axis). A flat rim has one Z; a wall patch spans its height.
- Two probe sections are taken: 0.01 mm under the top of the span, then
  0.01 mm over its bottom (`AUTO_PROBE_DZ`).
- At each probe, the area inside the rings and a 0.5 mm band just outside
  them (`AUTO_BAND`) are compared with the section. A side "holds
  material" at a fill of 0.5 or more (the threshold of the old fill law).
- Material inside only: Outside. Material outside only: Inside. The first
  probe that decides wins.
- The probes disagree, or neither decides (material on both sides, or on
  neither): the generate is refused by name, "Side Auto cannot tell ... -
  set Side to Inside or Outside". Auto never guesses.
- No readable part (a sketch, no facets) or no pick height: Outside, the
  side that Auto always took for an outline.

**Why.** A blind bore's floor fills the XY silhouette of its rim, so the
old fill law read the rim as material and resolved Outside, which the
#1927 law then refused. At the rim's own level the bore is air and the
plate is metal, which is the material that the wall bounds. The probe
under the top suits a mouth rim; the probe over the bottom suits a floor
rim (both sides are metal under a floor).

**Considered and dropped.**
- A verdict for each picked ring. Dropped: one Contour op has one Side,
  and a mixed verdict would need a per-ring side in the generator.
- Keep the silhouette law and add the bore floor as a special case.
  Dropped: the section at the level is the general rule and also covers
  steps and pockets in a pocket.
- A wider band or a probe deeper under the rim. Dropped: a deeper probe
  can pass the floor of a shallow counterbore and read the wrong wall.

**Known limit.** Picks at several levels share one verdict, read over
their combined span. If the probes disagree, the op is refused by name.

**Cost to change.** Local: the law is one function with its own tests
(`crates/motion/tests/contour_auto_side.rs`,
`crates/app/tests/contour_auto_level.rs`).
