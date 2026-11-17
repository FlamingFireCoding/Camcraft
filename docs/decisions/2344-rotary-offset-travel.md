## #2344 - Explain Rotary Offset's A-travel limit at the refusal

**Decided:** use the issue's permitted actionable-refusal resolution.
The cumulative A overtravel diagnostic now tells the user to use a machine
with verified A travel wide enough for the whole operation. It also explains
that Rotary Offset cutting levels cover two laps plus seam padding.

**Why:** those cutting levels cannot fit the microARC machine file's stated
-360..360 travel. PathPilot counts A cumulatively. The existing refusal
remains necessary until the controller's wider travel is verified or the
roughing algorithm can safely use less travel. This change adds advice to
that refusal; it changes no toolpath, machine limit, feed or dependency.
The rough remains refused on the stated microARC limits.

**Considered and dropped:**

- Removing the seam copy risks incomplete coverage at the seam.
- Automatic rewind needs a separate clearance and motion safety design.
- Widening the microARC file without checking the controller would replace
  a known limit with an unverified one.
- Splitting the rough into angular windows is not actionable through an
  available angular-window control.
- Rotary Parallel is a finishing strategy, so it is not an equivalent
  replacement for this roughing operation.

**Overlap:** stacked on PR #2342 at `f29c92bd1`. Its `indexFor` function
introduces the cumulative-travel refusal that this patch clarifies.
PR #2346 inherits that function from #2342 without an independent change
to the refusal.

**Validation:** both positive and negative overtravel diagnostic pins failed
before the change. The boundary control passed. Restoring upstream's post
made the pins fail; restoring the fixed post made them pass. Restoring
#2342's post also failed both diagnostic pins while its boundary control
passed, isolating the text change. The quick gate against `upstream/main`
passed all 589 steps in 34.8 minutes. Clippy passed for the post and inherited
app crates, with all targets and warnings denied. This patch moves no NC or
CL golden relative to #2342.

**Not exercised:** generating the reported round-bar rough, running its
program in the PathPilot Hub, or cutting on hardware. The post pins use
small cumulative-A streams to test the refusal and travel boundaries.

**Cost to change later:** verify the controller's actual limits before
changing the machine file, or design and independently test seam coverage
and safe unwind behavior before changing the roughing path.
