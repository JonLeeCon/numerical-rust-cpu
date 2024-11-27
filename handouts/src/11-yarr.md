# `ndarray`

At this point, you should know enough about numerical computing in Rust to
efficiently implement any classic one-dimensional computation, from FFT to BLAS
level 1 vector operations.

Sadly, this is not enough, because the real world insists on solving
multi-dimensional mathematical problems[^1]. And therefore you will need to
learn what it takes to make these efficient in Rust too.

The first step to get there is to have a way to actually get multi-dimensional
arrays into your code. Which will be the topic of this chapter.


## Motivation

Multi-dimensional array libraries are actually not a strict requirement for
numerical computing. If programming like a BASIC programmer from the 80s is your
kink, you can just treat any 1D array as a multidimensional array by using the
venerable index linearization trick:

```rust
let array = [00, 01, 02, 03, 04,
             10, 11, 12, 13, 14,
             20, 21, 22, 23, 24,
             30, 31, 32, 33, 34];
let row_length = 5;

let index_2d = [2, 1];
let index_linear = row_length * index_2d[0] + index_2d[1];

assert_eq!(array[index_linear], 21);
```

There are only two major problems with this way of doing things:

- Writing non-toy code using this technique is highly error-prone. You **will**
  spend many hours debugging why a wrong result is returned, only to find out
  much later that you have swapped the row and column index, or did not keep the
  row length metadata in sync with changes to the contents of the underlying
  array.
- It is nearly impossible to access manually implemented multidimensional arrays
  without relying on lots of manual array/`Vec` indexing. The correctness and
  runtime performance issues of these indexing operations in Rust should be
  familiar to you by now. So if we can, it would be great to get rid of them in
  our code.

The `ndarray` crate resolves these problems by implementing all the tricky index
linearization code for you, and exposing it using types that act like a
multidimensional version of Rust's `Vec`s and slices. These types keep data and
shape metadata in sync for you, and come with multidimensional versions of all
the utility methods that you are used to coming from standard library `Vec` and
slices, including multidimensional overlapping window iterators.

`ndarray` is definitely not the only library doing this in the Rust ecosystem,
but it is one of the most popular ones. This means that you will find plenty of
other libraries that build on top of it, like linear algebra and statistics
libraries. Also, of all available options, it is in my opinion currently the one
that provides the best tradeoff between ease of use, runtime performance,
generality and expressive power. This makes it an excellent choice for
computations that go a bit outside of the linear algebra textbook vocabulary,
including the Gray-Scott reaction simulation that we will implement next.


## Adding `ndarray` to your project

As usual, adding a new dependency is as easy as `cargo add`:

```bash
cargo add ndarray
```

But this time, you may want to look into the list of optional features that gets
displayed in your console when you run this command:

```text
$ cargo add ndarray
    Updating crates.io index
      Adding ndarray v0.15.6 to dependencies
             Features:
             + std
             - approx
             - approx-0_5
             - blas
             - cblas-sys
             - docs
             - libc
             - matrixmultiply-threading
             - rayon
             - rayon_
             - serde
             - serde-1
             - test
```

By default, `ndarray` keeps the set of enabled features small in order to speed
up compilation. But if you want to do more advanced numerics with `ndarray` in
the future, some of the optional functionality, like integration with the system
BLAS, or parallel Rayon iterators over the contents of multidimensional arrays
may come very handy. We will, however, not need them for this particular course,
so let's move on.


## Creating `Array`s

The heart of `ndarray` is
[`ArrayBase`](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html), a
very generic multidimensional array type that can either own its data (like
`Vec`) or borrow it from a Rust slice that you obtained from some other source.

While you are learning Rust, you will likely find it easier to avoid using this
very general-purpose generic type directly, and instead work with the various
type aliases that are provided by the `ndarray` crate for easier operation. For
our purposes, three especially useful aliases will be...

