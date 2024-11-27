# FMA vs ILP

If you paid close attention during the first part of this course, you may have
been thinking for a while now that there is some code in the Gray-Scott
computation that looks like a perfect candidate for introducing fused
multiply-add CPU instructions.

More specifically, if we look at an iteration of the update loop...

```rust,ignore
// Access the SIMD data corresponding to the center concentration
let u = win_u[STENCIL_OFFSET];
let v = win_v[STENCIL_OFFSET];

// Compute the diffusion gradient for U and V
let [full_u, full_v] = (win_u.rows().into_iter())
    .zip(win_v.rows())
    .zip(STENCIL_WEIGHTS)
    .flat_map(|((u_row, v_row), weights_row)| {
        (u_row.into_iter().copied())
            .zip(v_row.into_iter().copied())
            .zip(weights_row)
    })
    .fold(
        [Vector::splat(0.); 2],
        |[acc_u, acc_v], ((stencil_u, stencil_v), weight)| {
            let weight = Vector::splat(weight);
            [
                acc_u + weight * (stencil_u - u),
                acc_v + weight * (stencil_v - v),
            ]
        },
    );

// Compute SIMD versions of all the float constants that we use
let diffusion_rate_u = Vector::splat(DIFFUSION_RATE_U);
let diffusion_rate_v = Vector::splat(DIFFUSION_RATE_V);
let feedrate = Vector::splat(opts.feedrate);
let killrate = Vector::splat(opts.killrate);
let deltat = Vector::splat(opts.deltat);
let ones = Vector::splat(1.0);

// Compute the output values of u and v
let uv_square = u * v * v;
let du = diffusion_rate_u * full_u - uv_square + feedrate * (ones - u);
let dv = diffusion_rate_v * full_v + uv_square - (feedrate + killrate) * v;
*out_u = u + du * deltat;
*out_v = v + dv * deltat;
```

...we can see that the diffusion gradient's `fold()` statement and the
computation of the output values of `u` and `v` are full of floating point
multiplications followed by additions.

It would surely seem sensible to replace these with fused multiply-add
operations that compute multiplication-addition pairs 2x faster and more
accurately at the same time!


## Free lunch at last?

Enticed by the prospect of getting an easy 2x performance speedup at last, we
might proceed to rewrite all the code using fused multiply-add operations...

```rust,ignore
// We must bring this trait in scope in order to use mul_add()
use std::simd::StdFloat;

// Compute the diffusion gradient for U and V
let [full_u, full_v] = (win_u.rows().into_iter())
    .zip(win_v.rows())
    .zip(STENCIL_WEIGHTS)
    .flat_map(|((u_row, v_row), weights_row)| {
        (u_row.into_iter().copied())
            .zip(v_row.into_iter().copied())
            .zip(weights_row)
    })
    .fold(
        [Vector::splat(0.); 2],
        |[acc_u, acc_v], ((stencil_u, stencil_v), weight)| {
            let weight = Vector::splat(weight);
            [
                // NEW: Introduced FMA here
                (stencil_u - u).mul_add(weight, acc_u),
                (stencil_v - v).mul_add(weight, acc_v),
            ]
        },
    );

// Compute SIMD versions of all the float constants that we use
let diffusion_rate_u = Vector::splat(DIFFUSION_RATE_U);
let diffusion_rate_v = Vector::splat(DIFFUSION_RATE_V);
let feedrate = Vector::splat(opts.feedrate);
let killrate = Vector::splat(opts.killrate);
let deltat = Vector::splat(opts.deltat);
let ones = Vector::splat(1.0);

// Compute the output values of u and v
// NEW: Introduced even more FMA there
let uv_square = u * v * v;
let du = diffusion_rate_u.mul_add(full_u, (ones - u).mul_add(feedrate, -uv_square));
let dv = diffusion_rate_v.mul_add(full_v, -(feedrate + killrate).mul_add(v, uv_square));
*out_u = du.mul_add(deltat, u);
*out_v = dv.mul_add(deltat, v);
```

...and then run our microbenchmarks, full of hope...

