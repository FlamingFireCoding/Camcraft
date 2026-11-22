//! Parity of the stock removal kernel (#2627) with the f32 cut twin of
//! camcraft-sim (#2626) and with the f64 field cut, read from the
//! fixtures that camcraft-sim records (`fixtures/cut_twin_*_2627.txt`,
//! written by `stock_cut_twin::the_gpu_kernel_fixtures_match_the_twin`).
//! The CPU law always runs; the GPU path runs when the machine has an
//! adapter with compute, and the test prints which path ran.
//! Gate map: covers camcraft_gpu::cut, camcraft_gpu::cut_gpu,
//! camcraft_gpu::cut_run. Our own tests.

use std::time::Instant;

use camcraft_gpu::{
    cut_cpu, cut_or_cpu, cut_or_cpu_on, gpu_required, shared_device, spot_check, CutGrid, CutInput, CutItem,
    CutKernel, CutPath, SPOT_TEXELS,
};

const BALL: &str = include_str!("fixtures/cut_twin_ball_2627.txt");
const FLAT: &str = include_str!("fixtures/cut_twin_flat_2627.txt");

struct Fixture {
    input: CutInput,
    /// FNV-1a 64 of the sim twin's layer bits.
    twin: u64,
    reference: Vec<f32>,
}

fn fnv(layer: &[f32]) -> u64 {
    layer.iter().flat_map(|v| v.to_bits().to_le_bytes()).fold(0xcbf2_9ce4_8422_2325u64, |acc, b| {
        (acc ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn hex(t: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(t, 16).unwrap_or_else(|_| panic!("bad hex {t}")))
}

fn int<T: std::str::FromStr>(t: &str) -> T {
    t.parse().unwrap_or_else(|_| panic!("bad number {t}"))
}

fn fixture(text: &str) -> Fixture {
    let mut grid = None;
    let mut items = Vec::new();
    let mut tex = Vec::new();
    let mut twin = None;
    let mut slot = 0;
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let t: Vec<&str> = line.split_whitespace().collect();
        match t[0] {
            "grid" => grid = Some(CutGrid { dims: [int(t[1]), int(t[2]), int(t[3])], cell: hex(t[4]), cap: hex(t[5]) }),
            "item" => items.push(CutItem {
                a: [hex(t[1]), hex(t[2]), hex(t[3])],
                b: [hex(t[4]), hex(t[5]), hex(t[6])],
                r: hex(t[7]),
                flank: hex(t[8]),
                margin: hex(t[9]),
                ball: t[10] == "1",
                base: [int(t[11]), int(t[12]), int(t[13])],
                lo: [int(t[14]), int(t[15]), int(t[16])],
                hi: [int(t[17]), int(t[18]), int(t[19])],
            }),
            "twin" => twin = Some(u64::from_str_radix(t[1], 16).unwrap_or_else(|_| panic!("bad hash {line}"))),
            "t" => {
                slot += int::<usize>(t[1]);
                tex.push((slot, hex(t[2])));
            }
            _ => panic!("unknown fixture line: {line}"),
        }
    }
    let grid = grid.expect("the fixture has a grid line");
    let n = grid.texels().expect("the fixture grid is good");
    let mut reference = vec![-4.0 * grid.cap; n];
    for (s, w) in tex {
        reference[s] = w;
    }
    Fixture { input: CutInput { grid, items }, twin: twin.expect("the fixture has a twin line"), reference }
}

/// The one-sided law of `stock_cut_twin::parity`, the same tolerance:
/// never more removal than the f64 cut, never more than `tol` less.
/// Returns the largest lag under the f64 cut.
fn one_sided(got: &[f32], f: &Fixture, path: &str) -> f32 {
    let cap = f.input.grid.cap;
    let tol = 96.0 * f32::EPSILON * 64.0 + 1e-4;
    let (mut claimed, mut lag) = (0, 0.0f32);
    assert_eq!(got.len(), f.reference.len(), "{path}: texel count");
    for (n, (g, w)) in got.iter().zip(f.reference.iter()).enumerate() {
        assert!(g.is_finite(), "{path}: texel {n} is {g}");
        let (g, w) = (g.max(-cap), w.max(-cap));
        assert!(g <= w, "{path}: texel {n} removes more than the f64 cut: {g} > {w}");
        assert!(g >= w - tol, "{path}: texel {n} lags the f64 cut by more than {tol}: {g} < {w}");
        lag = lag.max(w - g);
        claimed += usize::from(w > -cap);
    }
    assert!(claimed > 200, "{path}: only {claimed} claimed texels; the fixture is trivial");
    lag
}

fn max_gap(a: &[f32], b: &[f32], cap: f32) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x.max(-cap) - y.max(-cap)).abs()).fold(0.0, f32::max)
}

