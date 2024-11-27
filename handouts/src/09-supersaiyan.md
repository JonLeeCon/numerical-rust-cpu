# ILP

Did you know that a modern CPU core can do multiple things in parallel? And by
that, I am not just refering to running different threads on different CPU
cores, or processing multiple data elements in a single CPU instruction. Each
CPU core from a multicore CPU can be executing multiple instructions (possibly
SIMD) at the same time, through the hardware magic of superscalar execution.

In what is becoming by now a recurring theme, however, this extra processing
power does not come for free. The assembly that your code compiles into must
actually feature multiple independent streams of instructions in order to feed
all these superscalar execution units. If each instruction from your code
depends on the previous one, as was the case in our first SIMD sum
implementation, then no superscalar execution will happen, resulting in
inefficient CPU hardware use.

In other words, to run optimally fast, your code must feature a form of
fine-grained concurrency called Instruction-Level Parallelism, or ILP for short. 


## The old and cursed way

Back in the days where dinosaurs roamed the Earth and Fortran 77 was a cool new
language that made the Cobol-74 folks jealous, the expert-sanctioned way to add
N-way instruction-level parallelism to a performance-sensitive computation would
have been to manually unroll your loops and write N copies of your computations
inside of them:

```rust
fn sum_ilp3(v: &Vec<f32>) -> f32 {
    let mut accs = [0.0, 0.0, 0.0];
    let num_chunks = v.len() / 3;
    for i in 0..num_chunks {
        // TODO: Add SIMD here
        accs[0] += v[3 * i];
        accs[1] += v[3 * i + 1];
        accs[2] += v[3 * i + 2];
    }
    let first_irregular_idx = 3 * num_chunks;
    for i in first_irregular_idx..v.len() {
        accs[i - first_irregular_idx] += v[i];
    }
    accs[0] + accs[1] + accs[2]
}
let v = vec![1.2, 3.4, 5.6, 7.8, 9.0];
assert_eq!(sum_ilp3(&v), v.into_iter().sum::<f32>());
```

Needless to say, this way of doing things does not scale well to more complex
computations or high degrees of instruction-level parallelism. And it can also
easily make code a lot harder to maintain, since one must remember to do each
modification to the ILP'd code in N different places. Also, I hope for you that
you will rarely if ever will need to change the N tuning parameter in order to
fit, say, a new CPU architecture with different quantitative parameters.

Thankfully, we are now living in a golden age of computing where high-end
fridges have more computational power than the supercomputers of the days where
this advice was relevant. Compilers have opted to use some of this abundant 
computing power to optimize programs better, and programming languages have
built on top of these optimizations to provide new features that give library
writers a lot more expressive power at little to no runtime performance cost. As
a result, we can now have ILP without sacrificing the maintainability of our
code like we did above.


## The [`iterator_ilp`](https://docs.rs/iterator_ilp/latest/iterator_ilp/) way

First of all, a mandatory disclaimer: I am the maintainer of the `iterator_ilp`
library. It started as an experiment to see if the advanced capabilities of
modern Rust could be leveraged to make the cruft of copy-and-paste ILP obsolete.
Since the experiment went well enough for my needs, I am now sharing it with
you, in the hope that you will also find it useful.

The whole _raison d'être_ of `iterator_ilp` is to take the code that I showed
you above, and make the bold but proven claim that the following code compiles
down to _faster_[^1] machine code:

```rust,ignore
use iterator_ilp::IteratorILP;

fn sum_ilp3(v: &Vec<f32>) -> f32 {
    v.into_iter()
     // Needed until Rust gets stable generics specialization
     .copied()
     .sum_ilp::<3, f32>()
}
# let v = vec![1.2, 3.4, 5.6, 7.8, 9.0];
# assert_eq!(sum_ilp3(&v), v.into_iter().sum::<f32>());
```

Notice how I am able to add new methods to Rust iterators. This leverages a
powerful property of Rust traits, which is that they can be implemented for
third-party types. The requirement for using such an _extension trait_, as they
are sometimes called, is that the trait that adds new methods must be explicitly
brought in scope using a `use` statement, as in the code above.

It's not just that I have manually implemented a special case for floating-point
sums, however. My end goal with this library is that any iterator you can
`fold()`, I should ultimately be able to `fold_ilp()` into the moral
equivalent of the ugly hand-unrolled loop that you've seen above, with only
minimal code changes required on your side. So for example, this should
ultimately be as efficient as hand-optimized ILP:[^2]

```rust,ignore
fn norm_sqr_ilp9(v: &Vec<f32>) -> f32 {
    v.into_iter()
     .copied()
     .fold_ilp::<9, _>(
        || 0.0
        |acc, elem| elem.mul_add(elem, acc),
        |acc1, acc2| acc1 + acc2
     )
}
```


## Exercise

Use `iterator_ilp` to add instruction-level parallelism to your SIMD sum, and
benchmark how close doing so gets you to the peak hardware performance of a
single CPU core.

Due to `std::simd` not being stable yet, it is unfortunately not yet fully
integrated in the broader Rust numerics ecosystem, so you will not be able to
use `sum_ilp()` and will need the following more verbose `fold_ilp()`
alternative:

```rust,ignore
use iterator_ilp::IteratorILP;

let result =
    array_of_simd
        .iter()
        .copied()
        .fold_ilp::<2, _>(
            // Initialize accumulation with a SIMD vector of zeroes
            || Simd::splat(0.0),
            // Accumulate one SIMD vector into the accumulator
            |acc, elem| ...,
            // Merge two SIMD accumulators at the end
            |acc1, acc2| ...,
        );
```

Once the code works, you will have to tune the degree of instruction-level
parallelism carefully:

- Too little and you will not be able to leverage all of the CPU's superscalar
  hardware.
- Too much and you will pass the limit of the CPU's register file, which will
  lead to CPU registers spilling to the stack at a great performance cost.
    * Also, beyond a certain degree of specified ILP, the compiler optimizer
      will often just give up and generate a scalar inner loop, as it does not
      manage to prove that if it tried harder to optimize, it might eventually
      get to simpler and faster code.

For what it's worth, compiler autovectorizers have gotten good enough that you
can actually get the compiler to generate _both_ SIMD instructions and ILP using
nothing but `iterator_ilp` with huge instruction-level parallelism. However,
reductions do not autovectorize that well for various reasons, so the
performance will be worse. Feel free to benchmark how much you lose by using
this strategy!


---

[^1]: It's mainly faster due to the runtime costs of manual indexing in Rust. I
      could rewrite the above code with only iterators to eliminate this
      particular overhead, but hopefully you will agree with me that it would
      make the code even cruftier than it already is.

[^2]: Sadly, we're not there yet today. You saw the iterator-of-reference issue
      above, and there are also still some issues around iterators of tuples
      from `zip()`. But I know how to resolve these issues on the implementation
      side, and once Rust gets generics specialization, I should be able to
      automatically resolve them for you without asking you to call the API
      differently.
