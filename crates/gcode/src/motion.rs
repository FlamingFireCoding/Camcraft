//! Motion execution (`G0`/`G1`/`G2`/`G3`, `G4`, `G28`/`G30`/`G53`, canned cycles).
//! Provenance: motion, reference return, and canned-cycle invocation semantics from RS-274/NGC (NISTIR 6556 §3.5) and ISO 6983.

use camcraft_cl::cl::MoveKind;
use camcraft_geometry::geometry::Point3;
use camcraft_geometry::geometry::units::from_display;

use crate::arc::resolve_arc;
use crate::block::{DecodedBlock, Group1G, LengthOp, NonModalG, PathControlG};
use crate::block_codes::check_unused_words;
use crate::cycle::{execute_cycle_block, CycleBlockInput};
use crate::interp::Interpreter;
use crate::types::{DistanceMode, DwellEvent, DwellUnit, GcodeError, MotionMode};

impl<'a> Interpreter<'a> {
    pub(crate) fn execute_motion(&mut self, b: &DecodedBlock) -> Result<(), GcodeError> {
        let line = b.line;
        let g64_consumed = b.path_control == Some(PathControlG::Continuous);
        let allow_h = b.length_op == Some(LengthOp::Apply);

        if b.non_modal == Some(NonModalG::Dwell) {
            return self.execute_dwell(b, allow_h);
        }

        if let Some(nm @ (NonModalG::HomeG28 | NonModalG::SecondaryG30)) = b.non_modal {
            check_unused_words(b, false, false, g64_consumed, g64_consumed, allow_h, false)?;
            if matches!(b.group1, Some(Group1G::ArcCw | Group1G::ArcCcw | Group1G::Cycle(_))) {
                return Err(GcodeError::InvalidModal {
                    line,
                    message: "G28/G30 cannot share a block with arcs or canned cycles".to_string(),
                });
            }
            if let Some(g1) = b.group1 {
                self.set_group1(g1);
            }
            return self.execute_reference_return(b, nm == NonModalG::HomeG28);
        }

        if b.g53 {
            check_unused_words(b, false, false, g64_consumed, g64_consumed, allow_h, false)?;
            if let Some(g1) = b.group1 {
                self.set_group1(g1);
            }
            return self.execute_g53(b);
        }

        if let Some(g1) = b.group1 {
            if g1 == Group1G::Cancel {
                check_unused_words(b, false, false, g64_consumed, g64_consumed, allow_h, false)?;
                if b.x.is_some() || b.y.is_some() || b.z.is_some() {
                    return Err(GcodeError::InvalidModal {
                        line,
                        message: "axis coordinates are invalid on a G80 cancel block".to_string(),
                    });
                }
                self.active_cycle = None;
                self.modal.motion = MotionMode::None;
                return Ok(());
            }
            self.set_group1(g1);
        }

        let has_coords = b.x.is_some()
            || b.y.is_some()
            || b.z.is_some()
            || b.i.is_some()
            || b.j.is_some()
            || b.k.is_some()
            || b.r.is_some()
            || (!g64_consumed && (b.p.is_some() || b.q.is_some()))
            || b.l.is_some();

        if !has_coords && b.group1.is_none() {
            return Ok(());
        }

        match self.modal.motion {
            MotionMode::None => {
                check_unused_words(b, false, false, g64_consumed, g64_consumed, allow_h, false)?;
                Err(GcodeError::InvalidModal {
                    line,
                    message: "axis motion or parameters with no active motion mode (G0/G1/G2/G3/cycle)"
                        .to_string(),
                })
            }
            MotionMode::Rapid => self.execute_linear(b, false, g64_consumed, allow_h),
            MotionMode::Linear => self.execute_linear(b, true, g64_consumed, allow_h),
            MotionMode::ArcCw | MotionMode::ArcCcw => {
                self.execute_arc(b, g64_consumed, allow_h)
            }
            MotionMode::Cycle(cycle_type) => {
                self.execute_cycle(b, cycle_type, allow_h)
            }
        }
    }

