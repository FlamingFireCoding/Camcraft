//! camcraft-gpu: the headless GPU compute seam (#2620). One windowless
//! device for each process, a capability probe, the off switch and the
//! CPU fallback rule, and the first kernel (conservative facet binning).
//! Provenance: our own seam over wgpu's public API; the binning law is
//! uniform-grid partitioning from C. Ericson, "Real-Time Collision
//! Detection" (2005), ch. 7.
//!
//! The fallback rule: every entry answers `Result`. An `Err` names the
//! reason (no adapter, no compute, the switch is off, a lost device, a
//! buffer past the limit, a failed kernel or read-back), and the caller
//! runs its own CPU code. A GPU answer is only a filter: the caller's f64
//! law stays the judge, so no posted output depends on this crate.

mod adapter;
pub mod binning;
#[cfg(not(target_arch = "wasm32"))]
mod binning_gpu;
pub mod probe;
#[cfg(not(target_arch = "wasm32"))]
mod shared;
pub mod switch;

pub use adapter::{device_async, gpu_required, AdapterKind, HeadlessGpu, REQUIRE_GPU_ENV};
#[cfg(not(target_arch = "wasm32"))]
pub use adapter::{device, device_for_test};
pub use binning::{bin_twin, pad_boxes, plan, CellLists, GridSpec, PaddedBoxes, Plan};
#[cfg(not(target_arch = "wasm32"))]
pub use binning_gpu::{bin_facets, bin_on};
pub use probe::{probe, Compute};
#[cfg(not(target_arch = "wasm32"))]
pub use shared::shared_device;
pub use switch::{compute_switch, COMPUTE_ENV};
