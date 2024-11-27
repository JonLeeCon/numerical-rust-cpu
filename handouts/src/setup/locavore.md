# Local installation

## Goals

To do the practicals on your local system, you are going to need the following
things:

- A basic software build toolchain including a libc development package and a
  linker.
- A Rust installation based on the official `rustup` installer.[^1]
- The [HDF5](https://www.hdfgroup.org/solutions/hdf5/) and
  [hwloc](https://www.open-mpi.org/projects/hwloc/) libraries.
- An implementation of the `pkg-config` command. This does not have to be the
  original implementation, [`pkgconf`](http://pkgconf.org/) on Unices and
  [`pkgconfiglite`](https://sourceforge.net/projects/pkgconfiglite/files/) on
  Windows work fine too.

Other tools which are not absolutely necessary but will prove convenient
include...

- A code editor with at least Rust syntax highlighting, and preferably TOML
  syntax highlighting and support for the `rust-analyzer` [language
  server](https://en.wikipedia.org/wiki/Language_Server_Protocol) too.
    * See [the index of the "Installation"
      section](https://rust-analyzer.github.io/manual.html) of the
      `rust-analyzer` manual for a fairly comprehensive list of supported
      editors and language server setup steps for each.
- A POSIX shell and the `lscpu`, `unzip` and `wget` command-line utilities.
  These will allow you to run the few non-Cargo commands featured in this course
  to automate some tasks.

The remainder of this chapter will guide you towards installing and setting up 
these dependencies, except for the code editor which we will treat as a personal
choice and leave you in charge of.

[^1]: Third-party `cargo` and `rustc` packages from Linux distributions,
      Homebrew and friends cannot be used during this course because we are
      going to need a nightly version of the compiler in order to demonstrate an
      important upcoming language feature pertaining to SIMD computations.

## OS-specific steps

Please pick your operating system for setup instructions:

- [Linux](linux.md)
- [macOS](macos.md)
- [Windows](windows.md)