```text
simulate/16x16          time:   [8.3555 µs 8.3567 µs 8.3580 µs]
                        thrpt:  [30.629 Melem/s 30.634 Melem/s 30.638 Melem/s]
                 change:
                        time:   [+731.23% +731.59% +731.89%] (p = 0.00 < 0.05)
                        thrpt:  [-87.979% -87.975% -87.970%]
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
simulate/64x64          time:   [68.137 µs 68.167 µs 68.212 µs]
                        thrpt:  [60.048 Melem/s 60.088 Melem/s 60.114 Melem/s]
                 change:
                        time:   [+419.07% +419.96% +421.09%] (p = 0.00 < 0.05)
                        thrpt:  [-80.809% -80.768% -80.735%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking simulate/256x256: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.7s, enable flat sampling, or reduce sample count to 50.
simulate/256x256        time:   [891.62 µs 891.76 µs 891.93 µs]
                        thrpt:  [73.476 Melem/s 73.491 Melem/s 73.502 Melem/s]
                 change:
                        time:   [+327.65% +329.35% +332.59%] (p = 0.00 < 0.05)
                        thrpt:  [-76.883% -76.709% -76.616%]
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) high mild
  8 (8.00%) high severe
simulate/1024x1024      time:   [10.769 ms 11.122 ms 11.489 ms]
                        thrpt:  [91.266 Melem/s 94.276 Melem/s 97.370 Melem/s]
                 change:
                        time:   [+34.918% +39.345% +44.512%] (p = 0.00 < 0.05)
                        thrpt:  [-30.802% -28.236% -25.881%]
                        Performance has regressed.
Benchmarking simulate/4096x4096: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.9s, or reduce sample count to 70.
simulate/4096x4096      time:   [71.169 ms 71.273 ms 71.376 ms]
                        thrpt:  [235.05 Melem/s 235.39 Melem/s 235.74 Melem/s]
                 change:
                        time:   [+0.1618% +0.4000% +0.6251%] (p = 0.00 < 0.05)
                        thrpt:  [-0.6213% -0.3984% -0.1616%] Change within noise
                        threshold.
```

...but ultimately, we get nothing but disappointment and perplexity. What's
going on, and why does this attempt at optimization slow everything down to a
crawl instead of speeding things up?


## The devil in the details

Software performance optimization is in many ways like natural sciences: when
experiment stubbornly refuses to agree with our theory, it is good to review our
theoretical assumptions.

When we say that modern x86 hardware can compute an FMA for the same cost as a
multiplication or an addition, what we actually mean is that hardware can
compute two FMAs per cycle, much like it can compute two additions per cycle[^1]
and two multiplications per cycle.

However, that statement comes with an "if" attached: our CPUs can only compute
two FMAs per cycle if there is sufficient instruction-level parallelism in the
code to keep the FMA units busy.

And it does not end there, those "if"s just keep piling up:

- FMAs have a higher latency than additions. Therefore it takes more
  instruction-level parallelism to hide that latency by excuting unrelated work
  while waiting for the results to come out. If you happen to be short on
  instruction-level parallelism, adding more FMAs will quickly make your program
  latency-bound, and thus slower.
- SIMD multiplication, addition and subtraction are quite flexible in terms of
  which registers or memory locations inputs can come from, which registers
  outputs can go to, and when negation can be applied. In contrast, because they
  have three operands, FMAs suffer from combinatorial explosion and end up with
  less supported patterns. It is therefore easier to end up in a situation where
  a single hardware FMA instruction cannot do the trick and you need more CPU
  instructions to do what looks like a single multiply + add/sub pattern in the
  code.

All this is to say, the hardware FMA implementations that we have today have
mainly been designed to make CPUs score higher at LINPACK benchmarking contests,
and achieved this goal with flying colors. As an unexpected bonus, it turns out
that they may also prove useful when implementing several other very regular
linear algebra and signal processing routines that do nothing but performing
lots of independent FMAs in rapid succession. But for any kind of computation
that exhibits a less trivial and regular pattern of floating point (add, mul)
pairs, it may take serious work from your side to achieve the promised 2x
speedup from the use of FMA instructions.


## Take two

Keeping the above considerations in mind, we will start by scaling back our FMA
ambitions, and only using FMAs in the initial part of the code that looks like a
dot product.

The rationale for this is that the rest of the code is largely latency-bound,
and irregular enough to hit FMA implementations corner cases. Therefore hardware
FMA is unlikely to help there.

