# Parallelism

So far, we have been focusing on using a single CPU core efficiently, honoring
some ancient words of software performance optimization wisdom:

> You can have have a second computer, once you know how to use the first one.

But as of this chapter, we have finally reached the point where our floating
point sum makes good use of the single CPU core that it is running on.
Therefore, it's now time to put all those other CPU cores that have been sitting
idle so far to good use.


## Easy parallelism with `rayon`

The Rust standard library only provides low-level parallelism primitives like
threads and mutexes. However, limiting yourself to these would be unwise, as the
third-party Rust library ecosystem is full of multithreading gems. One of them
is the `rayon` crate, which provides equivalents of standard Rust iterators that
automatically distribute your computation over multiple threads of execution.

Getting started with `rayon` is very easy. First you add it as a dependency to
your project...

```bash
cargo add rayon
```

...and then you pick the computation you want to parallelize, and replace
standard Rust iteration with the rayon-provided parallel iteration methods:

```rust,ignore
use rayon::prelude::*;

fn par_sum(v: &Vec<f32>) -> f32 {
    // TODO: Add back SIMD, ILP, etc
    v.into_par_iter().sum()
}
```

That's it, your computation is now running in parallel. By default, it will use
one thread per available CPU hyperthread. You can easily tune this using the
`RAYON_NUM_THREADS` environment variable. And for more advanced use cases like
comparative benchmarking at various numbers of threads, it is also possible to
[configure the thread pool from your
code](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html).


## The power of ownership

Other programming languages also provide easy-looking ways to easily parallelize
computations, like OpenMP in C/++ and Fortran. But it you have tried them, your
experience was probably not great. It is likely that you have run into all kinds
of wrong results, segfaults, and other nastiness.

This happens much more rarely in Rust, because Rust's ownership and borrowing
model has been designed from the start to make multi-threading easier. As a
reminder, in Rust, you can only have one code path writing to a variable at a
same time, or N code paths reading from it. So by construction, data races,
where multiple threads attempt to access data and at least one is writing,
cannot happen in safe Rust. This is great because data races are one of the
worst kinds of parallelism bug. They result in undefined behavior, and therefore
give the compiler and hardware license to trash your program in unpredictable
ways, which they will gladly do.

That being said, it is easy to misunderstand the guarantees that Rust gives you
and get a false sense of security from this, which will come back to bite you
later on. So let's make things straight right now: **Rust does not protect
against all parallelism bugs.** Deadlocks, and race conditions other than data
races where operations are performed in the "wrong" order, can still happen. It
is sadly not the case that just because you are using Rust, you can forget about
decades of research in multi-threaded application architecture and safe
multi-threading patterns. Getting there will hopefully be the job of [the next
generation of programming language research](https://www.ponylang.io/).

What Rust does give you, however, is a language-enforced protection against the
worst kinds of multi-threading bugs, and a vibrant ecosystem of libraries that
make it trivial for you to apply solid multi-threading architectural patterns in
your application in order to protect yourself from the other bugs. And all this
power is available for you today, not hopefully tomorrow if the research goes
well.


## Optimizing the Rayon configuration

Now, if you have benchmarked the above parallel computation, you may have been
disappointed with the runtime performance, especially at low problem size. One
drawback of libraries like Rayon that implement a very general parallelism model
is that they can only be performance-tuned for a specific kind of code, which is
not necessarily the kind of code that you are writing right now. So for specific
computations, fine-tuning may be needed to get optimal results.

In our case, one problem that we have is that Rayon automatically slices our
workload into arbitrarily small chunks, down to a single floating-point number,
in order to keep all of its CPU threads busy. This is appropriate when each
computation from the input iterator is relatively complex, like processing a
full data file, but not for simple computations like floating point sums. At
this scale, the overhead of distributing work and waiting for it to complete
gets much higher than the performance gain brought by parallelizing.

We can avoid this issue by giving Rayon a minimal granularity below
which work should be processed sequentially, using the `par_chunks` method:

```rust,ignore
use rayon::prelude::*;

fn par_sum(v: &Vec<f32>) -> f32 {
    // TODO: This parameter must be tuned empirically for your particular
    //       computation, on your target hardware.
    v.par_chunks(1024)
     .map(seq_sum)
     .sum()
}

// This function will operate sequentially on slices whose length is dictated by
// the tuning parameter given to par_chunks() above.
fn seq_sum(s: &[f32]) -> f32 {
    // TODO: Replace with optimized sequential sum with SIMD and ILP.
    s.into_iter().sum()
}
```

Notice that `par_chunks` method produces a parallel iterator of slices, not
`Vec`s. Slices are simpler objects than `Vec`s, so every `Vec` can be
reinterpreted as a slice, but not every slice can be reinterpreted as a `Vec`.
This is why the idiomatic style for writing numerical code in Rust is actually
to accept slices as input, not `&Vec`. I have only written code that takes
`&Vec` in previous examples to make your learning process easier.

With this change, we can reach the crossover point where the parallel
computation is faster than the sequential one at a smaller input size. But
spawning and joining a parallel job itself has fixed costs, and that cost
doesn't go down when you increase the sequential granularity. So a
computation that is efficient all input sizes would be closer to this:

```rust,ignore
fn sum(v: &Vec<f32>) -> f32 {
    // TODO: Again, this will require workload- and hardware-specific tuning
    if v.len() > 4096 {
        par_sum(v)
    } else {
        seq_sum(v.as_slice())
    }
}
```

Of course, whether you need these optimizations depends on how much you are
interested at performance at small input sizes. If the datasets that you are
processing are always huge, the default `rayon` parallelization logic should
provide you with near-optimal performance without extra tuning. Which is
likely why `rayon` does not perform these optimizations for you by default.


## Exercise

Parallelize one of the computations that you have optimized previously using
`rayon`. You will not need to run `cargo add rayon` in the exercises project,
because for HPC network policy reasons, the dependency needed to be added and
downloaded for you ahead of time.

Try benchmarking the computation at various number of threads, and see how it
affects performance. As a reminder, you can tune the number of threads using the
`RAYON_NUM_THREADS` environment variable. The hardware limit above which you can
expect no benefit from extra threads is the number of system CPUs, which can be
queried using `nproc`. But as you will see, sometimes less threads can be
better.

If you are using Devana, remember to use the `-cN` parameter to `srun` in order to allocate more CPU cores to your job.

If you choose to implement the `par_chunks()` and fully sequential fallback
optimizations, do not forget to adjust the associated tuning parameters for
optimal performance at all input sizes.
