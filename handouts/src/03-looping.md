# Loops and arrays

As a result of this course being time-constrained, we do not have the luxury of
deep-diving into Rust possibilities like a full Rust course would. Instead, we
will be focusing on the minimal set of building blocks that you need in order to
do numerical computing in Rust.

We've covered variables and basic debugging tools in the first chapter, and
we've covered integer and floating-point arithmetic in the second chapter. Now
it's time for the last major language-level component of numerical computations:
loops, arrays, and other iterable constructs.


## Range-based loop

The basic syntax for looping over a range of integers is simple enough:

```rust
for i in 0..10 {
    println!("i = {i}");
}
```

Following an old tradition, ranges based on the `..` syntax are left inclusive
and right exclusive, i.e. the left element is included, but the right element is
not included. The reasons why this is a good default have been [explained at
length
elsewhere](https://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD831.html),
so we will not repeat them here.

However, Rust acknowledges that ranges that are inclusive on both sides also
have their uses, and therefore they are available through a slightly more
verbose syntax:

```rust
println!("Fortran and Julia fans, rejoice!");
for i in 1..=10 {
    println!("i = {i}");
}
```

The Rust range types are actually used for more than iteration. They accept
non-integer bounds, and they provide a `contains()` method to check that a value
is contained within a range. And all combinations of inclusive, exclusive, and
infinite bounds are supported by the language, even though not all of them can
be used for iteration:

- The `..` infinite range contains all elements in some ordered set
- `x..` ranges start at a certain value and contain all subsequent values in the
  set
- `..y` and `..=y` ranges start at the smallest value of the set and contain all
  values up to an exclusive or inclusive upper bound
- The [`Bound`](https://doc.rust-lang.org/std/ops/enum.Bound.html) standard
  library type can be used to cover all other combinations of inclusive,
  exclusive, and infinite bounds, via `(Bound, Bound)` tuples


## Iterators

Under the hood, the Rust `for` loop has no special support for ranges of
integers. Instead, it operates over a pair of lower-level standard library
primitives called
[`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) and
[`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html).
These can be described as follows:

- A type that implements the `Iterator` trait provides a `next()` method, which
  produces a value and internally modifies the iterator object so that a
  different value will be produced when the `next()` method is called again.
  After a while, a special `None` value is produced, indicating that all
  available values have been produced, and the iterator should not be used
  again.
- A type that implements the `IntoIterator` trait "contains" one or more values,
  and provides an `into_iter()` method which can be used to create an `Iterator`
  that yields those inner values.

The for loop uses these mechanisms as follows:

```rust
# fn do_something(i: i32) {}
#
// A for loop like this...
for i in 0..3 {
    do_something(i);
}

// ...is effectively translated into this during compilation:
let mut iterator = (0..3).into_iter();
while let Some(i) = iterator.next() {
    do_something(i);
}
```

Readers familiar with C++ will notice that this is somewhat similar to STL
iterators and C++11 range-base for loops, but with a major difference: unlike
Rust iterators, C++ iterators have no knowledge of the end of the underlying
data stream. That information must be queried separately, carried around
throughout the code, and if you fail to handle it correctly, undefined behavior
will ensue.

This difference comes at a major usability cost, to the point where after much
debate, 5 years after the release of the first stable Rust version, the C++20
standard revision has finally decided to soft-deprecate standard C++ iterators
in favor of a Rust-like iterator abstraction, confusingly calling it a "range"
because the "iterator" name was already taken.[^1]

Another advantage of the Rust iterator model is that because Rust iterator
objects are self-sufficient, they can implement methods that transform an
iterator object in various ways. The Rust `Iterator` trait _heavily_ leverages
this possibility, providing [dozens of
methods](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.count)
that are automatically implemented for every standard and user-defined iterator
type, even though the default implementations can be overriden for performance.

Most of these methods consume the input iterator and produce a different
iterator as an output. These methods are commonly called "adapters". Here is an
example of one of them in action:

```rust
// Turn an integer range into an iterator, then transform the iterator so that
// it only yields 1 element for 10 original elements.
for i in (0..100).into_iter().step_by(10) {
    println!("i = {i}");
}
```

One major property of these iterator adapters is that they operate lazily:
transformations are performed on the fly as new iterator elements are generated,
without needing to collect transformed data in intermediary collections. Because
compilers are bad at optimizing out memory allocations and data movement, this
way of operating is a lot better than generating temporary collections from a
performance point of view, to the point where code that uses iterator adapters
usually compiles down to the same assembly as an optimal hand-written while
loop.

For reasons that will be explained during the next parts of this course, usage
of iterator adapters is very common in idiomatic Rust code, and generally
preferred over equivalent imperative programming constructs unless the latter
provide a significant improvement in code readability.


## Arrays and `Vec`s

It is not just integer ranges that can be iterated over. Two other iterable Rust
objects of major interest to numerical computing are arrays and `Vec`s.

They are very similar to `std::array` and `std::vector` in C++:

- The storage for array variables is fully allocated on the stack.[^2] In
  contrast, the storage for a `Vec`'s data is allocated on the heap, using the
  Rust equivalent of `malloc()` and `free()`.
- The size of an array must be known at compile time and cannot change during
  runtime. In contrast, it is possible to add and remove elements to a `Vec`,
  and the underlying backing store will be automatically resized through memory
  reallocations and copies to accomodate this.
- It is often a bad idea to create and manipulate large arrays because they can
  overflow the program's stack (resulting in a crash) and are expensive to move
  around. In contrast, `Vec`s will easily scale as far as available RAM can take
  you, but they are more expensive to create and destroy, and accessing their
  contents may require an extra pointer indirection.
- Because of the compile-time size constraint, arrays are generally less
  ergonomic to manipulate than `Vec`s. Therefore `Vec` should be your first
  choice unless you have a good motivation for using arrays (typically heap
  allocation avoidance).

There are three basic ways to create a Rust array...

- Directly provide the value of each element: `[1, 2, 3, 4, 5]`.
- State that all elements have the same value, and how many elements there are:
  `[42; 6]` is the same as `[42, 42, 42, 42, 42, 42]`.
- Use the
  [`std::array::from_fn`](https://doc.rust-lang.org/std/array/fn.from_fn.html)
  standard library function to initialize each element based on its position
  within the array.

...and `Vec`s supports the first two initialization method via the `vec!` macro,
which uses the same syntax as array literals:

```rust
let v = vec![987; 12];
```

However, there is no equivalent of `std::array::from_fn` for `Vec`, as it is
replaced by the superior ability to construct `Vec`s from either iterators or
C++-style imperative code:

```rust
// The following three declarations are largely equivalent.

// Here, we need to tell the compiler that we're building a Vec, but we can let
// it infer the inner data type.
let v1: Vec<_> = (123..456).into_iter().collect();
let v2 = (123..456).into_iter().collect::<Vec<_>>();

let mut v3 = Vec::with_capacity(456 - 123 + 1);
for i in 123..456 {
    v3.push(i);
}

assert_eq!(v1, v2);
assert_eq!(v1, v3);
```

In the code above, the `Vec::with_capacity` constructor plays the same role as
the `reserve()` method of C++'s `std::vector`: it lets you tell the `Vec`
implementation how many elements you expect to `push()` upfront, so that said
implementation can allocate a buffer of the right length from the beginning and
thus avoid later reallocations and memory movement on `push()`.

And as hinted during the beginning of this section, both arrays and `Vec`s
implement `IntoIterator`, so you can iterate over their elements:

```rust
for elem in [1, 3, 5, 7] {
    println!("{elem}");
}
```


## Indexing

Following the design of most modern programming languages, Rust lets you access
array elements by passing a zero-based integer index in square brackets:

```rust
let arr = [9, 8, 5, 4];
assert_eq!(arr[2], 5);
```

However, unlike in C/++, accessing arrays at an invalid index does not result in
undefined behavior that gives the compiler license to arbitrarily trash your
program. Instead, the thread will just deterministically panic, which by default
will result in a well-controlled program crash.

Unfortunately, this memory safety does not come for free. The compiler has to
insert bounds-checking code, which may or may not later be removed by its
optimizer. When they are not optimized out, these bound checks tend to make
array indexing a fair bit more expensive from a performance point of view in
Rust than in C/++.

And this is actually one of the many reasons to prefer iteration over manual
array and Vec indexing in Rust. Because iterators access array elements using a
predictable and known-valid pattern, they can work without bound checks.
Therefore, they can be used to achieve C/++-like performance, without relying on
faillible compiler optimizations or `unsafe` code in your program.[^3] And
another major benefit is obviously that you cannot crash your program by using
iterators wrong.

But for those cases where you do need some manual indexing, you will likely
enjoy the `enumerate()` iterator adapter, which gives each iterator element an
integer index that starts at 0 and keeps growing. It is a very convenient tool
for bridging the iterator world with the manual indexing world:

```rust
// Later in the course, you will learn a better way of doing this
let v1 = vec![1, 2, 3, 4];
let v2 = vec![5, 6, 7, 8];
for (idx, elem) in v1.into_iter().enumerate() {
    println!("v1[{idx}] is {elem}");
    println!("v2[{idx}] is {}", v2[idx]);
}
```


## Slicing

Sometimes, you need to extract not just one array element, but a subset of array
elements. For example, in the Gray-Scott computation that we will be working on
later on in the course, you will need to work on sets of 3 consecutive elements
from an input array.

The simplest tool that Rust provides you to deal with this situation is slices,
which can be built using the following syntax:

```rust
let a = [1, 2, 3, 4, 5];
let s = &a[1..4];
assert_eq!(s, [2, 3, 4]);
```

Notice the leading `&`, which means that we take a reference to the original
data (we'll get back to what this means in a later chapter), and the use of
integer ranges to represent the set of array indices that we want to extract.

If this reminds you of C++20's `std::span`, this is no coincidence. Spans are
another instance of C++20 trying to catch up with features that Rust v1 had 5
years earlier...

Manual slice extraction comes with the same pitfalls as manual indexing (costly
bound checks, crash on error...), therefore Rust provides more efficient slice iterators. The most popular ones are...

- [`chunks()`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks)
  and
  [`chunk_exact()`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks_exact),
  which cut up your array/vec into a set of consecutive slices of a certain
  length and provide an iterator over these slices.
    * For example, `chunks(2)` would yield elements at indices `0..2`, `2..4`,
      `4..6`, etc.
    * They differ in how they handle the remaining elements of the array after
      all regularly-sized chunks have been taken care of. `chunks_exact()`
      compiles down to more efficient code by making you handle trailing
      elements using a separate code path.
- [`windows()`](https://doc.rust-lang.org/std/primitive.slice.html#method.windows),
  where the iterator yields overlapping slices, each shifted one array/vec
  element away from the previous one.
    * For example, `windows(2)` would yield elements at indices `0..2`, `1..3`,
      `2..4`, etc.
    * This is exactly the iteration pattern that we need for discrete
      convolution, which the Gray-Scott reaction computation that we'll study
      later is an instance of.

All these methodes are not just restricted to arrays and `Vec`s, you can just as
well apply them to slices, because they are actually methods of the [slice
type](https://doc.rust-lang.org/std/primitive.slice.html) to begin with. It just
happens that Rust, through some compiler magic,[^4] allows you to call slice
type methods on arrays and `Vec`s, as if they were the equivalent
all-encompassing `&v[..]` slice.

Therefore, whenever you are using arrays and `Vec`s, the documentation of the
slice type is also worth keeping around. Which is why the official documentation
helps you at this by copying it into the documentation of the
[`array`](https://doc.rust-lang.org/std/primitive.array.html) and
[`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) types.


## Exercise

Now, go to your code editor, open the `examples/03-looping.rs` source file, and
address the TODOs in it. The code should compile and runs successfully at the
end.

To check this, you may use the following command in your development
environment's terminal:

```bash
cargo run --example 03-looping
```

---


[^1]: It may be worth pointing out that replacing a major standard library
      abstraction like this in a mature programming language is not a very wise
      move. 4 years after the release of C++20, range support in the standard
      library of major C++ compilers is still either missing or very immature
      and support in third-party C++ libraries is basically nonexistent.
      Ultimately, C++ developers will unfortunately be the ones paying the price
      of this standard commitee decision by needing to live with codebases that
      confusingly mix and match STL iterators and ranges for many decades to
      come. This is just one little example, among many others, of why
      attempting to iteratively change C++ in the hope of getting it to the
      point where it matches the ergonomics of Rust, is ultimately a futile
      evolutionary dead-end that does the C++ community more harm than good...

[^2]: When arrays are used as e.g. `struct` members, they are allocated
      _inline_, so for example an array within a heap-allocated `struct` is part
      of the same allocation as the hosting struct.

[^3]: Iterators are themselves implemented using `unsafe`, but that's the
      standard library maintainers' problem to deal with, not yours.

[^4]: *Cough cough*
      [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) trait *cough
      cough*.