```rust,ignore
// Compute the diffusion gradient for U and V
let [full_u, full_v] = (win_u.rows().into_iter())
    .zip(win_v.rows())
    .zip(STENCIL_WEIGHTS)
    .flat_map(|((u_row, v_row), weights_row)| {
        (u_row.into_iter().copied())
            .zip(v_row.into_iter().copied())
            .zip(weights_row)
    })
    .fold(
        [Vector::splat(0.); 2],
        |[acc_u, acc_v], ((stencil_u, stencil_v), weight)| {
            let weight = Vector::splat(weight);
            [
                // NEW: We keep the FMAs here...
                (stencil_u - u).mul_add(weight, acc_u),
                (stencil_v - v).mul_add(weight, acc_v),
            ]
        },
    );

// Compute SIMD versions of all the float constants that we use
let diffusion_rate_u = Vector::splat(DIFFUSION_RATE_U);
let diffusion_rate_v = Vector::splat(DIFFUSION_RATE_V);
let feedrate = Vector::splat(opts.feedrate);
let killrate = Vector::splat(opts.killrate);
let deltat = Vector::splat(opts.deltat);
let ones = Vector::splat(1.0);

// Compute the output values of u and v
// NEW: ...but we roll back the introduction of FMA there.
let uv_square = u * v * v;
let du = diffusion_rate_u * full_u - uv_square + feedrate * (ones - u);
let dv = diffusion_rate_v * full_v + uv_square - (feedrate + killrate) * v;
*out_u = u + du * deltat;
*out_v = v + dv * deltat;
```

If we benchmark this new implementation against the FMA-less implementation, we
get results that look a lot more reasonable already when compared to the
previous chapter's version:

```text
simulate/16x16          time:   [1.0638 µs 1.0640 µs 1.0642 µs]
                        thrpt:  [240.55 Melem/s 240.60 Melem/s 240.65 Melem/s]
                 change:
                        time:   [+5.5246% +5.5631% +5.5999%] (p = 0.00 < 0.05)
                        thrpt:  [-5.3029% -5.2699% -5.2354%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
simulate/64x64          time:   [13.766 µs 13.769 µs 13.771 µs]
                        thrpt:  [297.43 Melem/s 297.48 Melem/s 297.54 Melem/s]
                 change:
                        time:   [+6.7803% +6.8418% +6.9153%] (p = 0.00 < 0.05)
                        thrpt:  [-6.4680% -6.4037% -6.3498%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
simulate/256x256        time:   [218.72 µs 218.75 µs 218.78 µs]
                        thrpt:  [299.55 Melem/s 299.60 Melem/s 299.63 Melem/s]
                 change:
                        time:   [+5.9635% +5.9902% +6.0154%] (p = 0.00 < 0.05)
                        thrpt:  [-5.6741% -5.6516% -5.6278%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
simulate/1024x1024      time:   [7.8361 ms 7.9408 ms 8.0456 ms]
                        thrpt:  [130.33 Melem/s 132.05 Melem/s 133.81 Melem/s]
                 change:
                        time:   [-0.3035% +1.6779% +3.6310%] (p = 0.09 > 0.05)
                        thrpt:  [-3.5037% -1.6502% +0.3044%]
                        No change in performance detected.
Benchmarking simulate/4096x4096: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.9s, or reduce sample count to 70.
simulate/4096x4096      time:   [70.898 ms 70.994 ms 71.088 ms]
                        thrpt:  [236.01 Melem/s 236.32 Melem/s 236.64 Melem/s]
                 change:
                        time:   [+2.5330% +2.7397% +2.9421%] (p = 0.00 < 0.05)
                        thrpt:  [-2.8580% -2.6667% -2.4704%] Performance has
                        regressed.
```

That's not a speedup yet, but at least it is not a massive slowdown anymore.


## Breaking the (dependency) chain

The reason why our diffusion gradient computation did not get faster with FMAs
is that it features two long addition dependency chains, one for `acc_u` values
and one for `acc_v` values.

At each step of the original `fold()` reduction loop, if we consider the
computation of `acc_u`...

- There's one `stencil_u - u` that can be computed as soon as input `stencil_u`
  is available.[^2]
- Then one multiplication by `weight` which can only be done once that
  subtraction is done.
