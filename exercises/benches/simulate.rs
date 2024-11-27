//! Gray scott simulation microbenchmark

use clap::{Args, Command, FromArgMatches};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use grayscott_exercises::{data::Concentrations, options::Options};

pub fn criterion_benchmark(c: &mut Criterion) {
    // Make `clap` parse an empty CLI arguments list to get default arguments
    let opts = Options::from_arg_matches(
        &Options::augment_args(Command::default().no_binary_name(true))
            .get_matches_from(None::<&str>),
    )
    .expect("Failed to parse arguments from defaults & environment");

    // In this benchmark, we are using a criterion feature for grouping
    // benchmarks in logically related sets
    let mut group = c.benchmark_group("simulate");
    for size_pow4 in 1..=6 {
        let num_rows = 4usize.pow(size_pow4);
        let num_cols = 4usize.pow(size_pow4);
        let mut concentrations = Concentrations::new(num_rows, num_cols);

        // One feature of criterion groups is that you can use them to have
        // criterion compute the average time it takes to compute one element.
        group.throughput(Throughput::Elements((num_rows * num_cols) as u64));
        group.bench_function(&format!("{num_cols}x{num_rows}"), |b| {
            b.iter(|| {
                concentrations
                    .update(|start, end| grayscott_exercises::update(&opts.update, start, end))
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
