# Squaring

As you could see if you did the previous set of exercises, we have already
covered enough Rust to start doing some actually useful computations.

There is still one important building block that we are missing to make the most
of Rust iterator adapters, however, and that is anonymous functions, also known
as lambda functions or lexical closures in programming language theory circles.

In this chapter, we will introduce this language feature, and show how it can be
used, along with Rust traits and the higher-order function pattern, to compute
the square of every element of an array using fully idiomatic Rust code.


## Meet the lambda

In Rust, you can define a function anywhere, including inside of another
function[^1]. Parameter types are specified using the same syntax as variable
type ascription, and return types can be specified at the end after a `->` arrow
sign:

```rust
fn outsourcing(x: u8, y: u8) {
    fn sum(a: u8, b: u8) -> u8 {
        // Unlike in C/++, return statements are unnecessary in simple cases.
        // Just write what you want to return as a trailing expression.
        a + b
    }
    println!("{}", sum(x, y));
}
```

However, Rust is not Python, and inner function definitions cannot capture
variables from the scope of outer function definitions. In other words, the
following code does not compile:

```rust,compile_fail
fn outsourcing(x: u8, y: u8) {
    fn sum() -> u8 {
        // ERROR: There are no "x" and "y" variables in this scope.
        x + y
    }
    println!("{}", sum());
}
```

Rust provides a slightly different abstraction for this, namely anonymous
functions aka lambdas aka closures. In addition to being able to capture
surrounding variables, these also come with much lighter-weight syntax for
simple use cases...

```rust
fn outsourcing(x: u8, y: u8) {
    let sum = || x + y;  // Notice that the "-> u8" return type is inferred.
                         // If you have parameters, their type is also inferred.
    println!("{}", sum());
}
```

...while still supporting the same level of type annotation sophistication as
full function declarations, should you need it to guide type inference or
improve clarity:

```rust
fn outsourcing(x: u8, y: u8) {
    let sum = |a: u8, b: u8| -> u8 { a + b };
    println!("{}", sum(x, y));
}
```

The main use case for lambda functions, however, is interaction with
higher-order functions: functions that take other functions as inputs and/or
return other functions as output.


## A glimpse of Rust traits

We have touched upon the notion of traits several time before in this course,
without taking the time to really explain it. That's because Rust traits are a
complex topic, which we do not have the luxury of covering in depth in this
short course.

But now that we are getting to higher-order functions, we are going to need to
interact a little bit more with Rust traits, so this is a good time to expand a
bit more on what Rust traits do.

Traits are the cornerstone of Rust's genericity and polymorphism system. They
let you define a common protocol for interacting with several different types in
a homogeneous way. If you are familiar with C++, traits in Rust can be used to
replace any of these C++ features:

- Virtual methods and overrides
- Templates and C++20 concepts, with first-class support for the "type trait"
  pattern
- Function and method overloading
- Implicit conversions

The main advantage of having one single complex general-purpose language feature
like this, instead of many simpler narrow-purpose features, is that you do not
need to deal with interactions between the narrow-purpose features. As C++
practicioners know, these can be result in quite surprising behavior and getting
their combination right is a very subtle art.

Another practical advantage is that you will less often hit a complexity wall,
where you hit the limits of the particular language feature that you were using
and must rewrite large chunks code in terms of a completely different language
feature.

Finally, Rust traits let you do things that are impossible in C++. Such as
adding methods to third-party types, or verifying that generic code follows its
intended API contract.

<details>

<summary>If you are a C++ practicioner and just started thinking "hold on,
weren't C++20 concepts supposed to fix this generics API contract problem?",
please click on the arrow for a full explanation.</summary>

> Let us assume that you are writing a generic function and claim that it works
> with any type that has an addition operator. The Rust trait system will check
> that this is indeed the case as the generic code is compiled. Should you use any
> other type operation like, say, the multiplication operator, the compiler will
> error out during the compilation of the generic code, and ask you to either add
> this operation to the generic function's API contract or remove it from its
> implementation.
> 
> In contrast, C++20 concepts only let you check that the type parameters that
> generic code is instantiated with match the advertised contract. Therefore, in
> our example scenario, the use of C++20 concepts will be ineffective, and the
> compilation of the generic code will succeed in spite of the stated API
> contract being incorrect.
> 
> It is only later, as someone tries to use your code with a type that has an
> addition operator but no multiplication operator (like, say, a linear algebra
> vector type that does not use `operator*` for the dot product), that an error
> will be produced deep inside of the implementation of the generic code.
> 
> The error will point at the use of the multiplication operator by the
> implementation of the generic code. Which may be only remotely related to what
> the user is trying to do with your library, as your function may be a small
> implementation detail of a much bigger functionality. It may thus take users
> some mental gymnastics to figure out what's going on. This is part of why
> templates have a bad ergonomics reputation in C++, the other part being that
> function overloading as a programming language feature is fundamentally
> incompatible with good compiler error messages.
> 
> And sadly this error is unlikely to be caught during your testing because
> generic code can only be tested by instantitating it with specific types. As
> an author of generic code, you are unlikely to think about types with an
> addition operator but no multiplication operator, since these are relatively
> rare in programming. 
> 
> To summarize, unlike C++20 concepts, Rust traits are actually effective at
> making unclear compiler error messages deep inside of the implementation of
> generic code a thing of the past. They do not only work under the unrealistic
> expectation that authors of generic code are perfectly careful to type in the
> right concepts in the signature of generic code, and to keep the unchecked
> concept annotations up to date as the generic code's implementation evolves[^2].