- And there is one final accumulation which will need to wait until the result
  of the multiplication is available AND the result of the previous `acc_u`
  computation is ready.

...and `acc_v` plays a perfectly symmetrical role. But after introducing FMA, we
get this:

- As before, there's one `stencil_u - u` that can be computed as soon as input is
  available.
- And then there is one FMA operation that needs both the result of that
  subtraction and the result of the previous `acc_u` computation.

Overall, we execute less CPU instructions per loop iteration. But we also
lengthened the dependency chain for `acc_u` because the FMA that computes it has
higher latency. Ultimately, the two effects almost cancel each other out, and we
get performance that is close, but slightly worse.

---

Can we break this `acc_u` dependency chain and speed things up by introducing
extra instruction-level parallelism, as we did before when summing
floating-point numbers? We sure can try!

However, it is important to realize that we must do it with care, because
introducing more ILP increases our code's SIMD register consumption, and we're
already putting the 16 available x86 SIMD registers under quite a bit of
pressure.

Indeed, in an ideal world, any quantity that remains constant across update loop
iterations would remain resident in a CPU register. But in our current code,
this includes...

- All the useful stencil element values (which, after compiler optimizer
  deduplication and removal of zero computations, yields one register full of
  copies of the `0.5` constant and another full of copies of the `0.25`
  constant).
- Splatted versions of all 5 simulation parameters `diffusion_rate_u`,
  `diffusion_rate_v`, `feedrate`, `killrate` and `deltat`.
- One register full of `1.0`s, matching our variable `ones`.
- And most likely one register full of `0.0`, because one of these almost always
  comes up in SIMD computations. To the point where many non-x86 CPUs optimize
  for it by providing a fake register from which reads always return 0.

This means that in the ideal world where all constants can be kept resident in
registers, we already have 9 SIMD registers eaten up by those registers, and
only have 7 registers left for the computation proper. In addition, for each
loop iteration, we also need at a bare minimum...

- 2 registers for holding the center values of u and v.
- 2 registers for successively holding `stencil_u`, `stencil_u - u` and
  `(stencil_u - u) * weight` before accumulation, along with the v equivalent.
- 2 registers for holding the current values of `acc_u` and `acc_v`.

So all in all, we only have 1 CPU register left for nontrivial purposes before
we start spilling our constants back to memory. Which means that introducing
significant ILP (in the form of two new accumulators and input values) will
necessarily come at the expense at spilling constants to memory and needing to
reload them from memory later on. The question then becomes: are the benefits
of ILP worth this expense?


## Experiment time

Well, there is only one way to find out. First we try to introduce two-way
ILP[^3]...

```rust,ignore
// Compute SIMD versions of the stencil weights
let stencil_weights = STENCIL_WEIGHTS.map(|row| row.map(Vector::splat));

// Compute the diffusion gradient for U and V
let mut full_u_1 = (win_u[[0, 0]] - u) * stencil_weights[0][0];
let mut full_v_1 = (win_v[[0, 0]] - v) * stencil_weights[0][0];
let mut full_u_2 = (win_u[[0, 1]] - u) * stencil_weights[0][1];
let mut full_v_2 = (win_v[[0, 1]] - v) * stencil_weights[0][1];
full_u_1 = (win_u[[0, 2]] - u).mul_add(stencil_weights[0][2], full_u_1);
full_v_1 = (win_v[[0, 2]] - v).mul_add(stencil_weights[0][2], full_v_1);
full_u_2 = (win_u[[1, 0]] - u).mul_add(stencil_weights[1][0], full_u_2);
full_v_2 = (win_v[[1, 0]] - v).mul_add(stencil_weights[1][0], full_v_2);
assert_eq!(STENCIL_WEIGHTS[1][1], 0.0);  // <- Will be optimized out
full_u_1 = (win_u[[1, 2]] - u).mul_add(stencil_weights[1][2], full_u_1);
full_v_1 = (win_v[[1, 2]] - v).mul_add(stencil_weights[1][2], full_v_1);
full_u_2 = (win_u[[2, 0]] - u).mul_add(stencil_weights[2][0], full_u_2);
full_v_2 = (win_v[[2, 0]] - v).mul_add(stencil_weights[2][0], full_v_2);
full_u_1 = (win_u[[2, 1]] - u).mul_add(stencil_weights[2][1], full_u_1);
full_v_1 = (win_v[[2, 1]] - v).mul_add(stencil_weights[2][1], full_v_1);
full_u_2 = (win_u[[2, 2]] - u).mul_add(stencil_weights[2][2], full_u_2);
full_v_2 = (win_v[[2, 2]] - v).mul_add(stencil_weights[2][2], full_v_2);
let full_u = full_u_1 + full_u_2;
let full_v = full_v_1 + full_v_2;
```

