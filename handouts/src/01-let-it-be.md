# First steps

Welcome to Rust computing. This chapter will be a bit longer than the next ones,
because we need to introduce a number of basic concepts that you will likely all
need to do subsequent exercises. Please read on carefully!


## Hello world

Following an ancient tradition, let us start by displaying some text on stdout:

```rust
fn main() {
    println!("Hello world!");
}
```

Notice the following:

- In Rust, functions declarations start with the `fn` keyword. Returning a value
  is optional, if you do so the return value's type comes after the parameter
  list (as in `fn myfunc() -> f32`).
- Like in C/++, the main function of a program is called `main`.
- It is legal to return nothing from `main`. Like `return 0;` in C/++, it
  indicates success.
- Sending a line of text to standard output can be done using the `println!()`
  macro. We'll get back to why it is a macro later, but in a nutshell it allows
  controlling the formatting of values in a way that is similar to `printf()` in
  C and f-strings in Python.


## Variables

Rust is in many ways a hybrid of programming languages from the C/++ family and
the ML family (Ocaml, Haskell). Following the latter's tradition, it uses a
variable declaration syntax that is very reminiscent of mathematical notation:

```rust
let x = 4;
```

As in mathematics, variables are immutable by default, so the following code
does not compile:

```rust,compile_fail
let x = 4;
x += 2;  // ERROR: Can't modify non-mut variable
```

If you want to modify a variable, you must make it mutable by adding a `mut`
keyword after `let`:

```rust
let mut x = 4;
x += 2;  // Fine, variable is declared mutable
```

This design nudges you towards using immutable variables for most things, as in
mathematics, which tends to make code easier to reason about.

Alternatively, Rust allows for variable shadowing, so you are allowed to define
a new variable with the same name as a previously declared variable, even if it
has a different type:

```rust
let foo = 123;  // Here "foo" is an integer
let foo = "456";  // Now "foo" is a string and old integer "foo" is not reachable
```

This pattern is commonly used in scenarios like parsing where the old value
should not be needed after the new one has been declared. It is otherwise a bit
controversial, and can make code harder to read, so please don't abuse this
possibility.


## Type inference

### What gets inferred

Rust is a strongly typed language like C++, yet you may have noticed that the
variable declarations above contain no types. That's because the language
supports type inference as a core feature: the compiler can automatically
determine the type of variables based on various information.

- First, the value that is affected to the variable may have an unambiguous
  type. For example, string literals in Rust are always of type `&str`
  ("reference-to-string"), so the compiler knows that the following variable
  `s` must be of type `&str`:

  ```rust
  let s = "a string literal of type &str";
  ```

- Second, the way a variable is used after declaration may give its type away.
  If you use a variable in a context where a value of type `T` is expected, then
  that variable must be of type `T`.

  For example, Rust provides a heap-allocated variable-sized array type called
  `Vec` (similar to `std::vector` in C++), whose length is defined to be of type
  `usize` (similar to `size_t` in C/++). Therefore, if you use an integer as the
  length of a `Vec`, the compiler knows that this integer must be of type
  `usize`:

  ```rust
  let len = 7;  // This integer variable must be of type usize...
  let v = vec![4.2; len];  // ...because it's used as the length of a Vec here.
                           // (we'll introduce this vec![] macro later on)
  ```

- Finally, numeric literals can be annotated to force them to be of a specific
  type. For example, the literal `42i8` is a 8-bit signed integer, the literal
  `666u64` is a 64-bit unsigned integer, and the `12.34f32` literal is a 32-bit
  ("single precision") IEEE-754 floating point number. By this logic, the
  following variable `x` is a 16-bit unsigned integer:

  ```rust
  let x = 999u16;
  ```

  If none of the above rules apply, then by default, integer literals will be
  inferred to be of type `i32` (32-bit signed integer), and floating-point
  literals will be inferred to be of type `f64` (double-precision floating-point
  number), as in C/++. This ensures that simple programs compile without
  requiring type annotations.

  Unfortunately, this last fallback rule is not 100% reliable. There are a
  number of common code patterns that will not trigger it, typically involving
  some form of genericity.

