# Decision record - issue #2824

## #2824 - Rotary Flow kernel: surface and law choices taken unattended

**Decided:** the kernel's public surface is
`rotary_flow(chains: &[Vec<Point3>], facets: &[Triangle], ax: &Axis,
p: &RotaryFlowParams, c: &AxisControl) -> Result<Toolpath, GenError>`
with `RotaryFlowParams` carrying the `Tool` itself (the drilling
`CycleParams` precedent), so the ball-nose requirement is refused by
name from the tool's own kind. Three laws have a named constant
reading: the clearance and retract read as BALL-CENTER radii about
the stated axis (never world Z - a rotor may stand or lie); the
development's arc-length radius `R_ref` is the band's OUTERMOST chain
radius (a developed stepover then reads widest at the rim, and
ground inside it gets finer, never coarser); and one band per
ADJACENT chain pair is machined, both edges included (a shared
middle chain is walked once per band that uses it - the re-cut is
the same geometry at the same depth, safe and simple).

**Why:** the issue names the axis parameter (the #2772 seam), the
ball requirement and the nearest-whole-turn alignment but leaves
these readings open. The `Tool`-carrying params follow the only
in-crate precedent for a kind-refusing generator (drilling); the
ball-center radius reading is the only one that names a safe place
the same way for a standing and a lying rotor; the outermost-radius
development keeps the stepover promise one-sided (never coarser
than asked); adjacent pairs generalise the two-chain slice without
a second law.

**Considered and dropped:** carrying only `tool_diameter` and
refusing nothing (the issue asks for the non-ball refusal by
name); clearance as a world-Z height (wrong for a lying axis);
developing at the mesh's largest radius as `rotary_offset` does
(its ladder wraps the whole work, while a flow band's own radius
is where the passes run); skipping the shared edge pass between
bands (saved one pass, cost a second alignment law and a special
first pass per band).

**Cost to change:** the cylinder readings and the pair law live in
`rotary_flow.rs` and the emit module's doc header; the app wiring
sub-issue of #2804 will translate its own heights into these radii,
and a later change only touches that seam.