...and we observe that it is indeed quite a significant benefit for this
computation:

```text
simulate/16x16          time:   [452.35 ns 452.69 ns 453.13 ns]
                        thrpt:  [564.96 Melem/s 565.51 Melem/s 565.94 Melem/s]
                 change:
                        time:   [-57.479% -57.442% -57.399%] (p = 0.00 < 0.05)
                        thrpt:  [+134.73% +134.97% +135.18%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe
simulate/64x64          time:   [4.0774 µs 4.0832 µs 4.0952 µs]
                        thrpt:  [1.0002 Gelem/s 1.0031 Gelem/s 1.0046 Gelem/s]
                 change:
                        time:   [-70.417% -70.388% -70.352%] (p = 0.00 < 0.05)
                        thrpt:  [+237.29% +237.70% +238.03%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
simulate/256x256        time:   [59.714 µs 59.746 µs 59.791 µs]
                        thrpt:  [1.0961 Gelem/s 1.0969 Gelem/s 1.0975 Gelem/s]
                 change:
                        time:   [-72.713% -72.703% -72.691%] (p = 0.00 < 0.05)
                        thrpt:  [+266.18% +266.35% +266.47%]
                        Performance has improved.
Found 16 outliers among 100 measurements (16.00%)
  4 (4.00%) low mild
  3 (3.00%) high mild
  9 (9.00%) high severe
simulate/1024x1024      time:   [7.0386 ms 7.1614 ms 7.2842 ms]
                        thrpt:  [143.95 Melem/s 146.42 Melem/s 148.98 Melem/s]
                 change:
                        time:   [-11.745% -9.8149% -7.9159%] (p = 0.00 < 0.05)
                        thrpt:  [+8.5963% +10.883% +13.308%]
                        Performance has improved.
simulate/4096x4096      time:   [37.029 ms 37.125 ms 37.219 ms]
                        thrpt:  [450.78 Melem/s 451.91 Melem/s 453.08 Melem/s]
                 change:
                        time:   [-47.861% -47.707% -47.557%] (p = 0.00 < 0.05)
                        thrpt:  [+90.684% +91.229% +91.797%]
                        Performance has improved.
```

Encouraged by this first success, we try to go for 4-way ILP...

```rust,ignore
let mut full_u_1 = (win_u[[0, 0]] - u) * stencil_weights[0][0];
let mut full_v_1 = (win_v[[0, 0]] - v) * stencil_weights[0][0];
let mut full_u_2 = (win_u[[0, 1]] - u) * stencil_weights[0][1];
let mut full_v_2 = (win_v[[0, 1]] - v) * stencil_weights[0][1];
let mut full_u_3 = (win_u[[0, 2]] - u) * stencil_weights[0][2];
let mut full_v_3 = (win_v[[0, 2]] - v) * stencil_weights[0][2];
let mut full_u_4 = (win_u[[1, 0]] - u) * stencil_weights[1][0];
let mut full_v_4 = (win_v[[1, 0]] - v) * stencil_weights[1][0];
assert_eq!(STENCIL_WEIGHTS[1][1], 0.0);
full_u_1 = (win_u[[1, 2]] - u).mul_add(stencil_weights[1][2], full_u_1);
full_v_1 = (win_v[[1, 2]] - v).mul_add(stencil_weights[1][2], full_v_1);
full_u_2 = (win_u[[2, 0]] - u).mul_add(stencil_weights[2][0], full_u_2);
full_v_2 = (win_v[[2, 0]] - v).mul_add(stencil_weights[2][0], full_v_2);
full_u_3 = (win_u[[2, 1]] - u).mul_add(stencil_weights[2][1], full_u_3);
full_v_3 = (win_v[[2, 1]] - v).mul_add(stencil_weights[2][1], full_v_3);
full_u_4 = (win_u[[2, 2]] - u).mul_add(stencil_weights[2][2], full_u_4);
full_v_4 = (win_v[[2, 2]] - v).mul_add(stencil_weights[2][2], full_v_4);
let full_u = (full_u_1 + full_u_2) + (full_u_3 + full_u_4);
let full_v = (full_v_1 + full_v_2) + (full_v_3 + full_v_4);
```