### What does not get inferred

There are cases where these three rules will not be enough to determine a
variable's type. This happens in the presence of generic type and functions.

Getting back to the `Vec` type, for example, it is actually a generic type
`Vec<T>` where `T` can be almost any Rust type[^1]. As with `std::vector` in
C++, you can have a `Vec<i32>`, a `Vec<f32>`, or even a `Vec<MyStruct>` where
`MyStruct` is a data structure that you have defined yourself.

This means that if you declare empty vectors like this...

```rust,compile_fail
// The following syntaxes are strictly equivalent. Neither compile. See below.
let empty_v1 = Vec::new();
let empty_v2 = vec![];
```

...the compiler has no way to know what kind of `Vec` you are dealing with. This
cannot be allowed because the properties of a generic type like `Vec<T>` heavily
depend on what concrete `T` parameter it's instantiated with (e.g. `Vec`
equality is only defined when the inner data type has an equality operator).
Therefore the above code does not compile.

In that case, you can enforce a specific variable type using type ascription:

```rust
// The following syntaxes are almost equivalent.
// In both cases, the compiler knows this is a Vec<bool> because we said so
let empty_vb1: Vec<bool> = Vec::new();  // Specify the type of empty_vb1 directly
let empty_vb2 = Vec::<bool>::new();  // Specify the type of Vec we are creating
```

[^1]: ...with the exception of dynamic-sized types, an advanced topic which we
      cannot afford to cover during this short course. Ask the teacher if you
      really want to know ;)

### Inferring most things is the idiom

If you are coming from another programming language where type inference is
either not provided, or very hard to reason about as in C++, you may be tempted
to use type ascription to give an explicit type to every single variable. But I
would advise resisting this temptation for a few reasons:

- Rust type inference rules are much simpler than those of C++. It only takes a
  small amount of time to "get them in your head", and once you do, you will get
  more concise code that is less focused on pleasing the type system and more on
  performing the task at hand.
- Doing so is the idiomatic style in the Rust ecosystem. If you don't follow it,
  your code will look odd to other Rust developers, and you will have a harder
  time reading code written by other Rust developers.
- If you have any question about inferred types in your program, Rust comes with
  excellent IDE support via `rust-analyze`, so it is easy to configure your
  code editor to make it display inferred types, either all the time or on
  mouse hover.

But of course, there are limits to the "infer everything" approach. If every
single type in the program was inferred, then a small change somewhere in the
implementation your program could non-locally change the type of many other
variables in the program, or even in client code, resulting in accidental API
breakages, as commonly seen in dynamically typed programming language.

For this reason, Rust will not let you use type inference in entities that may
appear in public APIs, like function signatures or struct declarations. This
means that in Rust code, type inference will be restricted to the boundaries of
a single function's implementation. Which makes it more reliable and easier to
reason about, as long as you do not write huge functions.


## Back to `println!()`

With variable declarations out of the way, let's go back to our hello world
example and investigate the Rust text formatting macro in more details.

Remember that at the beginning of this chapter, we wrote this hello world
statement:

```rust
println!("Hello world!");
```

This program called the `println` macro with a string literal as an argument.
Which resulted in that string being written to the program's standard output,
followed by a line feed.

If all we could pass to `println` was a string literal, however, it wouldn't
need to be a macro. It would just be a regular function.

But like f-strings in Python, the `println` provides a variety of text
formatting operations, accessible via curly braces. For example, you can
interpolate variables from the outer scope...

```rust
let x = 4;
// Will print "x is 4"
println!("x is {x}");
```

...pass extra arguments in a style similar to printf in C...

```rust
let s = "string";
println!("s is {}", s);
```

...and name arguments so that they are easier to identify in complex usage.

```rust
println!("x is {x} and y is {y}. Their sum is {x} + {y} = {sum}",
         x = 4,
         y = 5,
         sum = 4 + 5);
```

