"""Pin PR #1724's six before/after evidence images, without pixel goldens."""
import pathlib
import re
import struct
import sys

root = pathlib.Path(__file__).resolve().parent
readme = (root / "README.md").read_text()
body = pathlib.Path(sys.argv[1]).read_text() if len(sys.argv) > 1 else None
for control in ("drill", "bore", "thread"):
    for state in ("before", "after"):
        name = f"{control}-{state}.png"
        for label, text in (("README", readme), ("PR body", body)):
            if text is not None:
                assert re.search(r"!\[[^\]]*\]\([^\s)]*" + re.escape(name) + r"\)", text), f"{label} lacks image {name}"
        data = (root / name).read_bytes()
        assert data[:8] == b"\x89PNG\r\n\x1a\n", f"invalid PNG: {name}"
        width, height = struct.unpack(">II", data[16:24])
        assert width >= 620 and height >= 600, f"unexpected dimensions: {name}"
print("PASS: all six Drill, Bore and Thread before/after screenshots are embedded")