#[test]
fn the_cpu_law_answers_the_sim_twin_bit_for_bit() {
    for (text, name) in [(BALL, "ball"), (FLAT, "flat")] {
        let f = fixture(text);
        let got = cut_cpu(&f.input).expect("the CPU law cuts");
        assert_eq!(fnv(&got), f.twin, "{name}: the CPU law's layer is not the sim twin's, bit for bit");
        let lag = one_sided(&got, &f, name);
        eprintln!("cut_parity: CPU law ({name}): {} items, {} texels, lag under f64 {lag:e}", f.input.items.len(), got.len());
    }
}

#[test]
fn no_device_falls_back_to_the_cpu_law_with_its_reason() {
    let f = fixture(BALL);
    let out = cut_or_cpu_on(Err("injected: no adapter".into()), &f.input).expect("the CPU law answers");
    assert_eq!(out.path, CutPath::Cpu { reason: "injected: no adapter".into() });
    assert_eq!(out.values, cut_cpu(&f.input).expect("the CPU law cuts"));
}

#[test]
fn a_layer_off_the_cpu_law_fails_the_spot_check() {
    let f = fixture(FLAT);
    let good = cut_cpu(&f.input).expect("the CPU law cuts");
    spot_check(&f.input, &good).expect("the CPU law passes its own check");
    // one probed, claimed texel 0.1 mm deeper than the law: a gouge of
    // the display (the probes are the center cells of every step-th item)
    let [dx, dy, _] = f.input.grid.dims.map(|d| d as usize);
    let step = f.input.items.len().div_ceil(SPOT_TEXELS).max(1);
    let slot = f
        .input
        .items
        .iter()
        .step_by(step)
        .map(|p| {
            let at = [0, 1, 2].map(|a| (i64::from(p.base[a]) + i64::from((p.lo[a] + p.hi[a]) / 2)) as usize);
            (at[2] * dy + at[1]) * dx + at[0]
        })
        .find(|s| good[*s] > -f.input.grid.cap)
        .expect("a probe on a claimed texel");
    let mut deep = good.clone();
    deep[slot] += 0.1;
    let err = spot_check(&f.input, &deep).expect_err("a deeper texel fails");
    assert!(err.contains("not used"), "{err}");
    assert!(spot_check(&f.input, &good[1..]).is_err(), "a short layer fails");
}

#[test]
fn bad_input_refuses_by_name() {
    let f = fixture(FLAT);
    let refuse = |change: &dyn Fn(&mut CutInput), needle: &str| {
        let mut input = f.input.clone();
        change(&mut input);
        let err = cut_cpu(&input).expect_err(needle);
        assert!(err.contains(needle), "{err:?} does not name {needle:?}");
        let err = cut_or_cpu_on(Err("no device".into()), &input).expect_err(needle);
        assert!(err.contains(needle), "{err:?} does not name {needle:?}");
    };
    refuse(&|i| i.items[0].a[0] = f32::NAN, "not finite");
    refuse(&|i| i.items[0].b[2] = 2.0e6, "not finite or past");
    refuse(&|i| i.items[0].r = 0.0, "not a tool size");
    refuse(&|i| i.items[0].flank = 0.0, "no flank");
    refuse(&|i| i.items[0].margin = -1.0, "margin");
    refuse(&|i| (i.items[0].lo[1], i.items[0].hi[1]) = (5, 4), "not inside one brick");
    refuse(&|i| i.items[0].hi[0] = 8, "not inside one brick");
    refuse(&|i| i.items[0].base[2] = -100, "outside the lattice");
    refuse(&|i| i.items[0].base[0] = i.grid.dims[0] as i32, "outside the lattice");
    refuse(&|i| i.grid.dims = [0, 4, 4], "texel budget");
    refuse(&|i| i.grid.dims = [4096, 4096, 4096], "texel budget");
    refuse(&|i| i.grid.cap = f32::INFINITY, "positive finite");
}

