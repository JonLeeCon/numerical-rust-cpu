<!-- Common subset of linux.md and macos.md -->


## Rust

To compile Rust code, you will need a Rust toolchain. You can install the one we
need like this:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none  \
&& . "$HOME/.cargo/env"  \
&& rustup toolchain install {{#include ../../../exercises/rust-toolchain}}
```

If you want to use `rust-analyzer`, you may want to add a `--component
rust-analyzer` flag to the `rustup toolchain install` command at the end. This
will ensure that you get a `rust-analyzer` version that is fully compatible with
the compiler version that we are going to use.


{{#include data-to-pics.md}}


## Environment test

Your Rust development environment should now be ready for this course's
practical work. I highly advise testing it by using the following shell script:

```bash
wget https://plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/-/archive/solution/numerical-rust-cpu-solution.zip  \
&& unzip numerical-rust-cpu-solution.zip  \
&& rm numerical-rust-cpu-solution.zip  \
&& cd numerical-rust-cpu-solution/exercises  \
&& cargo run -- -n3  \
&& mkdir pics  \
&& data-to-pics -o pics/  \
&& cd ../..  \
&& rm -rf numerical-rust-cpu-solution
```

It downloads, builds and runs the expected source code at the end of the last
chapter of this course, then renders the associated images, and finally cleans
up after itself by deleting everything.
