# Regularizing

If you ran a profiler on the initial code that you were provided with, you would
find that it spends an unacceptable amount of time computing which indices of
the concentration tables should be targeted by the input stencil, and slicing
the input tables at these locations.

In other words, this part of the code is the bottleneck:

```rust,ignore
// Determine the stencil's input region
let out_pos = [out_row, out_col];
let stencil_start = array2(|i| out_pos[i].saturating_sub(STENCIL_OFFSET[i]));
let stencil_end = array2(|i| (out_pos[i] + STENCIL_OFFSET[i] + 1).min(shape[i]));
let stencil_range = array2(|i| stencil_start[i]..stencil_end[i]);
let stencil_slice = ndarray::s![stencil_range[0].clone(), stencil_range[1].clone()];

// Compute the diffusion gradient for U and V
let [full_u, full_v] = (start.u.slice(stencil_slice).indexed_iter())
    .zip(start.v.slice(stencil_slice))
    .fold(/* ... proceed with the computation ... */)
```

We have seen, however, that `ndarray` provides us with an optimized sliding
window iterator called `windows()`. One obvious next steps would be to use this
iterator instead of doing all the indexing ourselves. This is not as easy as it
seems, but getting there will be the purpose of this chapter.


## The boundary condition problem

Our Gray-Scott reaction simulation is a member of a larger family or numerical
computations called stencil computations. What these computations all have in
common is that their output at one particular spatial location depends on a
weighted average of the neighbours of this spatial location in the input table.
And therefore, all stencil computations must address one common concern: what
should be done when there is no neighbour, on the edges or corners of the
simulation domain?

In this course, we use a zero boundary condition. That is to say, we extend the
simulation domain by saying that if we need to read a chemical species'
concentration outside of the simulation domain, the read will always return
zero. And the way we implement this policy in code is that we do not do the
computation at all for these stencil elements. This works because multiplying
one of the stencil weights by zero will return zero, and therefore the
associated contribution to the final weighted sum will be zero, as if the
associated stencil elements were not taken into account to begin with.

Handling missing values like this is a common choice, and there is nothing wrong
with it per se. However, it means that we cannot simply switch to `ndarray`'s
windows iterator by changing our simulation update loop into something like this:

```rust,ignore
ndarray::azip!(
    (
        out_u in &mut end.u,
        out_v in &mut end.v,
        win_u in start.u.windows([3, 3]),
        win_v in start.v.windows([3, 3]),
    ) {
        // TODO: Adjust the rest of the computation to work with these inputs
    }
);
```

The reason why it does not work is that for a 2D array of dimensions NxM,
iterating over output elements will produce all NxM elements, whereas iterating
over 3x3 windows will only produce (N-2)x(M-2) valid input windows. Therefore,
the above computation loop is meaningless and if you try to work with it anyway,
you will inevitably produce incorrect results.

There are two classic ways to resolve this issue:

- We can make our data layout more complicated by resizing the concentration
  tables to add a strip of zeroes all around the actually useful data, and be
  careful never to touch these zeroes so that they keep being zeros.
- We can make our update loop more complcated by splitting it into two parts,
  one which processes the center of the simulation domain with optimal
  efficiency, and one which processes the edge at a reduced efficiency.

Both approaches have their merits, and at nontrivial problem size they have
equivalent performance. The first approach makes the update loop simpler, but
the second approach avoids polluting the rest of the codebase[^1] with edge
element handling concerns. Knowing this, it is up to you to choose where you
should spend your code complexity budget.


## Optimizing the central iterations

The `azip` loop above was actually _almost_ right for computing the central
concentration values. It only takes a little bit of extra slicing to make it
correct:

```rust,ignore
let shape = start.shape();
let center = ndarray::s![1..shape[0]-1, 1..shape[1]-1];
ndarray::azip!(
    (
        out_u in end.u.slice_mut(center),
        out_v in end.v.slice_mut(center),
        win_u in start.u.windows([3, 3]),
        win_v in start.v.windows([3, 3]),
    ) {
        // TODO: Adjust the rest of the computation to work with these inputs
    }
);
```

With this change, we now know that the computation will always work on 3x3 input
windows, and therefore we can dramatically simplify the per-iteration code:

```rust,ignore
let shape = start.shape();
let center = s![1..shape[0]-1, 1..shape[1]-1];
ndarray::azip!(
    (
        out_u in end.u.slice_mut(center),
        out_v in end.v.slice_mut(center),
        win_u in start.u.windows([3, 3]),
        win_v in start.v.windows([3, 3]),
    ) {
        // Get the center concentration
        let u = win_u[STENCIL_OFFSET];
        let v = win_v[STENCIL_OFFSET];

        // Compute the diffusion gradient for U and V
        let [full_u, full_v] = (win_u.into_iter())
            .zip(win_v)
            .zip(STENCIL_WEIGHTS.into_iter().flatten())
            .fold(
                [0.; 2],
                |[acc_u, acc_v], ((&stencil_u, &stencil_v), weight)| {
                    [acc_u + weight * (stencil_u - u), acc_v + weight * (stencil_v - v)]
                },
            );

        // Rest of the computation is unchanged
        let uv_square = u * v * v;
        let du = DIFFUSION_RATE_U * full_u - uv_square + opts.feedrate * (1.0 - u);
        let dv = DIFFUSION_RATE_V * full_v + uv_square
            - (opts.feedrate + opts.killrate) * v;
        *out_u = u + du * opts.deltat;
        *out_v = v + dv * opts.deltat;
    }
);
```

You will probably not be surprised to learn that in addition to being much
easier to read and maintain, this Rust code will also compile down to
much faster machine code.

But of course, it does not fully resolve the problem at hand, as we are not
computing the edge values of the chemical species concentration correctly. We
are going to need either a separate code paths or a data layout change to get
there.


## Exercise

For this exercise, we give you two possible strategies:

1. Write separate code to handle the boundary values using the logic of the
   initial naive code. If you choose this path, keep around the initial stencil
   update loop in addition to the regularized loop above, you will need it to
   handle the edge values.
2. Change the code that allocates the simulation's data storage and writes
   output down to HDF5 in order to allocate one extra element on each side of
   the concentration arrays. Keep these elements equal to zero throughout the
   computation, and use a `center` slice analogous to the one above in order to
   only emit results in the relevant region of the concentration array.

If you are undecided, I would advise going for the first option, as the
resulting regular/irregular code split will give you an early taste of things to
come in the next chapter.


---

[^1]: Including, in larger numerical codebases, code that you may have little
      control over.
