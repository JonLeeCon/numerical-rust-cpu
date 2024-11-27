# Sum and dot

All the computations that we have discussed so far are, in the jargon of SIMD
programming, **vertical** operations. For each element of the input arrays, they
produce zero or one matching output array element. From the perspective of
software performance, these vertical operations are the best-case scenario, and
compilers know how to produce very efficient code out of them without much
assistance from the programmer. Which is why we have not discussed performance
much so far.

But in this chapter, we will now switch our focus to **horizontal** operations,
also known as **reductions**. We will see why these operations are much more
challenging to compiler optimizers, and then the next few chapters will cover
what programmers can do to make them more efficient.


## Summing numbers

One of the simplest reduction operations that one can do with an array of
floating-point numbers is to compute the sum of the numbers. Because this is a
common operation, Rust iterators make it very easy by providing a dedicated
method for it:

```rust
let sum = [1.2, 3.4, 5.6, 7.8].into_iter().sum::<f32>();
```

The only surprising thing here might be the need to spell out the type of the
sum. This need does _not_ come up because the compiler does not know about the
type of the array elements that we are summing. That could be handled by the
default "unknown floats are f64" fallback.

The problem is instead that the `sum()` method is generic in order to be able to
work with both numbers and reference to numbers (which we have not covered yet, 
for now you can think of them as being like C pointers). And wherever there is
genericity in Rust, there is loss of type inference.


## Dot product

Once we have `zip()`, `map()` and `sum()`, it takes only very little work to
combine them in order to implement a simple Euclidean dot product:

```rust
let x = [1.2f32, 3.4, 5.6, 7.8];
let y = [9.8, 7.6, 5.4, 3.2];
let dot = x.into_iter().zip(y)
                       .map(|(x, y)| x * y)
                       .sum::<f32>();
```

Hardware-minded people, however, may know that we are leaving some performance
and floating-point precision on the table by doing it like this. Modern CPUs
come with fused multiply-add operations that are as costly as an addition or
multiplication, and do not round between the multiplication and addition which
results in better output precision.

Rust exposes these hardware operations[^1], and we can use them by switching to
a more general cousin of `sum()` called `fold()`:

```rust
# let x = [1.2f32, 3.4, 5.6, 7.8];
# let y = [9.8, 7.6, 5.4, 3.2];
let dot = x.into_iter().zip(y)
                       .fold(0.0,
                             |acc, (x, y)| x.mul_add(y, acc));
```

Fold works by initializing an accumulator variable with a user-provided value,
and then going through each element of the input iterator and integrating it
into the accumulator, each time producing an updated accumulator. In other
words, the above code is equivalent to this:

```rust
# let x = [1.2f32, 3.4, 5.6, 7.8];
# let y = [9.8, 7.6, 5.4, 3.2];
let mut acc = 0.0;
for (x, y) in x.into_iter().zip(y) {
    acc = x.mul_add(y, acc);
}
let dot = acc;
```

And indeed, the iterator fold method optimizes just as well as the above
imperative code, resulting in identical generate machine code. The problem is
that unfortunately, that imperative code itself is not ideal and will result in
very poor computational performance. We will now show how to quantify this
problem, and later explain why it happens and what you can do about it.


## Setting up `criterion`

### The need for `criterion`

With modern hardware, compiler and operating systems, measuring the performance
of short-running code snippets has become a fine art that requires a fair amount
of care.

Simply surrounding code with OS timer calls and subtracting the readings may
have worked well many decades ago. But nowadays it is often necessary to use
specialized tooling that leverages repeated measurements and statistical
analysis, in order to get stable performance numbers that truly reflect the
code's performance and are reproducible from one execution to another.

