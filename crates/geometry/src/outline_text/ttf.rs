//! A read-only TrueType outline reader: the table directory, `head`, `maxp`, `hhea`, `hmtx`,
//! `OS/2`, `cmap` (formats 4 and 12), `loca` and `glyf` (simple and composite glyphs).
//!
//! Provenance: written from the public OpenType specification (Microsoft Typography) and the
//! Apple TrueType Reference Manual - table layouts only. No font library code was consulted.
//!
//! Contract. Each read is bounds-checked: a short or corrupt table is a [`FontError`], never
//! a panic and never an out-of-range index. A composite glyph is resolved to depth
//! [`MAX_DEPTH`], and the points of one glyph are capped at [`MAX_POINTS`].

use super::FontError;

/// The deepest chain of composite glyphs that is resolved.
pub const MAX_DEPTH: usize = 8;
/// The most points that one resolved glyph can have.
pub const MAX_POINTS: usize = 100_000;

/// One outline point in font units, and whether it lies on the curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlinePoint {
    pub x: f64,
    pub y: f64,
    pub on: bool,
}

/// The contours of one glyph, in font units.
pub type Contours = Vec<Vec<OutlinePoint>>;

pub struct Font<'a> {
    data: &'a [u8],
    glyf: (usize, usize),
    loca: (usize, usize),
    hmtx: (usize, usize),
    cmap: Cmap,
    long_loca: bool,
    num_glyphs: u16,
    num_h_metrics: u16,
    pub units_per_em: u16,
    os2_cap_height: Option<i16>,
}

#[derive(Clone, Copy)]
enum Cmap {
    Format4(usize),
    Format12(usize),
}

fn u16_at(d: &[u8], at: usize) -> Result<u16, FontError> {
    let b = d.get(at..at.checked_add(2).ok_or(FontError::Truncated)?).ok_or(FontError::Truncated)?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}

fn i16_at(d: &[u8], at: usize) -> Result<i16, FontError> {
    u16_at(d, at).map(|v| v as i16)
}

