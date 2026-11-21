## #3252 - Preserve a STEP edge when healing increases the endpoint error

**Decision.** Compare the solid before and after healing when the healed STEP solid has fewer edges than the source record and fails the 0.001 mm vertex-to-endpoint limit. Use the original topology only if it has a smaller maximum gap and passes the import checks. Those checks require a closed manifold solid, positive finite volume and area, a 3D curve for each edge and a gap within the limit. Also require Solidcore's whole-body level 2 validation before accepting that solid. Report both gaps and edge counts when the original gap also exceeds the limit.

**Why.** On the reported Fusion part, healing reduced 999 edges to 998. The adjacent spline then used the other end of an explicit short edge as its vertex. The maximum gap increased from 0.007736 mm to 0.011573 mm. Keeping the two vertices fixes that change in topology. The original gap still exceeds the machining limit. In a separate probe the mesher measured 0.011141 mm deviation on one face at a 0.005 mm target.

**Alternatives.** Skipping healing on every import would also skip repairs on unrelated solids. Increasing the global gap limit would admit larger source errors and would leave the separate mesh failure on this part. A reduced-fidelity view needs its own decision about how to communicate the achieved accuracy to the user and to downstream checks.

**Cost to change.** The extra comparison requires a second kernel build for the specific refusal above. A future Solidcore API that can preserve an individual explicit edge during healing could avoid that second build. Keep the endpoint and topology regression tests when changing the repair path.
