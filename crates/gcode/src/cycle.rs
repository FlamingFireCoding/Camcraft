//! Canned cycle validation and expansion into explicit `cl::Move` chains and `cl::Cycle` records.
//! Provenance: cycle motion geometry from RS-274/NGC (NISTIR 6556 §3.5.16) and CamCraft's canonical drilling chains (`crates/motion/src/drilling/cycle.rs`).

use camcraft_cl::cl::{lattice, Cycle, CycleKind, MoveKind, PathBuilder};
use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::geometry::Point3;

use crate::types::{CycleReturnMode, CycleType, DistanceMode, GcodeError};

/// Sticky parameters for an active canned-cycle modal group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveCycle {
    pub kind: CycleType,
    pub initial_z: f64,
    pub r_z: f64,
    pub bottom_z: f64,
    pub peck_mm: f64,
    pub dwell_s: f64,
}

/// Parameters parsed from one canned-cycle block (in mm and seconds).
#[derive(Clone, Copy, Debug)]
pub struct CycleBlockInput {
    pub line: usize,
    pub kind: CycleType,
    pub is_new_cycle: bool,
    pub distance: DistanceMode,
    pub return_mode: CycleReturnMode,
    pub x_mm: Option<f64>,
    pub y_mm: Option<f64>,
    pub z_mm: Option<f64>,
    pub r_mm: Option<f64>,
    pub q_mm: Option<f64>,
    pub p_sec: Option<f64>,
    pub repeat: u32,
    pub feed_mm_min: f64,
    pub chip_break_retract_mm: f64,
}

const MAX_PECKS: f64 = 4096.0;

/// Validate and expand one canned-cycle block (with optional `L` repeats) onto `builder`.
pub fn execute_cycle_block(
    input: CycleBlockInput,
    active: &mut Option<ActiveCycle>,
    cur: &mut Point3,
    builder: &mut PathBuilder,
    move_lines: &mut Vec<usize>,
) -> Result<(), GcodeError> {
    let line = input.line;
    let same_active = active.filter(|a| a.kind == input.kind && !input.is_new_cycle);
    let initial_z = same_active.map_or(cur.z, |a| a.initial_z);

    let r_z = match input.r_mm {
        Some(r) => match input.distance {
            DistanceMode::Absolute => r,
            DistanceMode::Incremental => initial_z + r,
        },
        None => same_active.map(|a| a.r_z).ok_or_else(|| GcodeError::InvalidCycle {
            line,
            message: format!("canned cycle {:?} requires R retract plane", input.kind),
        })?,
    };

    let bottom_z = match input.z_mm {
        Some(z) => match input.distance {
            DistanceMode::Absolute => z,
            DistanceMode::Incremental => r_z + z,
        },
        None => same_active.map(|a| a.bottom_z).ok_or_else(|| GcodeError::InvalidCycle {
            line,
            message: format!("canned cycle {:?} requires Z depth", input.kind),
        })?,
    };

    if !r_z.is_finite() || !bottom_z.is_finite() || bottom_z >= r_z {
        return Err(GcodeError::InvalidCycle {
            line,
            message: format!("bottom Z ({bottom_z:.4}) must be below R plane ({r_z:.4})"),
        });
    }

    let peck_mm = resolve_peck(input, same_active, r_z, bottom_z)?;
    let dwell_s = resolve_dwell(input, same_active)?;

    *active = Some(ActiveCycle {
        kind: input.kind,
        initial_z,
        r_z,
        bottom_z,
        peck_mm,
        dwell_s,
    });

    if input.repeat == 0 {
        return Ok(());
    }

    for rep in 0..input.repeat {
        let (hx, hy) = match input.distance {
            DistanceMode::Absolute => (
                input.x_mm.unwrap_or(cur.x),
                input.y_mm.unwrap_or(cur.y),
            ),
            DistanceMode::Incremental => (
                cur.x + input.x_mm.unwrap_or(0.0),
                cur.y + input.y_mm.unwrap_or(0.0),
            ),
        };
        let _ = rep;
        expand_single_hole(
            input.line,
            input.kind,
            input.return_mode,
            initial_z,
            hx,
            hy,
            r_z,
            bottom_z,
            peck_mm,
            dwell_s,
            input.feed_mm_min,
            input.chip_break_retract_mm,
            cur,
            builder,
            move_lines,
        );
    }

    Ok(())
}