You can also control how these arguments are converted to strings, using a
mini-language that is described in [the documentation of the std::fmt module
from the standard library](https://doc.rust-lang.org/std/fmt/).

For example, you can enforce that floating-point numbers are displayed with a
certain number of decimal digits:


```rust
let x = 4.2;
// Will print "x is 4.200000"
println!("x is {x:.6}");
```

---

`println!()` is part of a larger family of text formatting and text I/O macros
that includes...

- `print!()`, which differs from `println!()` by not adding a trailing newline
  at the end. Beware that since stdout is line buffered, this will result in no
  visible output until the next `println!()`, unless the text that you are
  printing contains the `\n` line feed character.
- `eprint!()` and `eprintln!()`, which work like `print!()` and `println!()` but
  write their output to stderr instead of stdout.
- `write!()` and `writeln!()`, which take a byte or text output stream[^2] as an
  extra argument and write down the specified text there. This is the same idea
  as `fprintf()` in C.
- `format!()`, which does not write the output string to any I/O stream, but
  instead builds a heap-allocated `String` containing it for later use.

All of these macros use the same format string mini-language as `println!()`,
although their semantics differ. For example, `write!()` takes an extra output
stream arguments, and returns a `Result` to account for the possibility of I/O
errors. Since these errors are rare on stdout and stderr, they are just treated
as fatal by the `print!()` family, keeping the code that uses them simpler.

[^2]: In Rust's abstraction vocabulary, text can be written to implementations
      of one of the `std::io::Write` and `std::fmt::Write` traits. We will
      discuss traits much later in this course, but for now you can think of a
      trait as a set of functions and properties that is shared by multiple
      types, allowing for type-generic code. The distinction between `io::Write`
      and `fmt::Write` is that `io::Write` is byte-oriented and `fmt::Write`
      latter is text-oriented. We need this distinction because not every byte
      stream is a valid UTF-8 text stream.


## From `Display` to `Debug`

So far, we have been printing simple numerical types. What they have in common
is that there is a single, universally agreed upon way to display them, modulo
formatting options. So the Rust standard library can afford to incorporate this
display logic into its stability guarantees.

But some other types are in a muddier situation. For example, take the `Vec`
dynamically-sized array type. You may think that something like "[1, 2, 3, 4,
5]" would be a valid way to display an array containing the numbers containing
numbers from 1 to 5. But what happens when the array contains billions of
numbers? Should we attempt to display all of them, drowning the user's terminal
in endless noise and slowing down the application to a crawl? Or should we
summarize the display in some way like numpy and pandas do in Python?

There is no single right answer to this kind of question, and attempting to
account for all use cases would bloat up Rust's text formatting mini-language
very quickly. So instead, Rust does not provide a standard text display for
these types, and therefore the following code does not compile:

```rust,compile_fail
// ERROR: Type Vec does not implement Display
println!("{}", vec![1, 2, 3, 4, 5]);
```

All this is fine and good, but we all know that in real-world programming, it is
very convenient during program debugging to have a way to exhaustively display
the contents of a variable. Unlike C++, Rust acknowledges this need by
distinguishing two different ways to translate a typed value to text:

- The `Display` trait provides, for a limited set of types, an "official"
  value-to-text translation logic that should be fairly consensual,
  general-purpose, suitable for end-user consumption, and can be covered by
  library stability guarantees.
- The `Debug` trait provides, for almost every type, a dumber value-to-text
  translation logic which prints out the entire contents of the variable,
  including things which are considered implementation details and subjected to
  change. It is purely meant for developer use, and showing debug strings to end
  users is somewhat frowned upon, although they are tolerated in
  developer-targeted output like logs or error messages.

As you may guess, although `Vec` does not implement the `Display` operation, it
does implement `Debug`, and in the mini-language of `println!()` et al, you can
access this alternate `Debug` logic using the `?` formatting option:

```rust
println!("{:?}", vec![1, 2, 3, 4, 5]);
```

