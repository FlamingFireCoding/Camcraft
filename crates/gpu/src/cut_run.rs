//! The fallback rule of the stock removal kernel (#2627, GPU stock sim
//! S2): the GPU answers only when its device, its kernel and a spot
//! check against the CPU law all agree; otherwise the CPU law answers
//! and the reason is logged one time. A timed-out batch or a failed
//! spot check turns the GPU path off for the rest of the process.
//! Provenance: our own rule (decision 2627).

use std::sync::OnceLock;

use crate::adapter::HeadlessGpu;
use crate::cut::{cell_cut, cut_cpu, slot, CutInput};
use crate::cut_gpu::CutKernel;
use crate::shared::shared_device;
use crate::switch::log_fallback_once;

/// Texels that the spot check compares, at most, for each batch.
pub const SPOT_TEXELS: usize = 64;

/// The spot check's bound, as a fraction of the winning item's claim
/// margin: the GPU value and the CPU law may differ by at most this
/// much of the margin, so the GPU cut stays under the f64 cut.
pub const SPOT_MARGIN_FRACTION: f32 = 0.5;

/// Which law answered a batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CutPath {
    /// The kernel ran on this adapter (`"<name> via <backend>"`).
    Gpu { adapter: String },
    /// The CPU law ran; the reason the GPU did not.
    Cpu { reason: String },
}

/// The cut layer of one batch and the law that made it.
#[derive(Clone, Debug, PartialEq)]
pub struct CutOutput {
    pub values: Vec<f32>,
    pub path: CutPath,
}

static KERNEL: OnceLock<Result<CutKernel, String>> = OnceLock::new();
static OFF: OnceLock<String> = OnceLock::new();

/// Cut `input` on the process's shared device, or by the CPU law with
/// the reason. `Err` only for an input that neither law can cut (it
/// names the bad value).
pub fn cut_or_cpu(input: &CutInput) -> Result<CutOutput, String> {
    input.check()?;
    let gpu = shared_device().and_then(|gpu| {
        let kernel = KERNEL.get_or_init(|| CutKernel::new(&gpu.device));
        kernel.as_ref().map(|k| (gpu, k)).map_err(Clone::clone)
    });
    cut_or_cpu_on(gpu, input)
}

/// [`cut_or_cpu`] on a given device and kernel (or on the reason that
/// there is none): the GPU result when it passes the spot check, else
/// the CPU law.
pub fn cut_or_cpu_on(gpu: Result<(&HeadlessGpu, &CutKernel), String>, input: &CutInput) -> Result<CutOutput, String> {
    input.check()?;
    let run = || -> Result<CutOutput, String> {
        if let Some(why) = OFF.get() {
            return Err(why.clone());
        }
        let (gpu, kernel) = gpu?;
        let compute = gpu.compute.as_ref().map_err(Clone::clone)?;
        let values = kernel.run(&gpu.device, &gpu.queue, compute, input).inspect_err(|e| {
            if e.contains("did not answer") {
                let _ = OFF.set(format!("{e}; the GPU stock cut is off for this process"));
            }
        })?;
        spot_check(input, &values).inspect_err(|e| {
            let _ = OFF.set(format!("{e}; the GPU stock cut is off for this process"));
        })?;
        Ok(CutOutput { values, path: CutPath::Gpu { adapter: gpu.name.clone() } })
    };
    match run() {
        Ok(out) => Ok(out),
        Err(reason) => {
            log_fallback_once(&reason);
            Ok(CutOutput { values: cut_cpu(input)?, path: CutPath::Cpu { reason } })
        }
    }
}

/// Compare the GPU layer with the CPU law on up to [`SPOT_TEXELS`]
/// texels: the center cell of evenly spaced items, each evaluated over
/// every item that covers it. Values below `-cap` never show and
/// compare clamped there.
pub fn spot_check(input: &CutInput, gpu: &[f32]) -> Result<(), String> {
    let n = input.check()?;
    if gpu.len() != n {
        return Err(format!("stock cut kernel: the GPU gave {} texels, not {n}", gpu.len()));
    }
    let g = &input.grid;
    let floor = -4.0 * g.cap;
    let step = input.items.len().div_ceil(SPOT_TEXELS).max(1);
    for probe in input.items.iter().step_by(step) {
        let local: [u32; 3] = std::array::from_fn(|a| (probe.lo[a] + probe.hi[a]) / 2);
        let Some(at) = slot(probe, local, g.dims) else {
            return Err("stock cut kernel: a checked item left the lattice".into());
        };
        let (mut want, mut margin) = (floor, 0.0f32);
        for it in &input.items {
            let l: [i64; 3] = std::array::from_fn(|a| i64::from(probe.base[a]) + i64::from(local[a]) - i64::from(it.base[a]));
            let inside = (0..3).all(|a| i64::from(it.lo[a]) <= l[a] && l[a] <= i64::from(it.hi[a]));
            if !inside {
                continue;
            }
            let l = l.map(|v| u32::try_from(v).unwrap_or(0));
            if let Some(v) = cell_cut(it, l, g)
                && v > want
            {
                (want, margin) = (v, it.margin);
            }
        }
        let got = gpu.get(at).copied().unwrap_or(f32::NAN);
        let (w, h) = (want.max(-g.cap), got.max(-g.cap));
        let bound = (margin * SPOT_MARGIN_FRACTION).max(f32::EPSILON * g.cap);
        if !got.is_finite() || (h - w).abs() > bound {
            return Err(format!(
                "stock cut kernel: the GPU texel {at} reads {got}, the CPU law {want} (bound {bound}); the GPU result is not used"
            ));
        }
    }
    Ok(())
}
