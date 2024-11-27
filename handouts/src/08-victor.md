# SIMD

Single Instruction Multiple Data, or SIMD, is a very powerful hardware feature
which lets you manipulate multiple numbers with a single CPU instruction. This
means that if you play your cards right, code that uses SIMD can be from 2x to
64x faster than code that doesn't, depending on what hardware you are running on
and what data type you are manipulating.

Unfortunately, SIMD is also a major pain in the bottom as a programmer, because
the set of operations that you can efficiently perform using SIMD instructions
is extremely limited, and the performance of these operations is extremely
sensitive to hardware details that you are not used to caring about, such as
data alignment, contiguity and layout.


## Why won't the compiler do it?

People new to software performance optimization are often upset when they learn
that SIMD is something that they need to take care of. They are used to
optimizing compilers acting as wonderful and almighty hardware abstraction
layers that usually generate near-optimal code with very little programmer
effort, and rightfully wonder why when it comes to SIMD, the abstraction layer
fails them and they need to take the matter into their own hands.

Part of the answer lies in this chapter's introductory sentences. SIMD
instruction sets are an absolute pain for compilers to generate code for,
because they are so limited and sensitive to detail that within the space of all
possible generated codes, the code that will actually work and run fast is
basically a tiny corner case that cannot be reached through a smooth,
progressive optimization path. This means that autovectorization code is usually
the hackiest, most special-case-driven part of an optimizing compiler codebase.
And you wouldn't expect such code to work reliably.

But there is another side to this story, which is the code that **you** wrote.
Compilers forbid themselves from performing certain optimizations when it is
felt that they would make the generated code too unrelated to the original code,
and thus impossible to reason about. For example, reordering the elements of an
array in memory is generally considered to be off-limits, and so is
reassociating floating-point operations, because this changes where
floating-point rounding approximations are performed. In sufficiently tricky
code, shifting roundings around can make the difference between producing a
fine, reasonably accurate numerical result on one side, and accidentally
creating a pseudorandom floating-point number generator on the other side.

---

When it comes to summing arrays of floating point numbers, the second factor
actually dominates. You asked the Rust compiler to do this:

```rust
fn sum_my_floats(v: &Vec<f32>) -> f32 {
    v.into_iter().sum()
}
```

...and the Rust compiler magically translated your code into something like this:

```rust
fn sum_my_floats(v: &Vec<f32>) -> f32 {
    let mut acc = 0.0;
    for x in v.into_iter() {
        acc += x;
    }
    acc
}
```

But that loop itself is actually not anywhere close to an optimal SIMD
computation, which would be conceptually closer to this:

```rust
// This is a hardware-dependent constant, here picked for x86's AVX
const HARDWARE_SIMD_WIDTH: usize = 8;

fn simd_sum_my_floats(v: &Vec<f32>) -> f32 {
    let mut accs = [0.0; HARDWARE_SIMD_WIDTH];
    let chunks = v.chunks_exact(HARDWARE_SIMD_WIDTH);
    let remainder = chunks.remainder();
    // We are not doing instruction-level parallelism for now. See next chapter.
    for chunk in chunks {
        // This loop will compile into a single fast SIMD addition instruction
        for (acc, element) in accs.iter_mut().zip(chunk) {
            *acc += *element;
        }
    }
    for (acc, element) in accs.iter_mut().zip(remainder) {
        *acc += *element;
    }
    // This would need to be tuned for optimal efficiency at small input sizes
    accs.into_iter().sum()
}
```