fn u32_at(d: &[u8], at: usize) -> Result<u32, FontError> {
    let b = d.get(at..at.checked_add(4).ok_or(FontError::Truncated)?).ok_or(FontError::Truncated)?;
    Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn u8_at(d: &[u8], at: usize) -> Result<u8, FontError> {
    d.get(at).copied().ok_or(FontError::Truncated)
}

fn f2dot14(d: &[u8], at: usize) -> Result<f64, FontError> {
    Ok(f64::from(i16_at(d, at)?) / 16384.0)
}

fn to_usize(v: u32) -> usize {
    usize::try_from(v).unwrap_or(usize::MAX)
}

/// The (offset, length) of each required table, checked against the file length.
fn table(data: &[u8], tag: &[u8; 4]) -> Result<Option<(usize, usize)>, FontError> {
    let count = usize::from(u16_at(data, 4)?);
    for i in 0..count {
        let rec = 12 + 16 * i;
        let name = data.get(rec..rec + 4).ok_or(FontError::Truncated)?;
        if name == tag {
            let offset = to_usize(u32_at(data, rec + 8)?);
            let length = to_usize(u32_at(data, rec + 12)?);
            let end = offset.checked_add(length).ok_or(FontError::Truncated)?;
            if end > data.len() {
                return Err(FontError::Truncated);
            }
            return Ok(Some((offset, length)));
        }
    }
    Ok(None)
}

fn required(data: &[u8], tag: &'static str) -> Result<(usize, usize), FontError> {
    let mut t = [0_u8; 4];
    t.copy_from_slice(tag.as_bytes());
    table(data, &t)?.ok_or(FontError::MissingTable(tag))
}

impl<'a> Font<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Font<'a>, FontError> {
        match data.get(0..4) {
            Some([0, 1, 0, 0]) | Some(b"true") => {}
            Some(b"OTTO") => return Err(FontError::CffOutlines),
            Some(b"ttcf") => return Err(FontError::Collection),
            _ => return Err(FontError::NotAFont),
        }
        let head = required(data, "head")?;
        let units_per_em = u16_at(data, head.0 + 18)?;
        if !(16..=16384).contains(&units_per_em) {
            return Err(FontError::Corrupt("the units per em are outside 16 to 16384"));
        }
        let long_loca = match i16_at(data, head.0 + 50)? {
            0 => false,
            1 => true,
            _ => return Err(FontError::Corrupt("the loca format is not 0 or 1")),
        };
        let num_glyphs = u16_at(data, required(data, "maxp")?.0 + 4)?;
        let num_h_metrics = u16_at(data, required(data, "hhea")?.0 + 34)?;
        if num_h_metrics == 0 {
            return Err(FontError::Corrupt("the font has no horizontal metrics"));
        }
        let glyf = table(data, b"glyf")?.ok_or(if table(data, b"CFF ")?.is_some() {
            FontError::CffOutlines
        } else {
            FontError::MissingTable("glyf")
        })?;
        let os2_cap_height = match table(data, b"OS/2")? {
            Some((at, len)) if u16_at(data, at)? >= 2 && len >= 90 => Some(i16_at(data, at + 88)?),
            _ => None,
        };
        Ok(Font {
            data,
            glyf,
            loca: required(data, "loca")?,
            hmtx: required(data, "hmtx")?,
            cmap: pick_cmap(data, required(data, "cmap")?.0)?,
            long_loca,
            num_glyphs,
            num_h_metrics,
            units_per_em,
            os2_cap_height,
        })
    }

    /// The glyph of `ch`, or `None` when the font maps it to no glyph.
    pub fn glyph_id(&self, ch: char) -> Result<Option<u16>, FontError> {
        let c = u32::from(ch);
        let d = self.data;
        let gid = match self.cmap {
            Cmap::Format12(at) => {
                let groups = to_usize(u32_at(d, at + 12)?);
                let mut found = 0_u32;
                for i in 0..groups.min(1 << 20) {
                    let g = at + 16 + 12 * i;
                    let (start, end) = (u32_at(d, g)?, u32_at(d, g + 4)?);
                    if (start..=end).contains(&c) {
                        found = u32_at(d, g + 8)?.saturating_add(c - start);
                        break;
                    }
                }
                found
            }
            Cmap::Format4(at) => {
                let Ok(c) = u16::try_from(c) else { return Ok(None) };
                let segs = usize::from(u16_at(d, at + 6)? / 2);
                let ends = at + 14;
                let starts = ends + 2 * segs + 2;
                let deltas = starts + 2 * segs;
                let ranges = deltas + 2 * segs;
                let mut found = 0_u32;
                for s in 0..segs {
                    if u16_at(d, ends + 2 * s)? < c {
                        continue;
                    }
                    let start = u16_at(d, starts + 2 * s)?;
                    if start > c {
                        break;
                    }
                    let delta = u16_at(d, deltas + 2 * s)?;
                    let range = u16_at(d, ranges + 2 * s)?;
                    found = if range == 0 {
                        u32::from(c.wrapping_add(delta))
                    } else {
                        let addr = ranges + 2 * s + usize::from(range) + 2 * usize::from(c - start);
                        match u16_at(d, addr)? {
                            0 => 0,
                            g => u32::from(g.wrapping_add(delta)),
                        }
                    };
                    break;
                }
                found
            }
        };
        Ok(u16::try_from(gid).ok().filter(|&g| g != 0 && g < self.num_glyphs))
    }

    /// The advance width of `gid`, in font units.
    pub fn advance(&self, gid: u16) -> Result<u16, FontError> {
        let index = usize::from(gid.min(self.num_h_metrics - 1));
        if 4 * (index + 1) > self.hmtx.1 {
            return Err(FontError::Truncated);
        }
        u16_at(self.data, self.hmtx.0 + 4 * index)
    }

    /// The cap height in font units: the `OS/2` value, else the top of `H`, else 70 % of the
    /// em.
    pub fn cap_height(&self) -> Result<f64, FontError> {
        if let Some(h) = self.os2_cap_height.filter(|&h| h > 0) {
            return Ok(f64::from(h));
        }
        if let Some(gid) = self.glyph_id('H')? {
            let top = self.outline(gid)?.iter().flatten().map(|p| p.y).fold(0.0_f64, f64::max);
            if top > 0.0 {
                return Ok(top);
            }
        }
        Ok(0.7 * f64::from(self.units_per_em))
    }

    fn glyph_range(&self, gid: u16) -> Result<(usize, usize), FontError> {
        let g = usize::from(gid);
        let (a, b) = if self.long_loca {
            (u32_at(self.data, self.loca.0 + 4 * g)?, u32_at(self.data, self.loca.0 + 4 * g + 4)?)
        } else {
            let a = u16_at(self.data, self.loca.0 + 2 * g)?;
            let b = u16_at(self.data, self.loca.0 + 2 * g + 2)?;
            (2 * u32::from(a), 2 * u32::from(b))
        };
        let (a, b) = (to_usize(a), to_usize(b));
        if a > b || b > self.glyf.1 {
            return Err(FontError::Corrupt("a glyph lies outside the glyf table"));
        }
        Ok((self.glyf.0 + a, self.glyf.0 + b))
    }

    /// The contours of `gid` in font units, composites resolved. A glyph with no outline (the
    /// space) has no contours.
    pub fn outline(&self, gid: u16) -> Result<Contours, FontError> {
        let mut out = Vec::new();
        self.append(gid, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0], 0, &mut out)?;
        Ok(out)
    }

    fn append(&self, gid: u16, m: [f64; 6], depth: usize, out: &mut Contours) -> Result<(), FontError> {
        if depth > MAX_DEPTH {
            return Err(FontError::Corrupt("composite glyphs nest too deep"));
        }
        if gid >= self.num_glyphs {
            return Err(FontError::Corrupt("a composite glyph names a glyph that does not exist"));
        }
        let (start, end) = self.glyph_range(gid)?;
        if start == end {
            return Ok(());
        }
        let d = self.data.get(start..end).ok_or(FontError::Truncated)?;
        let contours = i16_at(d, 0)?;
        if contours >= 0 {
            for contour in simple(d, usize::from(contours.unsigned_abs()))? {
                out.push(contour.into_iter().map(|p| transform(m, p)).collect());
            }
        } else {
            self.composite(d, m, depth, out)?;
        }
        if out.iter().map(Vec::len).sum::<usize>() > MAX_POINTS {
            return Err(FontError::Corrupt("a glyph has too many points"));
        }
        Ok(())
    }

    fn composite(&self, d: &[u8], m: [f64; 6], depth: usize, out: &mut Contours) -> Result<(), FontError> {
        let mut at = 10;
        loop {
            let flags = u16_at(d, at)?;
            let child = u16_at(d, at + 2)?;
            at += 4;
            if flags & 0x0002 == 0 {
                return Err(FontError::Corrupt("a composite glyph places a part by point matching"));
            }
            let (dx, dy) = if flags & 0x0001 != 0 {
                at += 4;
                (f64::from(i16_at(d, at - 4)?), f64::from(i16_at(d, at - 2)?))
            } else {
                at += 2;
                (f64::from(u8_at(d, at - 2)? as i8), f64::from(u8_at(d, at - 1)? as i8))
            };
            let (a, b, c, e) = if flags & 0x0008 != 0 {
                at += 2;
                let s = f2dot14(d, at - 2)?;
                (s, 0.0, 0.0, s)
            } else if flags & 0x0040 != 0 {
                at += 4;
                (f2dot14(d, at - 4)?, 0.0, 0.0, f2dot14(d, at - 2)?)
            } else if flags & 0x0080 != 0 {
                at += 8;
                (f2dot14(d, at - 8)?, f2dot14(d, at - 6)?, f2dot14(d, at - 4)?, f2dot14(d, at - 2)?)
            } else {
                (1.0, 0.0, 0.0, 1.0)
            };
            // The part's own matrix, then the parent's: x' = a x + c y + dx, y' = b x + e y + dy.
            let part = [a, b, c, e, dx, dy];
            let joined = [
                m[0] * part[0] + m[2] * part[1],
                m[1] * part[0] + m[3] * part[1],
                m[0] * part[2] + m[2] * part[3],
                m[1] * part[2] + m[3] * part[3],
                m[0] * part[4] + m[2] * part[5] + m[4],
                m[1] * part[4] + m[3] * part[5] + m[5],
            ];
            self.append(child, joined, depth + 1, out)?;
            if flags & 0x0020 == 0 {
                return Ok(());
            }
        }
    }
}

