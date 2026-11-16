## #2173 - Locate the stock on an existing vise

**Decided.** The locate takes a held body: the part (as before) or the
stock of the fixture's setup (`LocateBody::Stock`). With the stock held,
the first pick of each mate is a stock face or point. The solve moves the
part, and the stock follows it by the stock-follow law, so the part keeps
its place in the blank and the stock lands on the mates. The Fixtures page
and the Model page hold the stock by default when the fixture's setup has
one ("Place by: Stock faces / Part faces"). The API and the MCP
`locate_part` keep the part as the default (`"body": "stock"` opts in).
`list_features` lists the faces of a stock, so MCP can name the mates.

**Why.** A vise holds the blank, not the finished part. The fixture
collision checks and the carried-workholding check read the stock where
it is, so a stock that cannot be seated on the vise leaves those checks
judging a guessed place. Moving the part with the stock keeps the
part-in-blank relation that the toolpaths use.

**Refusals.** A stock locate that names no part, a setup with no stock,
and a mate on the wrong body are refused by name. A square stock re-fits
round a turned part with the allowance of each setup side; when that
re-fit would leave the stock faces off the mates, the locate is refused
("would not land on the mates") instead of posting a stock off its seat.

**Considered and dropped.**
- Move the stock alone: it detaches the part from its blank, and every
  stock-relative height and the rest-material model would then be wrong.
- Infer the body from the first pick: a hidden choice, and a part inside
  the stock is never under the cursor first. The picks pass over the parts
  while the stock is held, so a stock face round a part can be picked.
- Default the API to the stock: it would change the result of every
  existing script and project test that locates a part.

**Cost to change.** Small: `LocateBody` is one enum on `LocateSpec`; a
turned stock locate could later re-fit the stock in the turned frame
instead of the refusal.
