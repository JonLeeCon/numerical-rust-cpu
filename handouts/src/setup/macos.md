# macOS

## Homebrew

To install CLI tools and libraries on macOS while retaining basic sanity, it is
strongly recommended to use the [Homebrew](https://brew.sh/) package manager. If
you have not already installed it, it can be done like this:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

This will also prompt you to install the "Xcode command line tools" if you have
not done so already. You will need those to build any software, including Rust
software, so that's a good thing.


## CLI tools and libraries

Now that the basics are covered, we can install all needed packages as follows:

```bash
brew install curl hdf5 hwloc pkgconf unzip util-linux wget
```

You may get a warning stating that `pkgconf` can't be installed because
`pkg-config` is installed. This is harmless: they are two compatible
implementations of the same functionality.


{{#include unix.md}}