...and there is simply no way to go from `sum_my_floats` to `simd_sum_my_floats` 
without reassociating floating-point operations. Which is not a nice thing to do
behind the original code author's back, for reasons that [numerical computing
god William Kahan](https://people.eecs.berkeley.edu/~wkahan/) will explain much
better than I can in his many papers and presentations.

All this to say: yes there is unfortunately no compiler optimizer free 
lunch with SIMD reductions and you will need to help the compiler a bit in order
to get there...


## A glimpse into the future: `portable_simd`

Unfortunately, the influence of Satan on the design of SIMD instruction sets
does not end at the hardware level. The [hardware vendor-advertised APIs for
using
SIMD](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html)
hold the dubious distinction of feeling even worse to use. But
thankfully, there is some hope on the Rust horizon.

The GCC and clang compilers have long provided a SIMD hardware abstraction
layer, which provides an easy way to access the common set of SIMD operations
that all hardware vendors agree is worth having (or at least is efficient
enough to emulate on hardware that disagrees). As is too common when compiler
authors design user interfaces, the associated C API [is not exactly
intuitive](https://gcc.gnu.org/onlinedocs/gcc/Vector-Extensions.html), and is
therefore rarely used by C/++ practicioners. But a small but well-motivated team
has been hard at work during the past few years, building a nice high-level Rust
API on top of this low-level compiler backend functionality.

This project is currently integrated into nightly versions of the Rust compiler
as the [`std::simd`](https://doc.rust-lang.org/std/simd/index.html) experimental
standard library module. It is an important project because if it succeeds at
being integrated into stable Rust on a reasonable time frame, it might actually 
be the first time a mainstream programming language provides a standardized[^1]
API for writing SIMD code, that works well enough for common use cases without
asking programmers to write one separate code path for each supported hardware
instruction set.

This becomes important when you realize that x86 released more than 35[^2]
extensions to its SIMD instruction set in around 40 years of history, while Arm
has been maintaining two families of incompatible SIMD instruction set
extensions with completely different API logic[^3] across their platform for a
few years now and will probably need to keep doing so for many decades to come.
And that's just the two main CPU vendors as of 2024, not accounting for the
wider zoo of obscure embedded CPU architectures that application domains like
space programs need to cope with. Unless you are Intel or Arm and have thousands
of people-hours to spend on maintaining dozens of optimized backends for your
mathematical libraries, achieving portable SIMD performance through hand-tuned
hardware-specific code paths is simply not feasible anymore in the 21st century.

In contrast, using `std::simd` and the [`multiversion`
crate](https://docs.rs/multiversion/latest/multiversion/)[^4], a reasonably
efficient SIMD-enabled floating point number summing function can be written
like this...

```rust,ignore
#![feature(portable_simd)]  // Release the nightly compiler kraken

use multiversion::{multiversion, target::selected_target};
use std::simd::prelude::*;

#[multiversion(targets("x86_64+avx2+fma", "x86_64+avx", "x86_64+sse2"))]
fn simd_sum(x: &Vec<f32>) -> f32 {
    // This code uses a few advanced language feature that we do not have time
    // to cover during this short course. But feel free to ask the teacher about
    // it, or just copy-paste it around.
    const SIMD_WIDTH: usize = const {
        if let Some(width) = selected_target!().suggested_simd_width::<f32>() {
            width
        } else {
            1
        }
    };
    let (peel, body, tail) = x.as_simd::<SIMD_WIDTH>();
    let simd_sum = body.into_iter().sum::<Simd<f32, SIMD_WIDTH>>();
    let scalar_sum = peel.into_iter().chain(tail).sum::<f32>();
    simd_sum.reduce_sum() + scalar_sum
}
```

...which, as a famous TV show would put it, may not look _great_, but is
definitely not terrible.


## Back from the future: [`slipstream`](https://docs.rs/slipstream/latest/slipstream/) & [`safe_arch`](https://docs.rs/slipstream/latest/slipstream/)

The idea of using experimental features from a nightly version of the Rust
compiler may send shivers down your spine, and that's understandable.
Having your code occasionally fail to build because the _language standard
library_ just changed under your feet is really not for everyone.

If you need to target current stable versions of the Rust compilers, the main
alternatives to `std::simd` that I would advise using are...

- [`slipstream`](https://docs.rs/slipstream/latest/slipstream/), which tries to
  do the same thing as `std::simd`, but using autovectorization instead of
  relying on direct compiler SIMD support. It usually generates worse SIMD code
  than `std::simd`, which says a lot about autovectorization, but for simple
  things, a slowdown of "only" ~2-3x with respect to peak hardware performance
  is achievable.
- [`safe_arch`](https://docs.rs/slipstream/latest/slipstream/), which is
  x86-specific, but provides a very respectable attempt at making the Intel SIMD
  intrinsics usable by people who were not introduced to programming through the
  afterschool computing seminars of the cult of Yog-Sothoth.

But as you can see, you lose quite a bit by settling for these, which is why
Rust would really benefit from getting `std::simd` stabilized sooner rather than
later. If you know of a SIMD expert who could help at this task, please consider
attempting to nerd-snipe her into doing so!


## Exercise

The practical work environement has already been configured to use a nightly
release of the Rust compiler, which is version-pinned for reproducibility.

Integrate `simd_sum` into the benchmark and compare it to your previous
optimized version. As often with SIMD, you should expect worse performance on
very small inputs, followed by much improved performance on larger inputs, which
will degrade back into less good performance as the input size gets so large
that you start trashing the fastest CPU caches and hitting slower memory tiers.

Notice that the `#![feature(portable_simd)]` experimental feature enablement 
directive must be at the top of the benchmark source file, before any other
program declaration. The other paragraphs of code can be copied anywhere you
like, including after the point where they are used.

If you would fancy a more challenging exercise, try implementing a dot product
using the same logic.


---

[^1]: Standardization matters here. Library-based SIMD abstraction layers have
      been around for a long while, but since SIMD hates abstraction and
      inevitably ends up leaking through APIs sooner rather than later, it is
      important to have common language vocabulary types so that everyone is
      speaking the same SIMD language. Also, SIMD libraries have an unfortunate
      tendency to only correctly cover the latest instruction sets from the most
      popular hardware vendors, leaving other hardware manufacturers out in the
      cold and thus encouraging hardware vendor lock-in that this world doesn't
      need.

[^2]: [MMX](https://en.wikipedia.org/wiki/MMX_(instruction_set)),
[3DNow!](https://en.wikipedia.org/wiki/3DNow!),
[SSE](https://en.wikipedia.org/wiki/Streaming_SIMD_Extensions),
[SSE2](https://en.wikipedia.org/wiki/SSE2),
[SSE3](https://en.wikipedia.org/wiki/SSE3),
[SSSE3](https://en.wikipedia.org/wiki/SSSE3) (not a typo!),
[SSE4](https://en.wikipedia.org/wiki/SSE4),
[AVX](https://en.wikipedia.org/wiki/Advanced_Vector_Extensions),
[F16C](https://en.wikipedia.org/wiki/F16C),
[XOP](https://en.wikipedia.org/wiki/XOP_instruction_set), [FMA4 and
FMA3](https://en.wikipedia.org/wiki/FMA_instruction_set),
[AVX2](https://en.wikipedia.org/wiki/AVX2), the 19 different subsets of
[AVX-512](https://en.wikipedia.org/wiki/AVX-512),
[AMX](https://en.wikipedia.org/wiki/Advanced_Matrix_Extensions), and most
recently at the time of writing, [AVX10.1 and the upcoming
AVX10.2](https://en.wikipedia.org/wiki/AVX10). Not counting more specialized ISA
extension that would also arguably belong to this list like
[BMI](https://en.wikipedia.org/wiki/Bit_Manipulation_Instruction_Sets) and the
various cryptography primitives that are commonly (ab)used by the implementation
of fast PRNGs.

[^3]: [NEON](https://en.wikipedia.org/wiki/ARM_architecture_family#Advanced_SIMD_(NEON))
and
[SVE](https://en.wikipedia.org/wiki/AArch64#Scalable_Vector_Extension_(SVE)),
both of which come with many sub-dialects analogous to the x86 SIMD menagerie.

[^4]: Which handles the not-so-trivial matter of having your code adapt at
      runtime to the hardware that you have, without needing to roll out one
      build per cursed revision of the x86/Arm SIMD instruction set.