fn resolve_peck(
    input: CycleBlockInput,
    same_active: Option<ActiveCycle>,
    r_z: f64,
    bottom_z: f64,
) -> Result<f64, GcodeError> {
    let needs_peck = matches!(input.kind, CycleType::ChipBreakPeck | CycleType::DeepPeck);
    if !needs_peck {
        if input.q_mm.is_some() {
            return Err(GcodeError::InvalidCycle {
                line: input.line,
                message: format!("Q peck depth is not valid for {:?}", input.kind),
            });
        }
        return Ok(0.0);
    }

    let q = match input.q_mm {
        Some(v) => v,
        None => same_active.map(|a| a.peck_mm).ok_or_else(|| GcodeError::InvalidCycle {
            line: input.line,
            message: format!("peck cycle {:?} requires positive Q increment", input.kind),
        })?,
    };
    if !q.is_finite() || q <= 0.0 {
        return Err(GcodeError::InvalidCycle {
            line: input.line,
            message: format!("peck increment Q ({q}) must be positive"),
        });
    }
    if (r_z - bottom_z) / q > MAX_PECKS {
        return Err(GcodeError::InvalidCycle {
            line: input.line,
            message: "peck increment Q is too small (exceeds 4096 pecks per hole)".to_string(),
        });
    }
    Ok(q)
}

fn resolve_dwell(
    input: CycleBlockInput,
    same_active: Option<ActiveCycle>,
) -> Result<f64, GcodeError> {
    let allows_dwell = matches!(
        input.kind,
        CycleType::DwellDrill | CycleType::DeepPeck | CycleType::Bore | CycleType::BackBore
    );
    if !allows_dwell {
        if input.p_sec.is_some() {
            return Err(GcodeError::InvalidCycle {
                line: input.line,
                message: format!("P dwell is not valid for {:?}", input.kind),
            });
        }
        return Ok(0.0);
    }

    let p = match input.p_sec {
        Some(v) => v,
        None if input.kind == CycleType::DwellDrill => {
            same_active.map(|a| a.dwell_s).ok_or_else(|| GcodeError::InvalidCycle {
                line: input.line,
                message: "G82 dwell cycle requires P dwell word".to_string(),
            })?
        }
        None => same_active.map_or(0.0, |a| a.dwell_s),
    };
    if !p.is_finite() || p < 0.0 {
        return Err(GcodeError::InvalidCycle {
            line: input.line,
            message: format!("dwell P ({p}) must be non-negative"),
        });
    }
    Ok(p)
}