</details>


## Higher order functions

One of Rust's most important traits is the
[`Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html) trait, which is
implemented for types that can be called like a function. It also has a few
cousins that we will cover later on.

Thanks to special treatment by the compiler[^3], the `Fn` trait is actually a
family of traits that can be written like a function signature, without
parameter names. So for example, an object of a type that implements the
`Fn(i16, f32) -> usize` trait can be called like a function, with two parameters
of type `i16` and `f32`, and the call will return a result of type `usize`.

You can write a generic function that accepts any object of such a type like
this...

```rust
fn outsourcing(op: impl Fn(i16, f32) -> usize) {
    println!("The result is {}", op(42, 4.2));
}
```

...and it will accept any matching callable object, including both regular
functions, and closures:

```rust
# fn outsourcing(op: impl Fn(i16, f32) -> usize) {
#     println!("The result is {}", op(42, 6.66));
# }
#
// Example with a regular function
fn my_op(x: i16, y: f32) -> usize {
    (x as f32 + 1.23 * y) as usize
}
outsourcing(my_op);

// Example with a closure
outsourcing(|x, y| {
    println!("x may be {x}, y may be {y}, but there is only one answer");
    42
});
```

As you can see, closures shine in this role by keeping the syntax lean and the
code more focused on the task at hand. Their ability to capture environment can
also be very powerful in this situation, as we will see in later chapters.

You can also use the `impl Trait` syntax as the return type of a function, in
order to state that you are returning an object of a type that implements a
certain trait, without specifying what the trait is.

This is especially useful when working with closures, because the type of a
closure object is a compiler-internal secret that cannot be named by the
programmer:

```rust
/// Returns a function object with the signature that we have seen so far
fn make_op() -> impl Fn(i16, f32) -> usize {
    |x, y| (x as f32 + 1.23 * y) as usize
}
```

By combining these two features, Rust programmers can very easily implement any
higher-order function that takes a function as a parameter or returns a
function as a result. And because the code of these higher-order functions is
specialized for the specific function type that you're dealing with at compile
time, runtime performance can be much better than when using dynamically
dispatched higher-order function abstractions in other languages, like
`std::function` in C++[^4].


## Squaring numbers at last

The `Iterator` trait provides a number of methods that are actually higher-order
functions. The simpler of them is the `map` method, which consumes the input
iterator, takes a user-provided function, and produces an output iterator whose
elements are the result of applying the user-provided function to each element
of the input iterator:

```rust
let numbers = [1.2f32, 3.4, 5.6];
let squares = numbers.into_iter()
                     .map(|x| x.powi(2))
                     .collect::<Vec<_>>();
println!("{numbers:?} squared is {squares:?}");
```

And thanks to good language design and heroic optimization work by the Rust
compiler team, the result will be just as fast as hand-optimized assembly for
all but the smallest input sizes[^5].


## Exercise

Now, go to your code editor, open the `examples/04-square-one.rs` source file,
and address the TODOs in it. The code should compile and runs successfully at
the end.

To check this, you may use the following command in your development
environment's terminal:

```bash
cargo run --example 04-square-one
```

---

[^1]: This reflects a more general Rust design choice of letting almost
      everything be declared almost anywhere, for example Rust will happily
      declaring types inside of functions, or even inside of value expressions.

[^2]: You may think that this is another instance of the C++ standardization
      commitee painstakingly putting together a bad clone of a Rust feature as
      an attempt to play catch-up 5 years after the first stable release of
      Rust. But that is actually not the case. C++ concepts have been in
      development for more than 10 years, and were a major contemporary
      inspiration for the development of Rust traits along with Haskell's
      typeclasses. However, the politically dysfunctional C++ standardization
      commitee failed to reach an agreement on the original vision, and had to
      heavily descope it before they succeeded at getting the feature out of the
      door in C++20. In contrast, Rust easily succeeded at integrating a much
      more ambitious generics API contract system into the language. This
      highlights once again the challenges of integrating major changes into an
      established programming language, and why the C++ standardization commitee
      might actually better serve C++ practicioners by embracing the "refine and
      polish" strategy of its C and Fortran counterparts.

[^3]: There are a number of language entities like this that get special
      treatment by the Rust compiler. This is done as a pragmatic alternative to
      spending more time designing a general version that could be used by
      library authors, but would get it in the hands of Rust developers much
      later. The long-term goal is to reduce the number of these exceptions over
      time, in order to give library authors more power and reduce the amount of
      functionality that can only be implemented inside of the standard library.

[^4]: Of course, there is no free lunch in programming, and all this
      compile-time specialization comes at the cost. As with C++ templates, the
      compiler must effectively recompile higher-order functions that take
      functions as a parameter for each input function that they're called with.
      This will result in compilation taking longer, consuming more RAM, and 
      producing larger output binaries. If this is a problem and the runtime
      performance gains are not useful to your use case, you can use `dyn Trait`
      instead of `impl Trait` to switch to dynamic dispatch, which works much
      like C++ `virtual` methods. But that is beyond the scope of this short
      course.

[^5]: To handle arrays smaller than about 100 elements optimally, you will need
      to specialize the code for the input size (which is possible but beyond
      the scope of this course) and make sure the input data is optimally
      aligned in memory for SIMD processing (which we will cover later on).
