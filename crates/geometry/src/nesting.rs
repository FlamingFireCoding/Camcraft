//! Nesting building blocks: the CPU raster filter of the raster nest (#2525).
//!
//! Provenance: the raster ("no-fit raster") method of the nesting literature -
//! Bennell and Oliveira, "The geometry of nesting problems: a tutorial", EJOR
//! 2008 - and the conservative-coverage rules of research comment 2 on #2525.
//! Nothing here reaches posted output or a verdict: a raster only filters
//! candidates, and the exact `geo2d` check decides.

pub mod raster;