fn transform(m: [f64; 6], p: OutlinePoint) -> OutlinePoint {
    OutlinePoint { x: m[0] * p.x + m[2] * p.y + m[4], y: m[1] * p.x + m[3] * p.y + m[5], on: p.on }
}

/// The contours of a simple glyph: end points, flags (with repeats), then the x and y deltas.
fn simple(d: &[u8], contours: usize) -> Result<Contours, FontError> {
    let mut ends = Vec::with_capacity(contours);
    for i in 0..contours {
        ends.push(usize::from(u16_at(d, 10 + 2 * i)?));
    }
    let Some(&last) = ends.last() else { return Ok(Vec::new()) };
    if ends.windows(2).any(|w| w[0] >= w[1]) {
        return Err(FontError::Corrupt("the contour end points of a glyph do not ascend"));
    }
    let count = last + 1;
    if count > MAX_POINTS {
        return Err(FontError::Corrupt("a glyph has too many points"));
    }
    let instructions = 10 + 2 * contours;
    let mut at = instructions + 2 + usize::from(u16_at(d, instructions)?);
    let mut flags = Vec::with_capacity(count);
    while flags.len() < count {
        let f = u8_at(d, at)?;
        at += 1;
        let repeat = if f & 0x08 != 0 {
            at += 1;
            usize::from(u8_at(d, at - 1)?)
        } else {
            0
        };
        for _ in 0..=repeat {
            flags.push(f);
        }
    }
    flags.truncate(count);
    let mut coord = |short: u8, same: u8| -> Result<Vec<f64>, FontError> {
        let mut v = 0_i32;
        let mut out = Vec::with_capacity(count);
        for &f in &flags {
            if f & short != 0 {
                let b = i32::from(u8_at(d, at)?);
                at += 1;
                v += if f & same != 0 { b } else { -b };
            } else if f & same == 0 {
                v += i32::from(i16_at(d, at)?);
                at += 2;
            }
            out.push(f64::from(v));
        }
        Ok(out)
    };
    let xs = coord(0x02, 0x10)?;
    let ys = coord(0x04, 0x20)?;
    let mut out = Vec::with_capacity(contours);
    let mut first = 0;
    for end in ends {
        out.push((first..=end).map(|i| OutlinePoint { x: xs[i], y: ys[i], on: flags[i] & 0x01 != 0 }).collect());
        first = end + 1;
    }
    Ok(out)
}

/// The best Unicode subtable: format 12 (full Unicode) first, then format 4 (the BMP).
fn pick_cmap(data: &[u8], at: usize) -> Result<Cmap, FontError> {
    let count = usize::from(u16_at(data, at + 2)?);
    let mut best: Option<Cmap> = None;
    for i in 0..count {
        let rec = at + 4 + 8 * i;
        let (platform, encoding) = (u16_at(data, rec)?, u16_at(data, rec + 2)?);
        let unicode = platform == 0 || (platform == 3 && (encoding == 1 || encoding == 10));
        if !unicode {
            continue;
        }
        let sub = at.checked_add(to_usize(u32_at(data, rec + 4)?)).ok_or(FontError::Truncated)?;
        match u16_at(data, sub)? {
            12 => return Ok(Cmap::Format12(sub)),
            4 if best.is_none() => best = Some(Cmap::Format4(sub)),
            _ => {}
        }
    }
    best.ok_or(FontError::MissingTable("cmap (a Unicode subtable of format 4 or 12)"))
}