    fn execute_dwell(&mut self, b: &DecodedBlock, allow_h: bool) -> Result<(), GcodeError> {
        let line = b.line;
        check_unused_words(b, false, false, true, false, allow_h, false)?;
        if b.y.is_some() || b.z.is_some() {
            return Err(GcodeError::InvalidModal {
                line,
                message: "Y/Z words are invalid with G4 dwell".to_string(),
            });
        }
        if b.p.is_some() && b.x.is_some() {
            return Err(GcodeError::InvalidModal {
                line,
                message: "G4 dwell accepts P or X, not both".to_string(),
            });
        }
        let seconds = if let Some(p) = b.p {
            dwell_to_seconds(line, p, self.options.dialect.g4_dwell_unit)?
        } else if let Some(x) = b.x {
            dwell_to_seconds(line, x, DwellUnit::Seconds)?
        } else {
            return Err(GcodeError::Syntax {
                line,
                message: "G4 dwell requires a P or X duration".to_string(),
            });
        };
        self.dwells.push(DwellEvent {
            line,
            seconds,
            section: self.current.index,
            after_move: self.current.builder.moves_len().checked_sub(1),
        });
        Ok(())
    }

    fn execute_linear(
        &mut self,
        b: &DecodedBlock,
        is_feed: bool,
        g64_consumed: bool,
        allow_h: bool,
    ) -> Result<(), GcodeError> {
        let line = b.line;
        check_unused_words(b, false, false, g64_consumed, g64_consumed, allow_h, false)?;
        if b.x.is_none() && b.y.is_none() && b.z.is_none() {
            return Ok(());
        }
        let unit = self.require_unit(line)?;
        let (kind, feed) = if is_feed {
            (MoveKind::Feed, Some(self.require_feed(line, "G1")?))
        } else {
            (MoveKind::Rapid, None)
        };
        let to = self.target_point(b, unit);
        self.current.push_move(line, kind, to, feed);
        self.pos = to;
        Ok(())
    }

    fn execute_arc(
        &mut self,
        b: &DecodedBlock,
        g64_consumed: bool,
        allow_h: bool,
    ) -> Result<(), GcodeError> {
        let line = b.line;
        check_unused_words(b, true, true, g64_consumed, g64_consumed, allow_h, false)?;
        if b.group1.is_none()
            && b.x.is_none()
            && b.y.is_none()
            && b.z.is_none()
            && b.i.is_none()
            && b.j.is_none()
            && b.k.is_none()
            && b.r.is_none()
        {
            return Ok(());
        }
        let unit = self.require_unit(line)?;
        let ccw = self.modal.motion == MotionMode::ArcCcw;
        let feed = self.require_feed(line, if ccw { "G3" } else { "G2" })?;
        let to = self.target_point(b, unit);
        let ijk = [
            b.i.map(|v| from_display(v, unit)),
            b.j.map(|v| from_display(v, unit)),
            b.k.map(|v| from_display(v, unit)),
        ];
        let r_mm = b.r.map(|v| from_display(v, unit));
        let resolved = resolve_arc(
            line,
            self.pos,
            to,
            self.modal.plane,
            ccw,
            self.modal.arc_center,
            ijk,
            r_mm,
            self.options.dialect.arc_radius_tolerance_mm,
        )?;
        if let Some(mid) = resolved.split_mid {
            self.current.push_arc(
                line,
                mid,
                feed,
                resolved.center_words,
                ccw,
                self.modal.plane,
            );
        }
        self.current.push_arc(
            line,
            to,
            feed,
            resolved.center_words,
            ccw,
            self.modal.plane,
        );
        self.pos = to;
        Ok(())
    }

    fn execute_cycle(
        &mut self,
        b: &DecodedBlock,
        cycle_type: crate::types::CycleType,
        allow_h: bool,
    ) -> Result<(), GcodeError> {
        let line = b.line;
        check_unused_words(b, false, true, true, true, allow_h, true)?;
        let unit = self.require_unit(line)?;
        let feed = self.require_feed(line, "canned cycle")?;
        let repeat = b.l.unwrap_or(1);
        if repeat > 10_000 {
            return Err(GcodeError::InvalidCycle {
                line,
                message: format!("cycle repeat count L{repeat} exceeds 10000"),
            });
        }
        let p_sec = match b.p {
            Some(p) => Some(dwell_to_seconds(line, p, self.options.dialect.cycle_dwell_unit)?),
            None => None,
        };
        let is_new_cycle = matches!(b.group1, Some(Group1G::Cycle(_)));
        let input = CycleBlockInput {
            line,
            kind: cycle_type,
            is_new_cycle,
            distance: self.modal.distance,
            return_mode: self.modal.cycle_return,
            x_mm: b.x.map(|v| from_display(v, unit)),
            y_mm: b.y.map(|v| from_display(v, unit)),
            z_mm: b.z.map(|v| from_display(v, unit)),
            r_mm: b.r.map(|v| from_display(v, unit)),
            q_mm: b.q.map(|v| from_display(v, unit)),
            p_sec,
            repeat,
            feed_mm_min: feed,
            chip_break_retract_mm: self.options.dialect.chip_break_retract_mm,
        };
        execute_cycle_block(
            input,
            &mut self.active_cycle,
            &mut self.pos,
            &mut self.current.builder,
            &mut self.current.move_lines,
        )
    }