As a more interesting example, strings implement both `Display` and `Debug`. The
`Display` variant displays the text as-is, while the `Debug` logic displays it
as you would type it in a program, with quotes around it and escape sequences
for things like line feeds and tab stops:

```rust
let s = "This is a \"string\".\nWell, actually more of an str.";
println!("String display: {s}");
println!("String debug: {s:?}");
```

Both `Display` and `Debug` additionally support an alternate display mode,
accessible via the `#` formatting option. For composite types like `Vec`, this
has the effect of switching to a multi-line display (one line feed after each
inner value), which can be more readable for complex values:

```rust
let v = vec![1, 2, 3, 4, 5];
println!("Normal debug: {v:?}");
println!("Alternate debug: {v:#?}");
```

For simpler types like integers, this may have no effect. It's ultimately up to
implementations of `Display` and `Debug` to decide what this formatting option
does, although staying close to the standard library's convention is obviously
strongly advised.

---

Finally, for an even smoother printout-based debugging experience, you can use
the `dbg!()` macro. It takes an expression as input and prints out...

- Where you are in the program (source file, line, column).
- What is the expression that was passed to `dbg`, in un-evaluated form (e.g.
  `dbg!(2 * x)` would literally print "2 * x").
- What is the result of evaluating this expression, with alternate debug
  formatting.

...then the result is re-emitted as an output, so that the program can keep
using it. This makes it easy to annotate existing code with `dbg!()` macros,
with minimal disruption:

```rust
let y = 3 * dbg!(4 + 2);
println!("{y}");
```


## Assertions

With debug formatting covered, we are now ready to cover the last major
component of the exercises, namely assertions.

Rust has an `assert!()` macro which can be used similarly to the C/++ macro of
the same name: make sure that a condition that should always be true if the code
is correct, is indeed true. If the condition is not true, the thread will panic.
This is a process analogous to throwing a C++ exception, which in simple cases
will just kill the program.

```rust,should_panic
assert!(1 == 1);  // This will have no user-visible effect
assert!(2 == 3);  // This will trigger a panic, crashing the program
```

There are, however, a fair number of differences between C/++ and Rust
assertions:

- Although well-intentioned, the C/++ practice of only checking assertions in
  debug builds has proven to be tracherous in practice. Therefore, most Rust
  assertions are checked in all builds. When the runtime cost of checking an
  assertion in release builds proves unacceptable, you can use `debug_assert!()`
  instead, for assertions which are only checked in debug builds.
- Rust assertions do not abort the process in the default compiler
  configuration. Cleanup code will still run, so e.g. files and network
  conncections will be closed properly, reducing system state corruption in the
  event of a crash. Also, unlike unhandled C++ exceptions, Rust panics make it
  trivial to get a stack trace at the point of assertion failure by setting the
  `RUST_BACKTRACE` environment variable to 1.
- Rust assertions allow you to customize the error message that is displayed in
  the event of a failure, using the same formatting mini-language as
  `println!()`:

  ```rust,should_panic
  let sonny = "dead";
  assert!(sonny == "alive",
          "Look how they massacred my boy :( Why is Sonny {}?",
          sonny);
  ```

Finally, one common case for wanting custom error messages in C++ is when
checking two variables for equality. If they are not equal, you will usually
want to know what are their actual values. In Rust, this is natively supported
by the `assert_eq!()` and `assert_ne!()`, which respectively check that two
values are equal or not equal.

If the comparison fails, the two values being compared will be printed out with
Debug formatting.

```rust,should_panic
assert_eq!(2 + 2,
           5,
           "Don't you dare make Big Brother angry! >:(");
```


## Exercise

Now, go to your code editor, open the `examples/01-let-it-be.rs` source file
inside of the provided `exercises/` source tree, and address the TODOs in it.

Once you are done, the code should compile and run successfully. To check this,
you may use the following command in your development environment's terminal:

```bash
cargo run --example 01-let-it-be
```
