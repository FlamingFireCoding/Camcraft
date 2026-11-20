# #2022 — Wire punch from a placed DXF racetrack

Use the four source primitives of one complete DXF racetrack as the nominal
through-punch. Prove its tangent joins, positive radius, retained material,
placement and stock limits before generating the ordinary Wire Contour. Use
the intersections of adjacent circle tangents for the verification mesh.
The radial excess is bounded by 0.000025 mm. This keeps the retained circular
material inside the material checked by the verifier.

The Agie file uses the source arc centers and nominal endpoints. Posting
requires cardinal straight faces and grid-representable coordinates so the
formatted endpoints, radii and I/J centers stay consistent. Use the existing
perpendicular straight entry and capture sequence. Keep mixed drawings,
selected subchains, die openings from sketches and laser
operations for separate qualification.

Accept an independently qualified single DXF CIRCLE of radius 10 mm. Its
threading hole lies on the negative-X radial line. The native printer uses
two semicircles centered on the source drawing and the same 1 mm resident
separation. On the control each bridge subtended 0.1 radians, with the main
cut covering the remaining angle. Limit this version to the proven radius and
entry side. An unrestricted-radius policy would assume more about how the
controller separates small punches than these readings establish.
