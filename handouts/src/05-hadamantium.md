# Hadamard

So far, we have been iterating over a single array/`Vec`/slice at a time. And
this already took us through a few basic computation patterns. But we must not
forget that with the tools introduced so far, jointly iterating over two `Vec`s
at the same time still involves some pretty ugly code:

```rust
let v1 = vec![1, 2, 3, 4];
let v2 = vec![5, 6, 7, 8];
for (idx, elem) in v1.into_iter().enumerate() {
    println!("v1[{idx}] is {elem}");
    // Ew, manual indexing with slow bound checks and panic risks... :(
    println!("v2[{idx}] is {}", v2[idx]);
}
```

Thankfully, there is an easy fix called `zip()`. Let's use it to implement the
Hadamard product!


## Combining iterators with `zip()`

Iterators come with an adapter method called `zip()`. Like `for` loops, this
method expects an object of a type that implements `IntoInterator`. What it does
is to consume the input iterator, turn the user-provided object into an
interator, and return a new iterator that yields pairs of elements from both the
original iterator and the user-provided iterable object:

```rust
let v1 = vec![1, 2, 3, 4];
let v2 = vec![5, 6, 7, 8];
/// Iteration will yield (1, 5), (2, 6), (3, 7) and (4, 8)
for (x1, x2) in v1.into_iter().zip(v2) {
    println!("Got {x1} from v1 and {x2} from v2");
}
```

Now, if you have used the `zip()` method of any other programming language for
numerics before, you should have two burning questions:

- What happens if the two input iterators are of different length?
- Is this really as performant as a manual indexing-based loop?

To the first question, other programming languages have come with three typical
answers:

1. Stop when the shortest iterator stops, ignoring remaining elements of the
   other iterator.
2. Treat this as a usage error and report it using some error handling
   mechanism.
3. Make it undefined behavior and give the compiler license to randomly trash
   the program.

As you may guess, Rust did not pick the third option. It could reasonably have
picked the second option, but instead it opted to pick the first option. This
was likely done because error handling comes at a runtime performance cost, that
was not felt to be acceptable for this common performance-sensitive operation
where user error is rare. But should you need it, option 2 can be easily built
as a third-party library, and is therefore [available via the popular
`itertools` crate](https://docs.rs/itertools/latest/itertools/fn.zip_eq.html).

Speaking of performance, Rust's `zip()` is, perhaps surprisingly, usually just
as good as a hand-tuned indexing loop[^1]. It does not exhibit the runtime
performance issues that you would face when using C++20's range-zipping
operations[^2]. And it will especially be often highly superior to manual
indexing code, which come with a risk of panics and makes you rely on the black
magic of compiler optimizations to remove indexing-associated bound checks.
Therefore, you are strongly encouraged to use `zip()` liberally in your code!


## Hadamard product

One simple use of `zip()` is to implement the Hadamard vector product.

This is one of several different kinds of products that you can use in linear
algebra. It works by taking two vectors of the same dimensionality as input, and
producing a third vector of the same dimensionality, which contains the pairwise
products of elements from both input vectors:

```rust
fn hadamard(v1: Vec<f32>, v2: Vec<f32>) -> Vec<f32> {
    assert_eq!(v1.len(), v2.len());
    v1.into_iter().zip(v2)
                  .map(|(x1, x2)| x1 * x2)
                  .collect() 
}
assert_eq!(
    hadamard(vec![1.2, 3.4, 5.6], vec![9.8, 7.6, 5.4]),
    [
        1.2 * 9.8,
        3.4 * 7.6,
        5.6 * 5.4
    ]
);
```


## Exercise

Now, go to your code editor, open the `examples/05-hadamantium.rs` source file,
and address the TODOs in it. The code should compile and runs successfully at
the end.

To check this, you may use the following command in your development
environment's terminal:

```bash
cargo run --example 05-hadamantium
```


---

[^1]: This is not to say that the hand-tuned indexing loop is itself perfect. It
      will inevitably suffer from runtime performance issues caused by
      suboptimal data alignment. But we will discuss how to solve this problem
      and achieve optimal Hadamard product performance after we cover data
      reductions, which suffer from **much** more severe runtime performance
      problems that include this one among many others.

[^2]: The most likely reason why this is the case is that Rust pragmatically
      opted to make tuples a primitive language type that gets special support
      from the compiler, which in turn allows the Rust compiler to give its LLVM
      backend very strong hints about how code should be generated (e.g. pass
      tuple elements via CPU registers, not stack pushes and pops that may or
      may not be optimized out by later passes). On its side, the C++
      standardization commitee did not do this because they cared more about
      keeping `std::tuple` a library-defined type that any sufficiently
      motivated programmer could re-implement on their own given an infinite
      amount of spare time. This is another example, if needed be, that even
      though both the C++ and Rust community care a lot about giving maximal
      power to library writers and minimizing the special nature of their
      respective standard libraries, it is important to mentally balance the
      benefits of this against the immense short-term efficiency of letting the
      compiler treat a few types and functions specially. As always, programming
      language design is all about tradeoffs.