/// A larger batch for the timing: eight ramps of `ball` or flat tools
/// over a 96 x 96 x 32 lattice (cell 0.5 mm), split into brick items the
/// way the twin's work list does (window = swept box + cap, clipped).
fn synthetic(ball: bool) -> CutInput {
    let grid = CutGrid { dims: [96, 96, 32], cell: 0.5, cap: 2.0 };
    let (r, flank) = (3.0f64, if ball { 0.0 } else { 12.0 });
    let (cell, cap) = (f64::from(grid.cell), f64::from(grid.cap));
    let mut items = Vec::new();
    for k in 0..8 {
        let y = 4.0 + 5.5 * f64::from(k);
        let (a, b) = ([2.0, y, 14.0 - f64::from(k)], [46.0, y + 3.0, 6.0 + 0.5 * f64::from(k)]);
        let (zlo, zhi) = if ball { (-r, r) } else { (0.0, flank) };
        let lo = [a[0].min(b[0]) - r - cap, a[1].min(b[1]) - r - cap, a[2].min(b[2]) + zlo - cap];
        let hi = [a[0].max(b[0]) + r + cap, a[1].max(b[1]) + r + cap, a[2].max(b[2]) + zhi + cap];
        let c: [(i64, i64); 3] = std::array::from_fn(|i| {
            let top = i64::from(grid.dims[i]) - 1;
            (((lo[i] / cell).floor() as i64).clamp(0, top), ((hi[i] / cell).floor() as i64).clamp(0, top))
        });
        for bz in c[2].0 / 8..=c[2].1 / 8 {
            for by in c[1].0 / 8..=c[1].1 / 8 {
                for bx in c[0].0 / 8..=c[0].1 / 8 {
                    let bk = [bx, by, bz];
                    let o = bk.map(|v| v as f64 * 8.0 * cell);
                    let rel = |p: [f64; 3]| [p[0] - o[0], p[1] - o[1], p[2] - o[2]];
                    let (ra, rb) = (rel(a), rel(b));
                    let reach = ra.iter().chain(rb.iter()).fold(0.0f64, |m, v| m.max(v.abs()));
                    let mut margin = 32.0 * f64::from(f32::EPSILON) * (reach + r + flank + 8.0 * cell);
                    if !ball {
                        let d = [rb[0] - ra[0], rb[1] - ra[1], rb[2] - ra[2]];
                        margin += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() * 1e-6;
                    }
                    let local = |i: usize, v: i64| (v - bk[i] * 8).clamp(0, 7) as u32;
                    items.push(CutItem {
                        a: ra.map(|v| v as f32),
                        b: rb.map(|v| v as f32),
                        r: r as f32,
                        flank: flank as f32,
                        margin: (margin as f32) * (1.0 + f32::EPSILON),
                        ball,
                        base: bk.map(|v| (v * 8) as i32),
                        lo: std::array::from_fn(|i| local(i, c[i].0)),
                        hi: std::array::from_fn(|i| local(i, c[i].1)),
                    });
                }
            }
        }
    }
    CutInput { grid, items }
}

