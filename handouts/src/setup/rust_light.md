# Using `rust_light`

The [`rust_light`
image](https://plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/container_registry/1019)
provides the minimal amount of CLI tooling and libraries needed to do the
practicals. It is based on Ubuntu 22.04, with libraries installed system-wide
and the Rust toolchain and associated software installed locally to the root
user's home directory (`/root`).

The intent is for you to layer whatever code editing environment you prefer[^1]
on top of this using your favorite container image building mechanisms:
Dockerfiles, `docker commit`, `buildah`...

Alternatively, you can just directly bind-mount the source code [as done with
the `rust_code_server`
image](rust_code_server.md#downloading-the-exercises-source-code) (minus the
`:U` bind mount flag for Podman, see bullets below) and start editing it with
your local code editor on the host system, only using the container for builds
and executions. Compared to the suggested `rust_code_server` workflow...

- You cannot get early feedback from `rust-analyzer` while editing code, because
  it must be running inside of the container to have access to all dependencies.
  In general, if your code editor's has check/build/run shortcuts or other IDE
  functionality, it won't work as expected.
- You will not experience any file permission issues _inside_ of the container
  and can/should drop the `:U` bind mount flag when using Podman, because
  everything in the `rust_light` container runs as the root user, and root can
  write to any file no matter which UID/GID is the owner.
- If you are using Docker, you will still get file permission issues _outside_
  of the container because on Linux, any file created by the root user inside of
  a Docker container is owned by the host root account. Podman does not suffer
  from this issue.[^2]

Given that this environment is open-ended and geared towards expert container
users who want to go in exotic directions, a detailed tutorial cannot be easily
written, so I invite you to just e-mail the trainer if you run into any trouble
while going this route, and we'll try to figure it out together.

[^1]: Be warned that getting X11/Wayland software to work inside of Linux
      containers involves a fair amount of suffering. You will make your life
      easier by favoring editors that run directly in the terminal (like vi and
      nano) or expose a web-based gui via an HTTP server (like jupyterlab and
      code-server).

[^2]: Speaking from the perspective of using Docker and Podman in the
      conventional way, which for Docker involves a daemon with root privileges
      and for Podman involves a rootless setup with sub-UIDs/GIDs. If you are
      using more exotic setups, like rootless Docker or sudo'd Podman commands,
      your experience may differ.
