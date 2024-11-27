# Numbers

Since this is a numerical computing course, a large share of the material will
be dedicated to the manipulation of numbers (especially floating-point ones). It
is therefore essential that you get a good grasp of how numerical data works in
Rust. Which is the purpose of this chapter.


## Primitive types

We have previously mentioned some of Rust's primitive numerical types. Here is
the current list:

- `u8`, `u16`, `u32`, `u64` and `u128` are fixed-size unsigned integer types. 
  The number indicates their storage width in bits.
- `usize` is an unsigned integer type suitable for storing the size of an object
  in memory. Its size varies from one computer to another : it is 64-bit wide on
  most computers, but can be as narrow as 16-bit on some embedded platform.
- `i8`, `i16`, `i32`, `i64`, `i128` and `isize` are signed versions of the above
  integer types.
- `f32` and `f64` are the single-precision and double-precision IEEE-754 
  floating-point types.

This list is likely to slowly expand in the future, for example there are
proposals for adding `f16` and `f128` to this list (representing IEEE-754
half-precision and quad-precision floating point types respectively). But for
now, these types can only be manipulated via third-party libraries.


## Literals

As you have seen, Rust's integer and floating-point literals look very similar
to those of C/++. There are a few minor differences, for example the
quality-of-life feature to put some space between digits of large numbers uses
the `_` character instead of `'`...

```rust
println!("A big number: {}", 123_456_789);
```

...but the major difference, by far, is that literals are not typed in Rust.
Their type is almost always inferred based on the context in which they are
used. And therefore...

- In Rust, you rarely need typing suffixes to prevent the compiler from
  truncating your large integers, as is the norm in C/++.
- Performance traps linked to floating point literals being treated as double
  precision, in situations when you actually wanted single precision
  computations, are less common in Rust.

Part of the reason why type inference works so well in Rust is that unlike C/++,
Rust has no implicit conversions.


## Conversions

In C/++, every time one performs arithmetic or assigns values to variables, the
compiler will silently insert conversions between number types as needed to get
the code to compile. This is problematic for two reasons:

1. "Narrowing" conversions from types with many bits to types with few bits can
   lose important information, and thus produce wrong results.
2. "Promoting" conversions from types with few bits to types with many bits can
   result in computations being performed with excessive precision, at a
   performance cost, only for the hard-earned extra result bits to be discarded
   during the final variable affectation step.

If we were to nonetheless apply this notion in a Rust context, there would be a
third Rust-specific problem, which is that implicit conversions would break the
type inference of numerical literals in all but the simplest cases. If you can
pass variables of any numerical types to functions accepting any other numerical
type, then the compiler's type inference cannot know what is the numerical
literal type that you actually intended to use. This would greatly limit type
inference effectiveness.

For all these reasons, Rust does not allow for implicit type conversions. A
variable of type `i8` can only accept values of type `i8`, a variable of type
`f32` can only accept values of type `f32`, and so on.

If you want C-style conversions, the simplest way is to use `as` casts:

```rust
let x = 4.2f32 as i32;
```

As many Rust programmers were unhappy with the lossy nature of these casts,
fancier conversions with stronger guarantees (e.g. only work if no information
is lost, report an error if overflow occurs) have slowly been made available.
But we probably won't have the time to cover them in this course.


## Arithmetic

The syntax of Rust arithmetic is generally speaking very similar to that of
C/++, with a few minor exceptions like `!` replacing `~` for integer bitwise
NOT. But the rules for actually using these operators are quite different.

For the same reason that implicit conversions are not supported, mixed
arithmetic between multiple numerical types is rarely supported in Rust. This
will often be a pain points for people used to the C/++ way, as it means that
classic C numerical expressions like `4.2 / 2` are invalid and will not compile.
Instead, you will need to get used to writing `4.2 / 2.0`.

On the flip side, Rust tries harder than C/++ to handler incorrect arithmetic
operations in a sensible manner. In C/++, two basic strategies are used:

1. Some operations, like overflowing unsigned integers or assigning the 123456
   literal to an 8-bit integer variable, silently produce results that violate
   mathematical intuition.
2. Other operations, like overflowing signed integers or casting floating-point
   NaNs and infinities to integers, result in undefined behavior. This gives the
   compiler and CPU license to trash your entire program (not just the function
   that contains the faulty instruction) in unpredictable ways.

As you may guess by the fact that signed and unsigned integer operations are
treated differently, it is quite hard to guess which strategy is being used,
even though one is obviously a lot more dangerous than the other.

But due to the performance impact of checking for arithmetic errors at runtime,
Rust cannot systematically do so and remain performance-competitive with C/++.
So a distinction is made between debug and release builds:

- In debug builds, invalid arithmetic stops the program using panics. Rust
  panics are similar to C++ exceptions, except you are not encouraged to recover
  from them. They are meant to stop buggy code before it does more damage, not
  to report "expected" error conditions.
- In release builds, invalid Rust arithmetic silently produces wrong results,
  but it never causes undefined behavior.

As one size does not fit all, individual integer and floating-point types also
provide methods which re-implement the arithmetic operator with different
semantics. For example, the `saturating_add()` method of integer types handle
addition overflow and underflow by returning the maximal or minimal value of the
integer type of interest, respectively:

```rust
println!("{}", 42u8.saturating_add(240));  // Prints 255
println!("{}", (-40i8).saturating_add(-100));  // Prints -128
```


## Methods

In Rust, unlike in C++, any type can have methods, not just class-like types. As
a result, most of the mathematical functions that are provided as free functions
in the C and C++ mathematical libraries are provided as methods of the
corresponding types in Rust:

```rust
let x = 1.2f32;
let y = 3.4f32;
let basic_hypot = (x.powi(2) + y.powi(2)).sqrt();
``` 

Depending on which operation you are looking at, the effectiveness of this
design choice varies. On one hand, it works great for operations which are
normally written on the right hand side in mathematics, like raising a number to
a certain power. And it allows you to access mathematical operations with less
module imports. On the other hand, it looks decidedly odd and Java-like for
operations which are normally written in prefix notation in mathematics, like
`sin()` and `cos()`.

If you have a hard time getting used to it, note that prefix notation can be
quite easily implemented as a library, see for example
[`prefix-num-ops`](http://docs.rs/prefix-num-ops).

The set of operations that Rust provides on primitive types is also a fair bit
broader than that provided by C/++, covering many operations which are
traditionally only available via compiler intrinsics or third-party libraries in
other languages. Although to C++'s credit, it must be said that this situation
has actually improved in newer standard revisions.

To know which operations are available via methods, just check the
[documentation page for the associated arithmetic
types](https://doc.rust-lang.org/std/index.html#primitives).


## Exercise

Now, go to your code editor, open the `examples/02-numerology.rs` source file,
and address the TODOs in it. The code should compile and runs successfully at
the end.

To check this, you may use the following command in your development
environment's terminal:

```bash
cargo run --example 02-numerology
```
