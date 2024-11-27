# Linux containers

We provide two different containerized Rust development environment options,
geared towards two different kinds of container users.

* For maximal ease of use, the [`rust_code_server`
  environment](rust_code_server.md) provides a pre-configured installation of
  the popular VS Code editor that you can use from your web browser. This is the
  recommended choice if you like this code editor, are not very familiar with
  containers or just want to get something working with minimal effort.
* If you are an experienced container user and want more control, the
  [`rust_light` environment](rust_light.md) only provides the minimum CLI
  tooling and libraries needed to build and run the course's code. You can then
  set up any code editing environment you like on top of it.

Both environments have been tested with the Docker and Podman container
runtimes. **They are not compatible with Apptainer/Singularity** because the way
this container runtime manages user accounts and home directories is
fundamentally at odds with our needs.