- [`Array2`](https://docs.rs/ndarray/latest/ndarray/type.Array2.html), which
  represents an owned two-dimensional array of data (similar to `Vec<T>`)
- [`ArrayView2`](https://docs.rs/ndarray/latest/ndarray/type.ArrayView2.html),
  which represents a shared 2D slice of data (similar to `&[T]`)
- [`ArrayViewMut2`](https://docs.rs/ndarray/latest/ndarray/type.ArrayViewMut2.html),
  which represents a mutable 2D slice of data (similar to `&mut [T]`)

But because these are just type aliases, the documentation page for them will
not tell you about all available methods. So you will still want to keep the
[`ArrayBase` documentation
](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html) close by.

---

Another useful tool in ndarray is the `array!` macro, which lets you create
`Array`s using a syntax analogous to that of the `vec![]` macro:

```rust,ignore
use ndarray::array;

let a2 = array![[01, 02, 03],
                [11, 12, 13]];
```

You can also create `Array`s from a function that maps each index to a
corresponding data element...

```rust,ignore
use ndarray::{Array, arr2};

// Create a table of i × j (with i and j from 1 to 3)
let ij_table = Array::from_shape_fn((3, 3), |(i, j)| (1 + i) * (1 + j));

assert_eq!(
    ij_table,
    // You can also create an array from a slice of Rust arrays
    arr2(&[[1, 2, 3],
           [2, 4, 6],
           [3, 6, 9]])
);
```

...and if you have a `Vec` of data around, you can [turn it into an `Array` as
well](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.from_shape_vec).
But because a `Vec` does not come with multidimensional shape information, you
will need to provide this information separately, at the risk of it going out of
sync with the source data. And you will also need to pay close attention to the
the order in which `Array` elements should be provided by the input `Vec`
(`ndarray` uses row-major order by default).

So all in all, in order to avoid bugs, it is best to avoid these conversions and
stick with the ndarray APIs whenever you can.


## Iteration

Rust iterators were designed for one-dimensional data, and using them for
multidimensional data comes at the cost of losing useful information. For
example, they cannot express concepts like "here is a block of data that is
contiguous in memory, but then there is a gap" or "I would like the iterator to
skip to the next row of my matrix here, and then resume iteration".

For this reason, `ndarray` does not directly use standard Rust iterators.
Instead, it uses a homegrown abstraction called
[`NdProducer`](https://docs.rs/ndarray/latest/ndarray/trait.NdProducer.html),
which can be lossily converted to a standard iterator.

We will not be leveraging the specifics of `ndarray` producers much in this
course, as standard iterators are enough for what we are doing. But I am telling
you about this because it explains why iterating over `Arrays` may involve a
different API than iterating over a standard Rust collection, or require an
extra `into_iter()` producer-to-iterator conversion step.

In simple cases, however, the conversion will just be done automatically. For
example, here is how one would iterate over 3x3 overlapping windows of an
`Array2` using `ndarray`:

```rust,ignore
use ndarray::{array, azip};

let arr = array![[01, 02, 03, 04, 05, 06, 07, 08, 09],
                 [11, 12, 13, 14, 15, 16, 17, 18, 19],
                 [21, 22, 23, 24, 25, 26, 27, 28, 29],
                 [31, 32, 33, 34, 35, 36, 37, 38, 39],
                 [41, 42, 43, 44, 45, 46, 47, 48, 49]];

for win in arr.windows([3, 3]) {
    println!("{win:?}");
}
```

`ndarray` comes with a large set of producers, some of which are more
specialized and optimized than others. It is therefore a good idea to spend some
time looking through the various options that can be used to solve a particular
problem, rather than picking the first producer that works.


## Indexing and slicing

Like `Vec`s and slices, `Array`s and `ArrayView`s support indexing and slicing.
And as with `Vec`s and slices, it is generally a good idea to avoid using these
operations for performance and correctness reasons, especially inside of
performance-critical loops.

Indexing works using square brackets as usual, the only new thing being that you
can pass in an array or a tuple of indices instead of just one index:

```rust,ignore
use ndarray::Array2;
let mut array = Array2::zeros((4, 3));
array[[1, 1]] = 7;
```

However, a design oversight in the Rust indexing operator means that it cannot
be used for slicing multidimensional arrays. Instead, you will need the
following somewhat unclean syntax:

```rust,ignore
use ndarray::s;
let b = a.slice(s![.., 0..1, ..]);
```

Notice the use of an `s![]` macro for constructing a slicing configuration
object, which is then passed to a `slice()` method of the generic `ArrayView`
object.

This is just one of many slicing methods. Among others, we also get...

- [`slice()`](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.slice),
  which creates `ArrayView`s (analogous to `&vec[start..finish]`)
- [`slice_mut()`](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.slice_mut),
  which creates `ArrayViewMut`s (analogous to `&mut vec[start..finish]`)
- [`multi_slice_mut()`](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.multi_slice_mut),
  which creates several non-overlapping `ArrayViewMut`s in a single transaction.
  This can be used to work around Rust's "single mutable borrow" rule when
  it proves to be an unnecessary annoyance.
- [`slice_move()`](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.slice_move),
  which consumes the input array or view and returns an owned slice.


## Exercise

All `ndarray` types natively support a `sum()` operation. Compare its
performance to that or your optimized floating-point sum implementation over a
wide range of input sizes.

One thing which will make your life easier is that
[`Array1`](https://docs.rs/ndarray/latest/ndarray/type.Array1.html), the owned
one-dimensional array type, can be built from a standard iterator using the
[`Array::from_iter()`
constructor](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.from_iter).


---

[^1]: ...which are a poor fit for the one-dimensional memory architecture of
      standard computers, and this causes a nearly infinite amount of fun
      problems that we will not all cover during this course.