The Rust compiler has built-in tools for this, but unfortunately they are not
fully exposed by stable versions at the time of writing, as there is a
longstanding desire to clean up and rework some of the associated APIs before
exposing them for broader use. As a result, third-party libraries should be used
for now. In this course, we will be mainly using
[`criterion`](https://bheisler.github.io/criterion.rs/book/index.html) as it is
by far the most mature and popular option available to Rust developers
today.[^2]

### Adding `criterion` to a project

Because `criterion` is based on the Rust compiler's benchmarking infrastructure,
but cannot fully use it as it is not completely stabilized yet, it does
unfortunately require a fair bit of unclean setup. First you must add a
dependency on `criterion` in your Rust project.

Because this course has been designed to work on HPC centers without Internet
access on worker nodes, we have already done this for you in the example source
code. But for your information, it is done using the following command:

```bash
cargo add --dev criterion
```

`cargo add` is an easy-to-use tool for editing your Rust project's `Cargo.toml`
configuration file in order to register a dependency on (by default) the latest
version of some library. With the `--dev` option, we specify that this
dependency will only be used during development, and should not be included in
production builds, which is the right thing to do for a benchmarking harness.

After this, every time we add a benchmark to the application, we will need to
manually edit the `Cargo.toml` configuration file in order to add an entry that
disables the Rust compiler's built-in benchmark harness. This is done so that it
does not interfere with `criterion`'s work by erroring out on criterion-specific 
CLI benchmark options that it does not expect. The associated `Cargo.toml`
configuration file entry looks like this:

```toml
[[bench]]
name = "my_benchmark"
harness = false
```

Unfortunately, this is not yet enough, because benchmarks can be declared pretty
much anywhere in Rust. So we must _additionally_ disable the compiler's built-in
benchmark harness on every other binary defined by the project. For a simple
library project that defines no extra binaries, this extra `Cargo.toml`
configuration entry should do it:

```toml
[lib]
bench = false
```

It is only after we have done all of this setup, that we can get `criterion` 
benchmarks that will reliably accept our CLI arguments, no matter how they were
started.

### A benchmark skeleton

Now that our Rust project is set up properly for benchmarking, we can start
writing a benchmark. First you need to create a `benches` directory in
your project if it does not already exists, and create a source file there,
named however you like with a `.rs` extension.

Then you must add the `criterion` boilerplate to this source file, which is
partially automated using macros, in order to get a runnable benchmark that
integrates with the standard `cargo bench` tool...

```rust,ignore
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    // This is just an example benchmark that you can freely delete 
    c.bench_function("sqrt 4.2", |b| b.iter(|| black_box(4.2).sqrt()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

...and finally you must add the aforementioned `Cargo.toml` boilerplate so that
criterion CLI arguments keep working as expected. Assuming you unimaginatively
named your benchmark source file "benchmark.rs", this would be...

```toml
[[bench]]
name = "benchmark"
harness = false
```


## Writing a good microbenchmark

There are a few basic rules that you should always follow whenever you are
writing a microbenchmark that measures the performance of a small function in
your code, if you do not want the compiler's optimizer to transform your
benchmark in unrealistic ways:

- Any input value that is known to the compiler's optimizer can be used to tune
  the code specifically for this input value, and sometimes even to reduce the
  benchmarking loop to a single iteration by leveraging the fact that the
  computation always operate on the same input. Therefore, you must always hide
  inputs from the compiler's optimizer using an optimization barrier such as
  `criterion`'s `black_box()`.
- Any output value that is not used in any way is a useless computation in the
  eyes of the compiler's optimizer. Therefore, the compiler's optimizer will
  attempt to delete any code that is involved in the computation of such values.
  To avoid this, you will want again to feed results into an optimization
  barrier like `criterion`'s `black_box()`. `criterion` implicitly does this for
  any output value that you return from the `iter()` API's callback.

It's not just about the compiler's optimizer though. Hardware and operating
systems can also leverage the regular nature of microbenchmarks to optimize
performance in unrealistic ways, for example CPUs will exhibit an unusually good
cache hit rate when running benchmarks that always operate on the same input
values. This is not something that you can guard against, just a pitfall that
you need to keep in mind when interpreting benchmark results: absolute timings
are usually an overly optimistic estimate of your application's performance, and
therefore the most interesting output of microbenchmarks is actually not the raw
result but the relative variations of this result when you change the code that
is being benchmarked.

Finally, on a more fundamental level, you must understand that on modern
hardware, performance usually depends on problem size in a highly nonlinear and
non-obvious manner. It is therefore a good idea to test the performance of your
functions over a wide range of problem sizes. Geometric sequences of problem
sizes like 1, 2, 4, 8, 16, 32, ... are often a good default choice.


## Exercise

Due to Rust benchmark harness design choices, the exercise for this chapter
will, for once, not take place in the `examples` subdirectory of the exercises'
source tree.

Instead, you will mainly work on the `benches/06-summit.rs` source file, which
is a Criterion benchmark that was created using the procedure described above.

Implement the `sum()` function within this benchmark to make it sum the elements
of its input `Vec`, then run the benchmark with `cargo bench --bench 06-summit`.

If you are using the Devana cluster, you will want to run the benchmarks on a
worker node by prefixing this command with `srun`.

To correctly interpret the results, you should know that a single core of a 
modern x86 CPU, with a 2 GHz clock rate and AVX SIMD instructions, can perform
32 billion f32 sums per second.[^3]


---

[^1]: Technically, at this stage of our performance optimization journey, we can
      only use these operations via a costly libm function call. This happens
      because not all x86_64 CPUs support the `fma` instruction family, and the
      compiler has to be conservative in order to produce code that runs
      everywhere. Later, we will see how to leverage modern hardware better
      using the `multiversion` crate.

[^2]: Although criterion would be my recommendation today due to its
      feature-completeness and popularity, [`divan`](http://docs.rs/divan) is
      quickly shaping up into an interesting alternative that might make it the
      recommendation in a future edition of this course. Its benefits over
      `criterion` include significantly improved API ergonomics, faster
      measurements, and better support for code that is generic over
      compile-time quantities aka "const generics".

[^3]: Recent Intel CPUs (as of 2024) introduced the ability to perform a third
      SIMD sum per clock cycle, which bumps the theoretical limit to 48 billion
      f32 sums per second per 2 GHz CPU core, or even double that on those few
      CPUs that support AVX-512 with native-width ALUs.
