# Ownership

After running the summing benchmark from the last chapter, you should have
observed that its performance is quite bad. Half a nanosecond per vector element
may not sound like much, but when we're dealing with CPUs that can process tens
of multiplications in that time, it's already something to be ashamed of.

One reason why this happens is that our benchmark does not just square floats as
it should, it also generates a full `Vec` of them on every iteration. That's not
a desirable feature, as it shifts benchmark numbers away from what we are trying
to measure. So in this chapter we will study the ownership and borrowing
features of Rust that will let us reuse input vectors and stop doing this.


## Some historical context

### RAII in Rust and C++

Rust relies on the [Resource Acquisition Is Initialization
(RAII)](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization)
pattern in order to automatically manage system resources like heap-allocated
memory. This pattern was originally introduced by C++, and for those unfamiliar
with it, here is a quick summary of how it works:

- In modern structured programming languages, variables are _owned_ by a certain
  code scope. Once you exit this scope, the variable cannot be named or
  otherwise accessed anymore. Therefore, the state of the variable has become
  unobservable to the programmer, and compilers and libraries should be able to
  do arbitrary things to it without meaningfully affecting the observable
  behavior of the program.
- Library authors can leverage this scoping structure by defining destructor
  functions[^1], which are called when a variable goes out of scope. These
  functions are used to clean up all system state associated with the variable.
  For example, a variable that manages a heap allocation would use it to
  deallocate the associated, now-unreachable heap memory.

### Move semantics and its problems in C++