    pub(crate) fn set_group1(&mut self, g1: Group1G) {
        let next = match g1 {
            Group1G::Rapid => MotionMode::Rapid,
            Group1G::Linear => MotionMode::Linear,
            Group1G::ArcCw => MotionMode::ArcCw,
            Group1G::ArcCcw => MotionMode::ArcCcw,
            Group1G::Cancel => MotionMode::None,
            Group1G::Cycle(ct) => MotionMode::Cycle(ct),
        };
        if !matches!(next, MotionMode::Cycle(_)) || self.modal.motion != next {
            self.active_cycle = None;
        }
        self.modal.motion = next;
    }

    fn execute_reference_return(
        &mut self,
        b: &DecodedBlock,
        is_g28: bool,
    ) -> Result<(), GcodeError> {
        let line = b.line;
        let code = if is_g28 { "G28" } else { "G30" };
        let unit = self.require_unit(line)?;
        let ref_pt = if is_g28 {
            self.options.references.g28
        } else {
            self.options.references.g30
        }
        .ok_or_else(|| GcodeError::MissingReferencePosition {
            line,
            code: code.to_string(),
        })?;
        let wcs_org = self
            .options
            .references
            .wcs_origin(self.modal.work_offset)
            .unwrap_or(Point3::ZERO);
        let ref_wcs = Point3::new(
            ref_pt.x - wcs_org.x,
            ref_pt.y - wcs_org.y,
            ref_pt.z - wcs_org.z,
        );
        if b.x.is_some() || b.y.is_some() || b.z.is_some() {
            let inter = self.target_point(b, unit);
            self.current.push_move(line, MoveKind::Rapid, inter, None);
            self.pos = inter;
            let final_pt = Point3::new(
                if b.x.is_some() { ref_wcs.x } else { self.pos.x },
                if b.y.is_some() { ref_wcs.y } else { self.pos.y },
                if b.z.is_some() { ref_wcs.z } else { self.pos.z },
            );
            self.current.push_move(line, MoveKind::Rapid, final_pt, None);
            self.pos = final_pt;
        } else {
            self.current.push_move(line, MoveKind::Rapid, ref_wcs, None);
            self.pos = ref_wcs;
        }
        Ok(())
    }

    fn execute_g53(&mut self, b: &DecodedBlock) -> Result<(), GcodeError> {
        let line = b.line;
        let unit = self.require_unit(line)?;
        if self.modal.distance != DistanceMode::Absolute {
            return Err(GcodeError::InvalidModal {
                line,
                message: "G53 machine-coordinate move requires G90 absolute distance mode"
                    .to_string(),
            });
        }
        let wcs_org = self
            .options
            .references
            .wcs_origin(self.modal.work_offset)
            .ok_or_else(|| GcodeError::MissingReferencePosition {
                line,
                code: "G53".to_string(),
            })?;
        let (kind, feed) = match self.modal.motion {
            MotionMode::Rapid => (MoveKind::Rapid, None),
            MotionMode::Linear => (MoveKind::Feed, Some(self.require_feed(line, "G53 G1")?)),
            _ => {
                return Err(GcodeError::InvalidModal {
                    line,
                    message: "G53 is valid only with G0 or G1 motion mode".to_string(),
                });
            }
        };
        let to = Point3::new(
            b.x.map_or(self.pos.x, |v| from_display(v, unit) - wcs_org.x),
            b.y.map_or(self.pos.y, |v| from_display(v, unit) - wcs_org.y),
            b.z.map_or(self.pos.z, |v| from_display(v, unit) - wcs_org.z),
        );
        self.current.push_move(line, kind, to, feed);
        self.pos = to;
        Ok(())
    }
}

fn dwell_to_seconds(line: usize, val: f64, unit: DwellUnit) -> Result<f64, GcodeError> {
    if !val.is_finite() || val < 0.0 {
        return Err(GcodeError::InvalidModal {
            line,
            message: format!("dwell duration ({val}) must be non-negative"),
        });
    }
    Ok(match unit {
        DwellUnit::Seconds => val,
        DwellUnit::Milliseconds => val / 1000.0,
    })
}
