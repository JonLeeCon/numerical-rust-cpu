use clap::Parser;
use grayscott_exercises::{options::Options, run_simulation};
use indicatif::ProgressBar;

fn main() -> hdf5::Result<()> {
    // Parse command line options
    let options = Options::parse();

    // Set up the progress bar
    let progress = ProgressBar::new(
        (options.runner.num_output_steps * options.runner.compute_steps_per_output_step) as u64,
    );

    // Run the simulation
    run_simulation(&options.runner, |start, end| {
        // TODO: Make this line of code work
        grayscott_exercises::update(&options.update, start, end);
        progress.inc(1);
    })?;

    // Declare the computation finished
    progress.finish();
    Ok(())
}
