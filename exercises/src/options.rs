//! Configurable and non-configurable parameters of the simulation

use crate::data::Float;
use clap::{Args, Parser};

/// Simulation runner options
#[derive(Debug, Args)]
pub struct RunnerOptions {
    /// Number of rows in the concentration table
    #[arg(short = 'r', long = "nbrow", default_value_t = 1080)]
    pub num_rows: usize,

    /// Number of columns in the concentration table
    #[arg(short = 'c', long = "nbcol", default_value_t = 1920)]
    pub num_cols: usize,

    /// Output file name
    #[arg(short = 'o', long = "output", default_value = "output.h5")]
    pub file_name: String,

    /// Number of simulation steps to write to the output file
    #[arg(short = 'n', long = "nbimage", default_value_t = 1000)]
    pub num_output_steps: usize,

    /// Number of computation steps to run between each write
    #[arg(short = 'e', long = "nbextrastep", default_value_t = 34)]
    pub compute_steps_per_output_step: usize,
}

/// Simulation update options
#[derive(Debug, Args)]
pub struct UpdateOptions {
    /// Speed at which U is added to the simulation and U, V and P are removed
    #[arg(short, long, default_value_t = 0.014)]
    pub feedrate: Float,

    /// Speed at which V turns into P
    #[arg(short, long, default_value_t = 0.054)]
    pub killrate: Float,

    /// Simulated time interval on each simulation step
    #[arg(short = 't', long, default_value_t = 1.0)]
    pub deltat: Float,
}

/// Gray-Scott reaction simulation
///
/// This program simulates the Gray-Scott reaction through a finite difference
/// schema that gets integrated via the Euler method.
#[derive(Debug, Parser)]
#[command(version)]
pub struct Options {
    #[command(flatten)]
    pub runner: RunnerOptions,
    #[command(flatten)]
    pub update: UpdateOptions,
}

/// Weights of the discrete convolution stencil
#[rustfmt::skip]
pub const STENCIL_WEIGHTS: [[Float; 3]; 3] = [
    [0.25, 0.5, 0.25],
    [0.5,  0.0, 0.5],
    [0.25, 0.5, 0.25]
];

/// Offset from the top-left corner of STENCIL_WEIGHTS to its center
pub const STENCIL_OFFSET: [usize; 2] = [1, 1];

/// Diffusion rate of the U species
pub const DIFFUSION_RATE_U: Float = 0.1;

/// Diffusion rate of the V species
pub const DIFFUSION_RATE_V: Float = 0.05;
