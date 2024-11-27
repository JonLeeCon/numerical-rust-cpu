# Parallelism

Well, that last chapter was a disappointment. All this refactoring work, only to
find out in the final microbenchmark that the optimization we implemented does
improve L1d cache hit rate (as can be measured using `perf stat -d`), but this
improvement in this CPU utilization efficiency metric does not translate into an
improvement in execution speed.

The reason is not clear to me at this point in time, unfortunately. It could be
several things:

- The CPU manages to hide the latency of L1d cache misses by executing other
  pending instructions. So even at a 20% L1d cache miss rate we are still not
  memory-bound.
- The optimization is somehow not implemented properly and costs more than it
  helps. I have checked for absence of simple issues here, but there could be
  more subtle ones around.
- There is another, currently unknown factor preventing the execution of more
  instructions per cycles. So even if data is more readily available in the L1d
  cache, we still can't use it yet.

Hopefully I will find the time to clarify this before the next edition of this
school. But for now, let us move to the last optimization that should be
performed once we are convinced that we are using a single CPU core as
efficiently as we can, namely using all of our other CPU cores.


## Another layer of loop blocking

There is an old saying that almost every problem in programming can be resolved
by adding another layer of indirection. However it could just as well be argued
that almost every problem in software performance optimization can be resolved
by adding another layer of loop blocking.

In this particular case, the loop blocking that we are going to add revolves
around slicing our simulation domain into independent chunks that can be
processed by different CPU cores for parallelism. Which begs the question: in
which direction should all those chunks be cut? Across rows or columns? And how
big should they be?

Let's start with the first question. We are using `ndarray`, which in its
default configuration stores data in row-major order. And we also know that CPUs
are _very_ fond of iterating across data in long linear patterns, and will make
you pay a hefty price for any sort of jump across the underlying memory buffer.
Therefore, we should think twice before implementing any sort of chunking that
makes the rows that we are iterating over shorter, which means that chunking the
data into blocks of rows for parallelization is a better move.

As for how big the chunks should be, it is basically a balance between two
factors:

- Exposing more opportunities for parallelism and load balancing by cutting
  smaller chunks. This pushes us towards cutting the problem into at least `N`
  chunks where N is the number of CPU cores that our system has, and preferably
  more to allow for dynamic load balancing of tasks between CPU cores when some
  cores process work slower than others.[^1]
- Amortizing the overhead of spawning and awaiting parallel work by cutting
  larger chunks. This pushes us towards cutting the problem into chunks no
  smaller than a certain size, dictated by processing speed and task spawning
  and joining overhead.

The `rayon` library can take care of the first concern for us by dynamically
splitting work as many times as necessary to achieve good load balancing on the
specific hardware and system that we're dealing with at runtime. But as we have
seen before, it is not good at enforcing sequential processing cutoffs. Hence we
will be taking that matter into our own hands.


## Configuring the minimal block size

In the last chapter, we have been using a hardcoded safety factor to pick the
number of columns in each block, and you could hopefully see during the
exercises that this made the safety factor unpleasant to tune. This chapter will
thus introduce you to the superior approach of making the tuning parameters
adjustable via CLI parameters and environment variables.

`clap` makes this very easy. First we enable the environment variable feature...

```bash
cargo add --features=env clap
```

...we add the appropriate option to our `UpdateOptions` struct...

```rust,ignore
#[derive(Debug, Args)]
pub struct UpdateOptions {
    // ... unchanged existing options ...

    /// Minimal number of data points processed by a parallel task
    #[arg(env, long, default_value_t = 100)]
    pub min_elems_per_parallel_task: usize,
}
```

That's it. Now if we either pass the `--min-elems-per-parallel-task` option to
the `simulate` binary or set the `MIN_ELEMS_PER_PARALLEL_TASK` environment
variable, that can be used as our sequential processing granularity in the code
that we are going to write next.


## Adding parallelism