#[test]
fn one_larger_batch_agrees_with_the_cpu_law_and_is_timed() {
    let gpu = match shared_device() {
        Ok(gpu) => gpu,
        Err(why) => {
            assert!(!gpu_required() || why.contains("off"), "CAMCRAFT_REQUIRE_GPU=1 but {why}");
            eprintln!("*** cut_parity: GPU timing SKIPPED - NOT a pass: {why} ***");
            return;
        }
    };
    let compute = gpu.compute.as_ref().expect("shared_device has compute");
    let kernel = CutKernel::new(&gpu.device).unwrap_or_else(|e| panic!("the kernel builds on {}: {e}", gpu.name));
    for ball in [true, false] {
        let input = synthetic(ball);
        let t = Instant::now();
        let cpu = cut_cpu(&input).expect("the CPU law cuts");
        let cpu_time = t.elapsed();
        let t = Instant::now();
        let got = kernel.run(&gpu.device, &gpu.queue, compute, &input).unwrap_or_else(|e| panic!("on {}: {e}", gpu.name));
        let gpu_time = t.elapsed();
        let margin = input.items.iter().map(|i| i.margin).fold(f32::INFINITY, f32::min);
        let gap = max_gap(&got, &cpu, input.grid.cap);
        let claimed = cpu.iter().filter(|v| **v > -input.grid.cap).count();
        assert!(claimed > 10_000, "the batch claims only {claimed} texels");
        assert!(gap <= 0.5 * margin, "max |GPU - CPU law| {gap:e} passes half the smallest margin {margin:e}");
        spot_check(&input, &got).unwrap_or_else(|e| panic!("{e}"));
        eprintln!(
            "cut_parity: {} batch, {} items, {claimed} claimed texels on {}: max |GPU - CPU law| {gap:e} \
             (smallest margin {margin:e}); GPU {gpu_time:?}, CPU {cpu_time:?}",
            if ball { "ball" } else { "flat" },
            input.items.len(),
            gpu.name
        );
    }
}

#[test]
fn the_gpu_cut_never_removes_more_than_the_f64_cut_when_an_adapter_exists() {
    let gpu = match shared_device() {
        Ok(gpu) => gpu,
        Err(why) => {
            assert!(!gpu_required() || why.contains("off"), "CAMCRAFT_REQUIRE_GPU=1 but {why}");
            let out = cut_or_cpu(&fixture(BALL).input).expect("the CPU law answers");
            assert!(matches!(out.path, CutPath::Cpu { .. }), "no device: the CPU law must answer");
            eprintln!("*** cut_parity: GPU path SKIPPED - NOT a pass: {why} ***");
            return;
        }
    };
    let compute = gpu.compute.as_ref().expect("shared_device has compute");
    let t0 = Instant::now();
    let kernel = CutKernel::new(&gpu.device).unwrap_or_else(|e| panic!("the kernel builds on {}: {e}", gpu.name));
    let build = t0.elapsed();
    for (text, name) in [(BALL, "ball"), (FLAT, "flat")] {
        let f = fixture(text);
        let t1 = Instant::now();
        let first = kernel.run(&gpu.device, &gpu.queue, compute, &f.input).unwrap_or_else(|e| panic!("{name} on {}: {e}", gpu.name));
        let gpu_time = t1.elapsed();
        let again = kernel.run(&gpu.device, &gpu.queue, compute, &f.input).expect("the second run");
        assert!(first.iter().zip(&again).all(|(a, b)| a.to_bits() == b.to_bits()), "{name}: two runs differ");
        let lag = one_sided(&first, &f, name);
        let twin = cut_cpu(&f.input).expect("the CPU law cuts");
        assert_eq!(fnv(&twin), f.twin, "{name}: the CPU law is the twin");
        let gap = max_gap(&first, &twin, f.input.grid.cap);
        let margin = f.input.items.iter().map(|i| i.margin).fold(f32::INFINITY, f32::min);
        spot_check(&f.input, &first).unwrap_or_else(|e| panic!("{name}: {e}"));
        let t2 = Instant::now();
        let _ = cut_cpu(&f.input).expect("the CPU law cuts");
        let cpu_time = t2.elapsed();
        let via = cut_or_cpu(&f.input).expect("the shared path answers");
        assert_eq!(via.path, CutPath::Gpu { adapter: gpu.name.clone() }, "{name}: the shared path fell back");
        assert!(via.values.iter().zip(&first).all(|(a, b)| a.to_bits() == b.to_bits()), "{name}: shared and direct runs differ");
        eprintln!(
            "cut_parity: GPU path ran ({name}) on the {} adapter {}: lag under f64 {lag:e}, \
             max |GPU - twin| {gap:e} (smallest margin {margin:e}); kernel build {build:?}, \
             GPU {gpu_time:?}, CPU {cpu_time:?}",
            gpu.kind.label(),
            gpu.name
        );
    }
}
