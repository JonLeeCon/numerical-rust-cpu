# Numerical Computing with Rust on CPU

This is a fork of Hadrien Grasland (grasland)'s [numerical-rust-cpu](https://plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/-/tree/main?ref_type=heads).

This repository contains...

- Handouts, a render of which is online at
  <https://numerical-rust-cpu-d1379d.pages.math.cnrs.fr/>.
- Code exercises, located in the `exercises` directory.

The CI builds two Linux container images meant for external use:

- [`rust_code_server`](https://plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/container_registry/1020)
  contains a server-based installation of Visual Studio Code, which you can use
  to edit code in the school's intended development environment.
- [`rust_light`](https://plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/container_registry/1019)
  is a minimal development environment, which is good enough to run tests, or
  install a console text editor via apt and start hacking locally.

These images have been tested with the Docker and Podman OCI container runtimes.
**They are not compatible with Apptainer/Singularity**. For more information,
please check out [the course's environement setup
guide](https://numerical-rust-cpu-d1379d.pages.math.cnrs.fr/setup/pathfinding.html).