We then begin our parallelism journey by enabling the `rayon` support of the
`ndarray` crate. This enables [some `ndarray`
producers](https://docs.rs/ndarray/latest/ndarray/parallel/index.html) to be
turned into Rayon parallel iterators.

```bash
cargo add --features=rayon ndarray
```

Next we split our `update()` function into two:

- One top-level `update()` function that will be in charge of receiving user
  parameters, extracting the center of the output arrays, and parallelizing the
  work if deemed worthwhile.
- One inner `update_seq()` function that will do most of the work that we did
  before, but using array windows instead of manipulating the full concentration
  arrays directly.

Overall, it looks like this:

```rust,ignore
/// Parallel simulation update function
pub fn update<const SIMD_WIDTH: usize>(
    opts: &UpdateOptions,
    start: &UV<SIMD_WIDTH>,
    end: &mut UV<SIMD_WIDTH>,
    cols_per_block: usize,
) where
    LaneCount<SIMD_WIDTH>: SupportedLaneCount,
{
    // Extract the center of the output domain
    let center_shape = end.simd_shape().map(|dim| dim - 2);
    let center = s![1..=center_shape[0], 1..=center_shape[1]];
    let mut end_u_center = end.u.slice_mut(center);
    let mut end_v_center = end.v.slice_mut(center);

    // Translate the element-based sequential iteration granularity into a
    // row-based granularity.
    let min_rows_per_task = opts
        .min_elems_per_parallel_task
        .div_ceil(end_u_center.ncols() * SIMD_WIDTH);

    // Run the sequential simulation
    if end_u_center.nrows() > min_rows_per_task {
        // TODO: Run the simulation in parallel
    } else {
        // Run the simulation sequentially
        update_seq(
            opts,
            start.u.view(),
            start.v.view(),
            end_u_center,
            end_v_center,
            cols_per_block,
        );
    }
}

/// Sequential update on a subset of the simulation domain
#[multiversion(targets("x86_64+avx2+fma", "x86_64+avx", "x86_64+sse2"))]
pub fn update_seq<const SIMD_WIDTH: usize>(
    opts: &UpdateOptions,
    start_u: ArrayView2<'_, Vector<SIMD_WIDTH>>,
    start_v: ArrayView2<'_, Vector<SIMD_WIDTH>>,
    mut end_u: ArrayViewMut2<'_, Vector<SIMD_WIDTH>>,
    mut end_v: ArrayViewMut2<'_, Vector<SIMD_WIDTH>>,
    cols_per_block: usize,
) where
    LaneCount<SIMD_WIDTH>: SupportedLaneCount,
{
    // Slice the output domain into vertical blocks for L1d cache locality
    let num_blocks = end_u.ncols().div_ceil(cols_per_block);
    let end_u = end_u.axis_chunks_iter_mut(Axis(1), cols_per_block);
    let end_v = end_v.axis_chunks_iter_mut(Axis(1), cols_per_block);

    // Iterate over output blocks
    for (block_idx, (end_u, end_v)) in end_u.zip(end_v).enumerate() {
        let is_last = block_idx == (num_blocks - 1);

        // Slice up input blocks of the right width
        let input_base = block_idx * cols_per_block;
        let input_slice = if is_last {
            Slice::from(input_base..)
        } else {
            Slice::from(input_base..input_base + cols_per_block + 2)
        };
        let start_u = start_u.slice_axis(Axis(1), input_slice);
        let start_v = start_v.slice_axis(Axis(1), input_slice);

        // Process current input and output blocks
        for (win_u, win_v, out_u, out_v) in stencil_iter(start_u, start_v, end_u, end_v) {
            // TODO: Same code as before
        }
    }
}
```

Once this is done, parallelizing the loop becomes a simple matter of
implementing loop blocking as we did before, but across rows, and iterating over
the blocks using Rayon parallel iterators instead of sequential iterators:

```rust,ignore
// Bring Rayon parallel iteration in scope
use rayon::prelude::*;

// Slice the output domain into horizontal blocks for parallelism
let end_u = end_u_center.axis_chunks_iter_mut(Axis(0), min_rows_per_task);
let end_v = end_v_center.axis_chunks_iter_mut(Axis(0), min_rows_per_task);

// Iterate over parallel tasks
let num_tasks = center_shape[0].div_ceil(min_rows_per_task);
end_u
    .into_par_iter()
    .zip(end_v)
    .enumerate()
    .for_each(|(task_idx, (end_u, end_v))| {
        let is_last_task = task_idx == (num_tasks - 1);

        // Slice up input blocks of the right height
        let input_base = task_idx * min_rows_per_task;
        let input_slice = if is_last_task {
            Slice::from(input_base..)
        } else {
            Slice::from(input_base..input_base + min_rows_per_task + 2)
        };
        let start_u = start.u.slice_axis(Axis(0), input_slice);
        let start_v = start.v.slice_axis(Axis(0), input_slice);

        // Process the current block sequentially
        update_seq(opts, start_u, start_v, end_u, end_v, cols_per_block);
    });
```


## Exercise

Integrate these changes into your codebase, then adjust the available tuning
parameters for optimal runtime performance:

- First, adjust the number of threads that `rayon` uses via the
  `RAYON_NUM_THREADS` environment variable. If the machine that you are running
  on has hyper-threading enabled, it is almost always a bad idea to use it on
  performance-optimized code, so using half the number of system-reported CPUs
  will already provide a nice speedup. And since `rayon` is not NUMA-aware yet,
  using more threads than the number of cores in one NUMA domain (which you
  can query using `lscpu`) may not be worthwhile.
- Next, try to tune the `MIN_ELEMS_PER_PARALLEL_TASK` parameter. Runtime
  performance is not very sensitive to this parameter, so you will want to start
  with big adjustments by factors of 10x more or 10x less, then fine-tune with
  smaller adjustements once you find a region of the parameter space that seems
  optimal. Finally, adjust the defaults to your tuned value.

And with that, if we ignore the small wrinkle of cache blocking not yet working
in the manner where we would expect it to work (which indicates that there is a
bug in either our cache blocking implementation or our expectation of its
impact), we have taken Gray-Scott reaction computation performance as far as
Rust will let us on CPU.


---

[^1]: Load balancing becomes vital for performance as soon as your system has
      CPU cores of heterogeneous processing capabilities like Arm's big.LITTLE,
      Intel's Adler Lake, and any CPU that has per-core turbo frequencies. But
      even on systems with homogeneous CPU core processing capabilities, load
      imbalance can dynamically occur as a result of e.g. interrupts from some
      specific hardware being exclusively processed by one specific CPU core.
      Therefore designing your program to allow for some amount of load
      balancing is a good idea as long as the associated task spawning and
      joining work does not cost you too much.