...and it does not change much anymore on the Intel i9-10900 CPU that I'm
testing on here, although I can tell you that it still gives ~10% speedups on my
AMD Zen 3 CPUs at home.[^4]

```text
simulate/16x16          time:   [450.29 ns 450.40 ns 450.58 ns]
                        thrpt:  [568.16 Melem/s 568.38 Melem/s 568.52 Melem/s]
                 change:
                        time:   [-0.6248% -0.4375% -0.2013%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2017% +0.4394% +0.6287%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
simulate/64x64          time:   [4.0228 µs 4.0230 µs 4.0231 µs]
                        thrpt:  [1.0181 Gelem/s 1.0182 Gelem/s 1.0182 Gelem/s]
                 change:
                        time:   [-1.4861% -1.3761% -1.3101%] (p = 0.00 < 0.05)
                        thrpt:  [+1.3275% +1.3953% +1.5085%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
simulate/256x256        time:   [60.567 µs 60.574 µs 60.586 µs]
                        thrpt:  [1.0817 Gelem/s 1.0819 Gelem/s 1.0820 Gelem/s]
                 change:
                        time:   [+1.3650% +1.4165% +1.4534%] (p = 0.00 < 0.05)
                        thrpt:  [-1.4326% -1.3967% -1.3467%]
                        Performance has regressed.
Found 15 outliers among 100 measurements (15.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  10 (10.00%) high severe
simulate/1024x1024      time:   [6.8775 ms 6.9998 ms 7.1208 ms]
                        thrpt:  [147.26 Melem/s 149.80 Melem/s 152.47 Melem/s]
                 change:
                        time:   [-4.5709% -2.2561% +0.0922%] (p = 0.07 > 0.05)
                        thrpt:  [-0.0921% +2.3082% +4.7898%]
                        No change in performance detected.
simulate/4096x4096      time:   [36.398 ms 36.517 ms 36.641 ms]
                        thrpt:  [457.88 Melem/s 459.44 Melem/s 460.94 Melem/s]
                 change:
                        time:   [-2.0551% -1.6388% -1.2452%] (p = 0.00 < 0.05)
                        thrpt:  [+1.2609% +1.6661% +2.0982%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
```

Finally, we can try to implement maximal 8-way ILP...

```rust,ignore
let full_u_1 = (win_u[[0, 0]] - u) * stencil_weights[0][0];
let full_v_1 = (win_v[[0, 0]] - v) * stencil_weights[0][0];
let full_u_2 = (win_u[[0, 1]] - u) * stencil_weights[0][1];
let full_v_2 = (win_v[[0, 1]] - v) * stencil_weights[0][1];
let full_u_3 = (win_u[[0, 2]] - u) * stencil_weights[0][2];
let full_v_3 = (win_v[[0, 2]] - v) * stencil_weights[0][2];
let full_u_4 = (win_u[[1, 0]] - u) * stencil_weights[1][0];
let full_v_4 = (win_v[[1, 0]] - v) * stencil_weights[1][0];
assert_eq!(STENCIL_WEIGHTS[1][1], 0.0);
let full_u_5 = (win_u[[1, 2]] - u) * stencil_weights[1][2];
let full_v_5 = (win_v[[1, 2]] - v) * stencil_weights[1][2];
let full_u_6 = (win_u[[2, 0]] - u) * stencil_weights[2][0];
let full_v_6 = (win_v[[2, 0]] - v) * stencil_weights[2][0];
let full_u_7 = (win_u[[2, 1]] - u) * stencil_weights[2][1];
let full_v_7 = (win_v[[2, 1]] - v) * stencil_weights[2][1];
let full_u_8 = (win_u[[2, 2]] - u) * stencil_weights[2][2];
let full_v_8 = (win_v[[2, 2]] - v) * stencil_weights[2][2];
let full_u = ((full_u_1 + full_u_2) + (full_u_3 + full_u_4))
    + ((full_u_5 + full_u_6) + (full_u_7 + full_u_8));
let full_v = ((full_v_1 + full_v_2) + (full_v_3 + full_v_4))
    + ((full_v_5 + full_v_6) + (full_v_7 + full_v_8));
```

