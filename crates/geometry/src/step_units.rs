//! STEP length units (#306): how many millimetres one of a file's
//! length units is, so a part saved in inches or metres imports at its
//! true size instead of having its numbers read as millimetres.
//!
//! Provenance: the unit entities of ISO 10303-41 as Part 21 files write
//! them - `GLOBAL_UNIT_ASSIGNED_CONTEXT`, `LENGTH_UNIT`, `SI_UNIT`
//! (prefix, name), `CONVERSION_BASED_UNIT` (name, measure) and
//! `LENGTH_MEASURE_WITH_UNIT` (value, unit). Public standard; nothing
//! from any CAM vendor. Read from solidcore's Part 21 exchange since
//! #2789 (it was ruststep's); moved from the app with it.

use camcraft_kernel::p21::{self, Exchange, Param};

/// How deep a conversion-based unit may chain to other units before it
/// is refused: a real file goes INCH -> millimetre in one step.
const MAX_CHAIN: usize = 4;
/// Two contexts whose units differ by less than this (relative) state
/// the same unit (a file's `25.4` and `25.400000000000002`).
const SAME_UNIT: f64 = 1e-9;
/// The most typed wrappers (`LENGTH_MEASURE(..)`) [`real`] looks through.
const MAX_TYPED: usize = 4;

/// [`mm_per_unit`] of a whole Part 21 text. `Err` also when the text does
/// not parse (the parser's message).
pub fn length_unit_mm(text: &str) -> Result<Option<f64>, String> {
    let ex = p21::parse(text).map_err(|e| e.to_string())?;
    mm_per_unit(&ex)
}

/// Millimetres in one length unit of the file, read from every
/// representation context's `GLOBAL_UNIT_ASSIGNED_CONTEXT`.
///
/// `Ok(None)`: no context states a length unit; the caller keeps
/// millimetres, as every import did before. `Err`: a stated unit this
/// reader can't convert, or contexts that disagree. A refusal names
/// it, because guessing a part's scale is the silent wrong answer.
pub(crate) fn mm_per_unit(ex: &Exchange) -> Result<Option<f64>, String> {
    let mut found: Option<f64> = None;
    for e in ex.entities.values() {
        let Some(a) = e.record("GLOBAL_UNIT_ASSIGNED_CONTEXT") else {
            continue;
        };
        let Some(units) = a.first().and_then(Param::as_list) else {
            return Err("GLOBAL_UNIT_ASSIGNED_CONTEXT has no unit list".into());
        };
        for id in units.iter().filter_map(Param::as_ref_id) {
            if !ex.get(id).is_some_and(|u| u.has("LENGTH_UNIT")) {
                continue; // the angle and solid-angle units ride along
            }
            let k = length_mm(ex, id, 0)?;
            match found {
                Some(prev) if ((prev - k) / prev).abs() > SAME_UNIT => {
                    return Err(format!(
                        "the file states two length units ({prev} mm and {k} mm per unit)"
                    ));
                }
                _ => found = Some(k),
            }
        }
    }
    Ok(found)
}

/// Millimetres in the length unit entity `#id`.
fn length_mm(ex: &Exchange, id: u64, depth: usize) -> Result<f64, String> {
    if depth > MAX_CHAIN {
        return Err(format!(
            "length unit #{id} chains through more than {MAX_CHAIN} units"
        ));
    }
    let unit = ex
        .get(id)
        .ok_or_else(|| format!("length unit #{id} is not in the file"))?;
    if let Some(si) = unit.record("SI_UNIT") {
        return si_mm(id, si);
    }
    if let Some(conv) = unit.record("CONVERSION_BASED_UNIT") {
        let Some(measure) = conv.get(1).and_then(Param::as_ref_id) else {
            return Err(format!("conversion-based unit #{id} names no measure"));
        };
        let m = ex
            .get(measure)
            .and_then(|e| e.record("LENGTH_MEASURE_WITH_UNIT"))
            .ok_or_else(|| {
                format!("conversion-based unit #{id}: #{measure} is no length measure")
            })?;
        let (value, base) = match m {
            [v, Param::Ref(base), ..] => (real(v), *base),
            _ => (None, 0),
        };
        let Some(value) = value.filter(|v| v.is_finite() && *v > 0.0) else {
            return Err(format!(
                "length measure #{measure} states no positive value"
            ));
        };
        return Ok(value * length_mm(ex, base, depth + 1)?);
    }
    Err(format!(
        "length unit #{id} is neither an SI unit nor a conversion-based one"
    ))
}

/// `SI_UNIT(prefix, name)` for a length: the metre, with its prefix.
fn si_mm(id: u64, a: &[Param]) -> Result<f64, String> {
    let word = |p: Option<&Param>| p.and_then(Param::as_enum).map(|e| e.trim_matches('.').to_ascii_uppercase());
    if word(a.get(1)).as_deref() != Some("METRE") {
        return Err(format!("length unit #{id} is an SI unit but not the metre"));
    }
    // Millimetres per prefixed metre; no prefix (`$`) is the metre itself.
    match word(a.first()).as_deref() {
        None => Ok(1000.0),
        Some("KILO") => Ok(1.0e6),
        Some("DECI") => Ok(100.0),
        Some("CENTI") => Ok(10.0),
        Some("MILLI") => Ok(1.0),
        Some("MICRO") => Ok(1.0e-3),
        Some("NANO") => Ok(1.0e-6),
        Some(p) => Err(format!("length unit #{id}: SI prefix {p} is not supported")),
    }
}

/// A measure value: a bare real, or one typed as `LENGTH_MEASURE(..)`.
/// A real that is not finite is handed out as read, so the caller names
/// it; [`Param::as_f64`] would hide it as "not a number".
pub(crate) fn real(p: &Param) -> Option<f64> {
    let mut p = p;
    for _ in 0..=MAX_TYPED {
        match p {
            Param::Real(x) => return Some(*x),
            Param::Int(i) => return Some(*i as f64),
            Param::Typed(_, v) if v.len() == 1 => p = v.first()?,
            _ => return None,
        }
    }
    None
}
