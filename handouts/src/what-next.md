# Next steps

This concludes our introduction to numerical computations in Rust. I hope you
enjoyed it.

As you could see, the language is in a bit of an interesting spot as of 2024,
where this course is written. Some aspects like iterators and multi-threading
are much more advanced than in any other programming language, others like SIMD
and N-d arrays are in a place that's roughly comparable to the C++ state of the
art, and then other things like GPU programming need more work, that in some
cases is itself blocked on more fundamental ongoing language/compiler work
(variadic generics, generic const expressions, specialization, etc).

What the language needs most today, however, are contributors to the library
ecosystem. So if you think that Rust is close to your needs, and would be usable
for project X if and only if it had library Y, then stop and think. Do you have
the time to contribute to the existing library Y draft, or to write one from
scratch yourself? And would this be more or less worthy of your time, in the
long run, than wasting hours on programming languages that have a more mature
library ecosystem, but whose basic design is stuck in an evolutionary dead-end?

If you think the latter, then consider first perfecting your understanding of
Rust with one of these fine reference books:

- [The Rust Programming Language](https://doc.rust-lang.org/book/) is maintained
  by core contributors of the project, often most up to date with respect to
  language evolutions, and freely available online. It is, however, written for
  an audience that is relatively new to programming, so its pace can feel a bit
  slow for experienced practicioners of other programming languages.
- For this more advanced audience, the ["Programming
  Rust"](https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/)
  book by Blandy et al is a worthy purchase. It assumes the audience already has
  some familiarity with C/++, and exploits this to skip more quickly to aspects
  of the language that this audience will find interesting.

More of a fan of learning by doing? There are resources for that as well, like
the [Rustlings](https://github.com/rust-lang/rustlings/) interactive tutorial
that teaches you the language by making your fix programs that don't compile, or
[Rust By Example](https://doc.rust-lang.org/rust-by-example/) which gives you
lots of ready-made snippets that you can take inspiration from in your early
Rust projects.

And then, as you get more familiar with the language, you will be hungry for
more reference documentation. Common docs to keep close by are the [standard
library docs](https://doc.rust-lang.org/std/index.html), [Cargo
book](https://doc.rust-lang.org/cargo/index.html), [language
reference](https://doc.rust-lang.org/reference/index.html), and the
[Rustonomicon](https://doc.rust-lang.org/nomicon/index.html) for those cases
that warrant the use of unsafe code.

You can find all of these links and more on the language's official
documentation page at <https://www.rust-lang.org/learn>.

And that will be it for this course. So long, and thanks for all the fish!