...and it is undisputably slower:

```text
simulate/16x16          time:   [486.03 ns 486.09 ns 486.16 ns]
                        thrpt:  [526.58 Melem/s 526.65 Melem/s 526.72 Melem/s]
                 change:
                        time:   [+7.6024% +7.8488% +7.9938%] (p = 0.00 < 0.05)
                        thrpt:  [-7.4021% -7.2776% -7.0653%]
                        Performance has regressed.
Found 18 outliers among 100 measurements (18.00%)
  16 (16.00%) high mild
  2 (2.00%) high severe
simulate/64x64          time:   [4.6472 µs 4.6493 µs 4.6519 µs]
                        thrpt:  [880.51 Melem/s 881.00 Melem/s 881.38 Melem/s]
                 change:
                        time:   [+15.496% +15.546% +15.598%] (p = 0.00 < 0.05)
                        thrpt:  [-13.494% -13.454% -13.417%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
simulate/256x256        time:   [68.774 µs 68.923 µs 69.098 µs]
                        thrpt:  [948.45 Melem/s 950.86 Melem/s 952.91 Melem/s]
                 change:
                        time:   [+13.449% +13.563% +13.710%] (p = 0.00 < 0.05)
                        thrpt:  [-12.057% -11.943% -11.854%]
                        Performance has regressed.
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  16 (16.00%) high severe
simulate/1024x1024      time:   [7.0141 ms 7.1438 ms 7.2741 ms]
                        thrpt:  [144.15 Melem/s 146.78 Melem/s 149.50 Melem/s]
                 change:
                        time:   [-0.2910% +2.0563% +4.6567%] (p = 0.12 > 0.05)
                        thrpt:  [-4.4495% -2.0149% +0.2918%]
                        No change in performance detected.
simulate/4096x4096      time:   [38.128 ms 38.543 ms 38.981 ms]
                        thrpt:  [430.39 Melem/s 435.29 Melem/s 440.03 Melem/s]
                 change:
                        time:   [+4.2543% +5.5486% +6.7916%] (p = 0.00 < 0.05)
                        thrpt:  [-6.3597% -5.2569% -4.0807%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
```

This should not surprise you: at this point, we have fully lost the performance
benefits of fused multiply-add, and we use so many registers for our inputs and
accumulators that the compiler will need to generate a great number of constant
spills and reloads in order to fit the computation into the 16
microarchitectural x86 SIMD registers.

All in all, for this computation, the 4-way ILP version can be declared our
performance winner.


## Exercise

The astute reader will have noticed that we cannot compare between FMA and
multiply-add sequences yet because we have only implemented ILP optimizations in
the FMA-based code, not the original code based on non fused multiply-add
sequences.

Please assess the true performance benefits of FMAs on this computation by
starting from the 4-way ILP FMA version and replacing all `mul_add`s with
multiply-add sequences, then comparing the benchmark results in both cases.

Find the results interesting? We will get back to them in the next chapter.


---

[^1]: Yes, I know about those newer Intel CPU cores that have a third adder.
      Thanks Intel, that was a great move! Now wake me up when these chips are
      anywhere near widespread in computing centers.

[^2]: The astute reader will have noticed that we can actually get rid of this
      subtraction by changing the center weight of the stencil. As this
      optimization does not involve any new Rust or HPC concept, it is not super 
      interesting in the context of this course, so in the interest of time we
      leave it as an exercise to the reader. But if you're feeling lazy and
      prefer to read code than write it, it has been applied to [the reference
      implementation](https://github.com/HadrienG2/grayscott).

[^3]: Must use the traditional duplicated code ILP style here because
      `iterator_ilp` cannot implement the optimization of ignoring the zero
      stencil value at the center, which is critical to performance here.

[^4]: This is not surprising because AMD Zen CPUs have more independent
      floating-point ALUs than most Intel CPUs, and thus it takes more
      instruction-level parallelism to saturate their execution backend.