#[allow(clippy::too_many_arguments)]
fn expand_single_hole(
    line: usize,
    kind: CycleType,
    return_mode: CycleReturnMode,
    initial_z: f64,
    hx: f64,
    hy: f64,
    r_z: f64,
    bottom_z: f64,
    peck_mm: f64,
    dwell_s: f64,
    feed: f64,
    chip_break_retract_mm: f64,
    cur: &mut Point3,
    builder: &mut PathBuilder,
    move_lines: &mut Vec<usize>,
) {
    let push_tracked = |builder: &mut PathBuilder,
                        move_lines: &mut Vec<usize>,
                        k: MoveKind,
                        to: Point3,
                        f: Option<f64>| {
        let before = builder.moves_len();
        builder.push(k, to, f);
        let after = builder.moves_len();
        for _ in before..after {
            move_lines.push(line);
        }
    };

    // Preliminary positioning: lift to R first if currently below R, then XY, then R.
    if cur.z < r_z - 1e-9 {
        let p = Point3::new(cur.x, cur.y, r_z);
        push_tracked(builder, move_lines, MoveKind::Rapid, p, None);
        *cur = p;
    }
    let over_hole = Point3::new(hx, hy, cur.z);
    push_tracked(builder, move_lines, MoveKind::Rapid, over_hole, None);
    *cur = over_hole;

    let at_r = Point3::new(hx, hy, r_z);
    push_tracked(builder, move_lines, MoveKind::Rapid, at_r, None);
    *cur = at_r;

    let retract_z = match return_mode {
        CycleReturnMode::InitialZ => initial_z.max(r_z),
        CycleReturnMode::RPlane => r_z,
    };
    let first_move = builder.moves_len();

    match kind {
        CycleType::Drill | CycleType::DwellDrill | CycleType::Bore => {
            push_tracked(
                builder,
                move_lines,
                MoveKind::Plunge,
                Point3::new(hx, hy, bottom_z),
                Some(feed),
            );
            let out = Point3::new(hx, hy, retract_z);
            push_tracked(builder, move_lines, MoveKind::Rapid, out, None);
            *cur = out;
        }
        CycleType::ChipBreakPeck => {
            let mut t = (r_z - peck_mm).max(bottom_z);
            loop {
                push_tracked(
                    builder,
                    move_lines,
                    MoveKind::Plunge,
                    Point3::new(hx, hy, t),
                    Some(feed),
                );
                if t <= bottom_z + 1e-9 {
                    break;
                }
                let up = (t + chip_break_retract_mm).min(r_z);
                push_tracked(
                    builder,
                    move_lines,
                    MoveKind::Rapid,
                    Point3::new(hx, hy, up),
                    None,
                );
                t = (t - peck_mm).max(bottom_z);
            }
            let out = Point3::new(hx, hy, retract_z);
            push_tracked(builder, move_lines, MoveKind::Rapid, out, None);
            *cur = out;
        }
        CycleType::DeepPeck => {
            let mut t = (r_z - peck_mm).max(bottom_z);
            loop {
                push_tracked(
                    builder,
                    move_lines,
                    MoveKind::Plunge,
                    Point3::new(hx, hy, t),
                    Some(feed),
                );
                if t <= bottom_z + 1e-9 {
                    break;
                }
                push_tracked(
                    builder,
                    move_lines,
                    MoveKind::Rapid,
                    Point3::new(hx, hy, r_z),
                    None,
                );
                t = (t - peck_mm).max(bottom_z);
            }
            let out = Point3::new(hx, hy, retract_z);
            push_tracked(builder, move_lines, MoveKind::Rapid, out, None);
            *cur = out;
        }
        CycleType::Tap => {
            push_tracked(
                builder,
                move_lines,
                MoveKind::Feed,
                Point3::new(hx, hy, bottom_z),
                Some(feed),
            );
            push_tracked(
                builder,
                move_lines,
                MoveKind::Feed,
                Point3::new(hx, hy, r_z),
                Some(feed),
            );
            if retract_z > r_z + 1e-9 {
                let out = Point3::new(hx, hy, retract_z);
                push_tracked(builder, move_lines, MoveKind::Rapid, out, None);
                *cur = out;
            } else {
                *cur = at_r;
            }
        }
        CycleType::Ream | CycleType::BackBore => {
            push_tracked(
                builder,
                move_lines,
                MoveKind::Plunge,
                Point3::new(hx, hy, bottom_z),
                Some(feed),
            );
            push_tracked(
                builder,
                move_lines,
                MoveKind::Feed,
                Point3::new(hx, hy, r_z),
                Some(feed),
            );
            if retract_z > r_z + 1e-9 {
                let out = Point3::new(hx, hy, retract_z);
                push_tracked(builder, move_lines, MoveKind::Rapid, out, None);
                *cur = out;
            } else {
                *cur = at_r;
            }
        }
    }

    let cl_kind = match kind {
        CycleType::ChipBreakPeck => CycleKind::Peck,
        CycleType::Drill | CycleType::DwellDrill => CycleKind::Drill,
        CycleType::DeepPeck => CycleKind::Deep,
        CycleType::Tap => CycleKind::Tap,
        CycleType::Ream => CycleKind::Ream,
        CycleType::Bore => CycleKind::Bore,
        CycleType::BackBore => CycleKind::BackBore,
    };
    builder.push_cycle(Cycle {
        kind: cl_kind,
        xy: Point2::new(lattice(hx), lattice(hy)),
        r_z: lattice(r_z),
        bottom_z: lattice(bottom_z),
        entry_z: lattice(r_z),
        peck: lattice(peck_mm),
        dwell: dwell_s,
        first_move,
    });
}
