pub mod data;
pub mod io;
pub mod options;

use crate::{
    data::{Concentrations, UV},
    io::HDF5Writer,
    options::{RunnerOptions, UpdateOptions},
};

/// Simulation runner, with a user-specified concentration update function
pub fn run_simulation(
    opts: &RunnerOptions,
    // Notice that we must use FnMut here because the update function can be
    // called multiple times, which FnOnce does not allow.
    mut update: impl FnMut(&UV, &mut UV),
) -> hdf5::Result<()> {
    // Set up the concentrations buffer
    let mut concentrations = Concentrations::new(opts.num_rows, opts.num_cols);

    // Set up HDF5 I/O
    let mut hdf5 = HDF5Writer::create(
        &opts.file_name,
        concentrations.shape(),
        opts.num_output_steps,
    )?;

    // Produce the requested amount of concentration tables
    for _ in 0..opts.num_output_steps {
        // Run a number of simulation steps
        for _ in 0..opts.compute_steps_per_output_step {
            // Update the concentrations of the U and V chemical species
            concentrations.update(&mut update);
        }

        // Write down the current simulation output
        hdf5.write(concentrations.current())?;
    }

    // Close the HDF5 file
    hdf5.close()
}

/// Run an iteration of the Gray-scott simulation
pub fn update(_opts: &UpdateOptions, _start: &UV, _end: &mut UV) {
    todo!("This needs more code to work...")
}

// --- Everything below is course plumbing that need not concern you ---

/// Display the appropriate "cargo examples" command that the user should run in
/// order to skip to the next example (run this at the end of each example).
#[macro_export]
macro_rules! show_skip_command {
    () => {
        let example_name = env!("CARGO_BIN_NAME");
        println!(
            "\nCongratulations! You finished this exercise!\n\
            On your next run, you can skip to the next exercise by using the following command:\n\
            cargo examples --from {example_name} --skip {example_name}\n\n"
        );
    };
}
