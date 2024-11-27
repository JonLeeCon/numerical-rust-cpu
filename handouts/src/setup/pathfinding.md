# Environment setup

To do the practical work from this course, you will need access to a suitably
configured Rust development environment. We support three different ways to get
such an environment, but depending on your particular situation, they may not
all be available/suitable for you:

- If you have been granted access to it during registration, then you can use
  the [Devana computing cluster](devana.md) as your development environment.
- If you are using a Linux desktop environment[^1] and a CPU that is based on
  the common x86_64 CPU architecture (any modern CPU from Intel or AMD), then
  another way to get a pre-built development environment is to use [Linux
  containers](containers.md).
- If none of the above applies (e.g. you do not have access to Devana and you
  use a modern Mac with an Arm CPU), or if you fancy it for any another reason,
  then you can also [manually install the required packages](locavore.md) on
  your computer.

[^1]: There are non-native ways to run Linux containers from a Windows or macOS
      desktop: Docker Desktop, WSL2, etc. They all work by running the container
      inside of a hidden x86 Linux VM. We don't recommend them because in our
      experience you may experience issues connecting to the code editor of the
      `rust_code_server` image, and surprising runtime performance
      characteristics on Arm-based Macs due to Rosetta emulation.
