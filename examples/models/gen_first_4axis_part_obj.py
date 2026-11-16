#!/usr/bin/env python3
"""Regenerate first-4axis-part.obj, the model of the "Your first 4-axis part" guide.

The part is a clean-room CamCraft fixture: the 1.5 in of a 3/4 in round bar
that sticks out of a 4th-axis chuck. The bar axis is the X axis, the free
end is at X0, and the axis passes through Y0 Z0 (the A centreline).
The finished part has:

- four flats, 0.015 in deep, from X-0.875 to X-0.125, one on each of the
  +Z, -Y, -Z and +Y sides (0.720 in across each pair);
- one cross slot on the +Z flat, 0.125 in wide about X-0.5, with its
  floor 0.030 in below the bar's skin (Z0.345).

An OBJ file has no unit and CamCraft reads its numbers as millimetres, so
the script writes millimetres.

Method: each length of the bar is a prism of one cross-section, and each
cross-section is a radius function r(angle) sampled at the same angles.
The sections therefore share their vertices at each station, the walls
between two sections are plain rings, and the mesh is watertight. The
script checks that before it writes the file.

Usage: python3 gen_first_4axis_part_obj.py [output.obj]
"""

import math
import sys

IN = 25.4
R = 0.375           # bar radius, in
FLAT = 0.360        # centreline to each flat, in
GROOVE = 0.345      # centreline to the cross-slot floor, in
STATIONS = [-1.5, -0.875, -0.5625, -0.4375, -0.125, 0.0]
ZONES = ["round", "flats", "groove", "flats", "round"]
UNIFORM = 96


def radius(zone, t):
    """The section's radius at angle t (y = r cos t, z = r sin t)."""
    r = R
    if zone in ("flats", "groove"):
        c, s = abs(math.cos(t)), abs(math.sin(t))
        if c > 1e-12:
            r = min(r, FLAT / c)
        if s > 1e-12:
            r = min(r, FLAT / s)
    if zone == "groove" and math.sin(t) > 1e-12:
        r = min(r, GROOVE / math.sin(t))
    return r


def angles():
    """Uniform samples plus the exact corners of the flats and the slot."""
    out = {round(math.tau * k / UNIFORM, 12) for k in range(UNIFORM)}
    corner = math.acos(FLAT / R)
    for quarter in range(4):
        base = quarter * math.pi / 2
        for t in (base - corner, base + corner):
            out.add(round(t % math.tau, 12))
    g = math.asin(GROOVE / R)
    out.update({round(g, 12), round(math.pi - g, 12)})
    return sorted(out)


def build():
    ts = angles()
    n = len(ts)
    verts, index, tris = [], {}, []

    def vid(x, r, t):
        p = (round(x * IN, 6), round(r * math.cos(t) * IN, 6), round(r * math.sin(t) * IN, 6))
        if p not in index:
            index[p] = len(verts)
            verts.append(p)
        return index[p]

    def tri(a, b, c):
        if a != b and b != c and a != c:
            tris.append((a, b, c))

    for k, zone in enumerate(ZONES):
        x0, x1 = STATIONS[k], STATIONS[k + 1]
        for j in range(n):
            t0, t1 = ts[j], ts[(j + 1) % n]
            a0, a1 = vid(x0, radius(zone, t0), t0), vid(x0, radius(zone, t1), t1)
            b0, b1 = vid(x1, radius(zone, t0), t0), vid(x1, radius(zone, t1), t1)
            tri(a0, b1, b0)
            tri(a0, a1, b1)
    # The step between two sections at a station: a ring in the plane of X.
    for k in range(1, len(ZONES)):
        x, left, right = STATIONS[k], ZONES[k - 1], ZONES[k]
        for j in range(n):
            t0, t1 = ts[j], ts[(j + 1) % n]
            l0, l1 = vid(x, radius(left, t0), t0), vid(x, radius(left, t1), t1)
            r0, r1 = vid(x, radius(right, t0), t0), vid(x, radius(right, t1), t1)
            if radius(left, t0) + radius(left, t1) >= radius(right, t0) + radius(right, t1):
                tri(r0, l0, l1)   # the left section is larger: the ring faces +X
                tri(r0, l1, r1)
            else:
                tri(l0, r1, r0)   # the right section is larger: the ring faces -X
                tri(l0, l1, r1)
    back, front = vid(STATIONS[0], 0.0, 0.0), vid(STATIONS[-1], 0.0, 0.0)
    for j in range(n):
        t0, t1 = ts[j], ts[(j + 1) % n]
        tri(back, vid(STATIONS[0], R, t1), vid(STATIONS[0], R, t0))
        tri(front, vid(STATIONS[-1], R, t0), vid(STATIONS[-1], R, t1))
    return verts, tris


def check(verts, tris):
    """Closed, and wound outward: each edge once in each direction, volume > 0."""
    seen = {}
    for a, b, c in tris:
        for e in ((a, b), (b, c), (c, a)):
            seen[e] = seen.get(e, 0) + 1
    for (a, b), count in seen.items():
        if count != 1 or seen.get((b, a), 0) != 1:
            raise SystemExit(f"open or doubled edge {a}-{b}")
    volume = 0.0
    for a, b, c in tris:
        (ax, ay, az), (bx, by, bz), (cx, cy, cz) = verts[a], verts[b], verts[c]
        volume += ax * (by * cz - bz * cy) - ay * (bx * cz - bz * cx) + az * (bx * cy - by * cx)
    if volume <= 0.0:
        raise SystemExit("the mesh is wound inward")
    return volume / 6.0


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "first-4axis-part.obj"
    verts, tris = build()
    volume = check(verts, tris)
    with open(out, "w", newline="\n") as f:
        f.write("# CamCraft example: Your first 4-axis part (millimetres)\n")
        f.write("# Made by gen_first_4axis_part_obj.py - do not edit by hand\n")
        f.write("o first-4axis-part\n")
        for x, y, z in verts:
            f.write(f"v {x:.6f} {y:.6f} {z:.6f}\n")
        for a, b, c in tris:
            f.write(f"f {a + 1} {b + 1} {c + 1}\n")
    print(f"{out}: {len(verts)} vertices, {len(tris)} triangles, {volume:.3f} mm3")


if __name__ == "__main__":
    main()
