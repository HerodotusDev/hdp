//! The Data Processor CLI serves as an essential tool for developers working with Cairo programs and zkVM environments.
//! Its primary function is to translate human-readable requests into a format compatible with Cairo programs,
//! enabling commands to be executed over the Cairo VM and generating executable outputs.
//! This transformation is a crucial preprocessing step that prepares data for off-chain computations in zkVM environments.

pub mod cairo_runner;
pub mod constant;
pub mod hdp_run;
pub mod preprocessor;
pub mod primitives;
pub mod processor;
pub mod provider;

pub use hdp_run::run;
