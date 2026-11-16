## #1463 - Retry the machining mesh at half the chord

**Decided:** If a finite measured deviation exceeds the requested tolerance,
try one more plan at half the chord. Keep the existing floor of one millionth
of the body size. Keep the caller's tolerance and all planner-statistics,
triangle-budget, deviation, topology, volume and cancellation checks.
Record the finer chord in the mesh report when the retry succeeds.

**Why:** The display planner measures normal distance at triangle midpoints
and centroids. The verifier uses denser samples and a conservative parametric
distance on rational-spline faces. The two measures can disagree.
At 0.005 mm, nine rational-spline faces of the Solidcraft sample fail.
Face 50 has the largest measured deviation, 0.00684674657 mm.
Planning at 0.0025 mm reduces that maximum to 0.00323090453 mm.
The accepted mesh has all 53 faces, 279,600 triangles and no open sides.
A 0.1 mm request still fails after the retry: face 36 measures 0.13365682857 mm.
These sampled checks do not prove the continuous closest-surface distance.

**Dropped:** Always planning finer would charge every input for the retry.
Relaxing the verifier would weaken the machining guarantee.
A new closest-point solver or dependency update exceeds this fix's scope.
Unlimited retries would remove the bound on repeated planning.

**Cost to change later:** Change the bounded refinement policy and rerun the
sample, refusal and old-code mutation checks. Do not infer that other
previously refused models now pass.
