//! Benchmarking sum of array elements (for now)

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Function that we're trying to benchmark
fn sum(v: Vec<f32>) -> f32 {
    todo!("Sum the values {v:?}")
}

// Benchmark input that we're using
fn make_input(size: usize) -> Vec<f32> {
    (0..size).map(|x| x as f32 / (size as f32).sqrt()).collect()
}

// Benchmark for various problem sizes
pub fn criterion_benchmark(c: &mut Criterion) {
    for size_pow16 in 0..=6 {
        let size = 16usize.pow(size_pow16);
        c.bench_function(&format!("sum {size}"), |b| {
            b.iter(|| sum(black_box(make_input(size))))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