One thing which historical C++ did not provide, however, was an efficient way to
move a resource from one scope to another. For example, returning RAII types
from functions could only be made efficient through copious amounts of brittle
compiler optimizer black magic. This was an undesirable state of affair, so
after experimenting with several bad solutions including
[`std::auto_ptr`](https://en.cppreference.com/w/cpp/memory/auto_ptr), the C++
standardization commitee finally came up with a reasonable fix in C++11, called
_move semantics_.

The basic idea of move semantics is that it should be possible to transfer
ownership of one system resource from one variable to another. The original
variable would lose access to the resource, and give up on trying to liberate it
once its scope ends. While the new variable would gain access to the resource,
and become the one responsible for liberating it. Since the two variables
involved can be in different scopes, this could be used to resolve the function
return problem, among others.

Unfortunately, C++ move semantics were also heavily overengineered and bolted
onto a 26-years old programming language standard, with the massive user base
and the backwards compatibility constraints that come with it. As a result, the
design was made so complicated and impenetrable that even today, few C++
developers will claim to fully understand how it works. One especially bad
design decision, in retrospect, was the choice to make move semantics an opt-in
feature that each user-defined types had to individually add support for.
Predictably, few types in the ecosystem did, and as a result C++ move semantics
have mostly become an experts-only feature that you will be very lucky to see
working as advertised on anything but toy code examples.

### How Rust fixed move semantics

By virtue of being first released 4 years after C++11, Rust could learn from
these mistakes, and embrace a complete redesign of C++ move semantics that is
easier to understand and use, safer, and reliably works as intended. More
specifically, Rust move semantics improved upon C++11 by leveraging the
following insight:

- C++11 move semantics have further exacerbated the dangling pointer memory
  safety problems that had been plaguing C++ for a long time, by adding a new
  dangling variable problem in the form of moved-from variables. There was a
  pressing need to find a solution to this new problem, that could ideally also
  address the original problem.
- Almost every use case of C++ move constructors and move assignment operators
  could be covered by `memcpy()`-ing the bytes of the original variable into the
  new variable and ensuring that 1/the original variable cannot be used anymore
  and 2/its destructor will never run. By restricting the scope of move
  operations like this, they could be automatically and universally implemented
  for every Rust type without any programmer intervention.
- For types which do not manage resources, restricting access to the original
  variable would be overkill, and keeping it accessible after the memory copy is
  fine. The two copies are independent and each one can freely on its separate
  way.


## Moving and copying

In the general case, using a Rust value moves it. Once you have assigned the
value to a variable to another, passed it to a function, or used it in any other
way, you cannot use the original variable anymore. In other words, the
following code snippets are all illegal in Rust:

```rust,compile_fail
// Suppose you have a Vec defined like this...
let v = vec![1.2, 3.4, 5.6, 7.8];

// You cannot use it after assigning it to another variable...
let v2 = v;
println!("{v:?}");  // ERROR: v has been moved from

// ...and you cannot use it after passing it to a function
fn f(v: Vec<f32>) {}
f(v2);
println!("{v2:?}");  // ERROR: v2 has been moved from
```

Some value types, however, escape these restrictions by virtue of being safe to
`memcpy()`. For example, stack-allocated arrays do not manage heap-allocated
memory or any other system resource, so they are safe to copy:

```rust
let a = [1.2, 3.4, 5.6, 7.8];

let a2 = a;
println!("{a:?}");  // Fine, a2 is an independent copy of a

fn f(a: [f32; 4]) {}
f(a2);
println!("{a2:?}");  // Fine, f received an independent copy of a2
```

Types that use this alternate logic can be identified by the fact that they
implement the [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)
trait. Other types which can be copied but not via a simple `memcpy()` must use
the explicit `.clone()` operation from the
[`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) trait instead.
This ensures that expensive operations like heap allocations stand out in the
code, eliminating a classic performance pitfall of C++.[^2] 

```rust
let v = vec![1.2, 3.4, 5.6, 7.8];

let v2 = v.clone();
println!("{v:?}");  // Fine, v2 is an independent copy of v

fn f(v: Vec<f32>) {}
f(v2.clone());
println!("{v2:?}");  // Fine, f received an independent copy of v2
```

But of course, this is not good enough for our needs. In our motivating
benchmarking example, we do not want to simply replace our benckmark input
re-creation loop with a benchmark input copying loop, we want to remove the copy
as well. For this, we will need references and borrowing.


## References and borrowing

### The pointer problem

It has been said in the past that every problem in programming can be solved by
adding another layer of indirection, except for the problem of having too many
layers of indirection.

Although this quote is commonly invoked when discussing API design, one has to
wonder if the original author had programming language pointers and references
in mind, given how well the quote applies to them. Reasoned use of pointers can
enormously benefit a codebase, for example by avoiding unnecessary data
movement. But if you overuse pointers, your code will rapidly turn into a slow
and incomprehensible mess of pointer spaghetti.

Many attempts have been made to improve upon this situation, with interest
increasing as the rise of multithreading kept making things worse. [Functional
programming](https://en.wikipedia.org/wiki/Functional_programming) and
[communicating sequential
processes](https://en.wikipedia.org/wiki/Communicating_sequential_processes) are
probably the two most famous examples. But most of these formal models came with
very strong restrictions on how programs could be written, making each of them a
poor choice for a large amount of applications that did not "fit the model".

It can be argued that Rust is the most successful take at this problem from the
2010s, by virtue of managing to build a larger ecosystem over two simple but
very far-reaching sanity rules:

1. Any user-reachable reference must be safe to dereference
2. Almost[^3] all memory can be either shared in read-only mode or accessible
   for writing at any point in time, but never both at the same time.

### Shared references

In Rust, shared references are created by applying the ampersand `&` operator to
values. They are called shared references because they enable multiple variables
to share access to the same target:

```rust
// You can create as many shared references as you like...
let v = vec![1.2, 3.4, 5.6, 7.8];
let rv1 = &v;
let rv2 = rv1;  // ...and they obviously implement Copy

// Reading from all of them is fine
println!("{v:?} {rv1:?} {rv2:?}");
```

If this syntax reminds you of how we extracted slices from arrays and vectors
before, this is not a coincidence. A slice is a kind of Rust reference.

By the rules given above, a Rust reference cannot exit the scope of the variable
that it points to. And a variable that has at least one reference pointing to it
cannot be modified, moved, or go out of scope. More precisely, doing either of
these things will invalidate the reference, so it is not possible to use the
reference after this happens.

```rust,compile_fail
// This common C++ mistake is illegal in Rust: References can't exit data scope
fn dangling() -> &f32 {
    let x = 1.23;
    &x
}

// Mutating shared data invalidates the references, so this is illegal too
let mut data = 666;
let r = &data;
data = 123;
println!("{r}");  // ERROR: r has been invalidated
```

As a result, for the entire useful lifetime of a reference, its owner can assume
that the reference's target is valid and does not change. This is a very useful
invariant to operate under when applying optimizations like caching to your
programs.

### Mutable references

Shared references are normally[^3] read-only. You can read from them via either
the `*` dereference operator or the method call syntax that implicitly calls it,
but you cannot overwrite their target. For that you will need the mutable `&mut`
references:

```rust
let mut x = 123;
let mut rx = &mut x;
*rx = 666;
```

Shared and mutable references operate like a compiler-verified reader-writer
lock: at any point in time, data may be either accessible for writing by one
code path or accessible for reading by any number of code paths.

An obvious loophole would be to access memory via the original variable. But
much like shared references are invalidated when the original variable is
written to, mutable references are invalidated when the original variable is
either written to or read from. Therefore, code which has access to a mutable
reference can assume that as long as the reference is valid, reads and writes
which are made through it are not observable by any other code path or thread or
execution.

This prevents Rust code from getting into situations like NumPy in Python, where
modifying a variable in one place of the program can unexpectedly affect
readouts from memory made by another code path thousands of lines of code away:

```python
import numpy as np
a = np.array([1, 2, 3, 4])
b = a

# Much later, in a completely unrelated part of the program
b[0] = 0  # Will change the value of the first item of the original "a" variable
```


## What does this give us?

In addition to generally making code a lot easier to reason about by preventing
programmers from going wild with pointers, Rust references prevent many common
C/++ pointer usage errors that result in undefined behavior, including but not limited to:

- Null pointers
- Dangling pointers
- Misaligned pointers
- Iterator invalidation
- Data races between concurrently executing threads

Furthermore, the many type-level guarantees of references are exposed to the
compiler's optimizer, which can leverage them to speed up the code under the
assumption that forbidden things do not happen. This means that, for example,
there is no need for C's `restrict` keyword in Rust: almost[^3] every Rust
reference has `restrict`-like semantics without you needing to ask for it.

Finally, an ill-known benefit of Rust's shared XOR mutable data aliasing model
is that it closely matches the hardware-level MESI coherence protocol of CPU
caches, which means that code which idiomatically follows the Rust aliasing
model tends to exhibit better multi-threading performance, with fewer cache
ping-pong problems.


## At what cost?

The main drawback of Rust's approach is that even though it is much more
flexible than many previous attempts at making pointers easier to reason about,
many existing code patterns from other programming languages still do not
translate nicely to it.

So libraries designed for other programming languages (like GUI libraries) may
be hard to use from Rust, and all novice Rust programmers inevitable go through
a learning phase colloquially known as "fighting the borrow checker", where they
keep trying to do things that are against the rules before they full internalize
them.

A further-reaching consequence is that many language and library entities need
to exist in up to three different versions in order to allow working with owned
values, shared references, and mutable references. For example, the `Fn` trait
actually has two cousins called `FnMut` and `FnOnce`:

- The `Fn` trait we have used so far takes a shared reference to the input
  function object. Therefore, it cannot handle closures that can mutate internal
  state, and this code is illegal:

  ```rust,compile_fail
  fn call_fn(f: impl Fn()) {
      f()
  }

  let mut state = 42;
  call_fn(|| state = 43);  // ERROR: Can't call a state-mutating closure via Fn
  ```

  The flip side to this is that the implementation of `call_fn()` only needs a
  shared reference to f to call it, which gives it maximal flexibility.
- The `FnMut` trait takes a mutable reference to the input function object. So
  it can handle the above closure, but now a mutable reference will be needed to
  call it, which is more restrictive.

  ```rust
  fn call_fn_mut(mut f: impl FnMut()) {
      f()
  }

  let mut state = 42;
  call_fn_mut(|| state = 43);  // Ok, FnMut can handle state-mutating closure
  ```

- The `FnOnce` trait consumes the function object by value. Therefore, a
  function that implements this trait can only be called once. In exchange,
  there is even more flexibility on input functions, for example returning an
  owned value from the function is legal:

  ```rust
  fn call_fn_once(f: impl FnOnce() -> Vec<i16>) -> Vec<i16> {
      f()
  }

  let v = vec![1, 2, 3, 4, 5];
  call_fn_once(|| v);  // Ok, closure can move away v since it's only called once
  ```

Similarly, we actually have not one, but up to three ways to iterate over the
elements of Rust collections, depending on if you want owned values
(`into_iter()`), shared references (`iter()`), or mutable references
(`iter_mut()`). And `into_iter()` itself is a bit more complicated than this
because if you call it on a shared reference to a collection, it will yield
shared references to elements, and if you call it on a mutable reference to a
collection, it will yield mutable references to elements.

And there is much more to this, such as the `move` keyword that can be used to
force a closure to capture state by value when it would normally capture it by
reference, allowing said closure to be easily sent to a different threads of
executions... sufficient to say, the value/`&`/`&mut` dichotomy runs deep into
the Rust API vocabulary and affects many aspects of the language and ecosystem.


## References and functions

Rust's reference and borrowing rules interact with functions in interesting
ways. There are a few easy cases that you can easily learn:

1. The function only takes references as input. This requires no special
   precautions, since references are guaranteed to be valid for the entire
   duration of their existence.
2. The function takes only one reference as input, and returns one or more
   references as output. The compiler will infer by default that the output
   references probably comes from the input data, which is almost always true.
3. The function returns references out of nowhere. This is only valid when
   returning references to global data, in any other case you should thank the
   Rust borrow checker for catching your dangling pointer bug.

The first two cases are handled by simply replicating the reference syntax in
function parameter and return types, without any extra annotation...

```rust
fn forward_input_to_output(x: &i32) -> &i32 {
    x
}
```

...and the third case must be annotated with the `'static` keyword to advertise
the fact that only global state belongs here:

```rust
fn global_ref() -> &'static f32 {
    &std::f32::consts::PI
}
```

But as soon as a function takes multiple references as input, and return one
reference as an output, you need[^4] to specify which input(s) the output reference
can comes from, as this affects how other code can use your function. Rust
handles this need via lifetime annotations, which look like this:

```rust
fn output_tied_to_x<'a>(x: &'a i32, y: &f32) -> &'a i32 {
    x
}
```

Lifetime annotations as a language concept can take a fair amount of time to
master, so my advice to you as a beginner would be to avoid running into them at
the beginning of your Rust journey, even if it means sprinkling a few `.clone()`
here and there. It is possible to make cloning cheaper via reference counting if
need be, and this will save you from the trouble of attempting to learn all the
language subtleties of Rust at once. Pick your battles!


## Exercise:

Modify the benchmark from the previous chapter so that the input gets generated
once outside of `c.bench_function()`, then passed to `sum()` by reference rather
than by value. Measure performance again, and see how much it helped.



---

[^1]: In Rust, this is the job of the
      [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html) trait.

[^2]: The more experienced reader will have noticed that although this rule of
      thumb works well most of the time, it has some corner cases. `memcpy()`
      itself is cheap but not free, and copying large amount of bytes can easily
      become more expensive than calling the explicit copy operator of some
      types like reference-counted smart pointers. At the time where this course
      chapter is written, there is an [ongoing
      discussion](https://smallcultfollowing.com/babysteps/blog/2024/06/21/claim-auto-and-otherwise/)
      towards addressing this by revisiting the `Copy`/`Clone` dichotomy in
      future evolutions of Rust.

[^3]: Due to annoying practicalities like reference counting and mutexes in
      multi-threading, some amount of shared mutability has to exist in Rust.
      However, the vast majority of the types that you will be manipulating on a
      daily basis either internally follow the standard shared XOR mutable rule,
      or expose an API that does. Excessive use of shared mutability by Rust
      programs is frowned upon as unidiomatic.

[^4]: There is actually one last easy case involving methods from objects that
      return references to the `&self`/`&mut self` parameter, but we will not
      have time to get into this during this short course;
