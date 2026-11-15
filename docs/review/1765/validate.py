"""Pin the two actual PNG captures and their Markdown image references."""
from pathlib import Path
import struct
import unittest
import zlib

ROOT = Path(__file__).resolve().parent


def png(path):
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "PNG signature"
    offset = 8
    kinds = []
    pixels = b""
    while offset < len(data):
        length = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4:offset + 8]
        payload = data[offset + 8:offset + 8 + length]
        crc = struct.unpack_from(">I", data, offset + 8 + length)[0]
        assert zlib.crc32(kind + payload) == crc, "PNG chunk CRC"
        kinds.append(kind)
        if kind == b"IHDR":
            assert struct.unpack_from(">II", payload) == (620, 1400)
        if kind == b"IDAT":
            pixels += payload
        offset += length + 12
    assert kinds[0] == b"IHDR" and kinds[-1] == b"IEND"
    return zlib.decompress(pixels)


class PassesReviewEvidence(unittest.TestCase):
    def test_before_after_pngs_and_image_references(self):
        markdown = (ROOT / "README.md").read_text()
        captures = []
        for name in ("passes-before.png", "passes-after.png"):
            self.assertIn(f"]({name})", markdown)
            captures.append(png(ROOT / name))
        self.assertNotEqual(*captures, "before and after must show different pages")


if __name__ == "__main__":
    unittest.main()
