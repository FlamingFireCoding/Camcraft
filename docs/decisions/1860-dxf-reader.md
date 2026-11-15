## #1860 - Bounded ASCII DXF reader with explicit units

**What was decided.** Add an original reader in `camcraft-geometry::dxf`.
It accepts an in-memory byte slice and returns plain geometry values.
It adds no dependencies or application state.
Records borrow the input text. Polylines retain only their first and previous
vertices while the reader emits segments.
This follows Autodesk's published [DXF file structure](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm).

Keep coordinates in drawing units. Report `$INSUNITS` as `Known(Unit)`,
`MissingOrUnitless`, or `Unsupported(i32)`.
Provide scales for the 24 defined length-unit codes. A caller must choose the
units when they are missing, then check the range again after conversion.
The unit mapping comes from Autodesk's [HEADER reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm).
This keeps the API usable by the later DXF import paths without an implicit
millimeter or inch assumption.

Preserve lines, circles and arcs as analytic primitives. Convert polyline
bulges into signed arc sweeps, including the closing segment.
The conversion follows the group-42 definition in Autodesk's
[VERTEX reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm).
Keep the layer name on each primitive. Keep one common Z elevation for the
document. Accept extrusion directions only along positive Z.
Refuse unsupported geometry by name. Also refuse widths, thickness, fitted or
3D polylines, paper-space geometry, application-defined group 102, and
unsupported sections. Count skipped annotation entities by kind.

Default limits are 16 MiB of input, 100,000 entity records, and 1,000,000
polyline vertices. The entity count includes skipped entities, `VERTEX`, and
`SEQEND`. Layer definitions have the same count limit.
Check declared vertex counts before reading vertices. Never use an input
count as an allocation capacity.
Limit layer names to 255 bytes. The byte limit is a deliberate restriction of
this ASCII reader, even for UTF-8 names. It bounds the memory used when each
segment copies its layer name.
Bound the complete support circle of each arc against `geo2d::COORD_LIMIT`.
Refuse a bulge whose sweep cannot be represented safely.

**Why.** The reader must preserve geometry or return a named error.
Explicit units prevent a silent scale change. Exact arcs retain the source
curve for later machining work. Borrowed records and count limits bound
untrusted input before geometry allocation. Conservative circle bounds also
cover intermediate centers and radii.

**Considered and dropped.** An external DXF library adds a dependency that
the issue excludes. Guessing missing units can change the part size.
Polygonizing arcs loses the source curve. Silently discarding unsupported
geometry can remove a cut. General OCS transforms, block expansion, splines,
ellipses and GUI integration remain outside this core-reader issue.
Storing all group pairs or vertices adds allocation that the streaming reader
does not need.

**Cost to change later.** New entity kinds can extend the parser behind the
same document API. New public primitive or unit variants require callers to
update exhaustive matches. Supporting more sections or OCS directions needs
format-specific tests and refusal checks. A shared layer-name representation
could reduce memory and permit a larger byte limit, but changes the public
primitive type. Relaxing the full-circle bound needs tests for the actual arc
extent and all intermediate values.
