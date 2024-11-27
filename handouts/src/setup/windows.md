# Windows

## Native vs WSL

Windows is not the easiest OS to set up for native HPC software development.

If you like to use the [Windows Subsystem for
Linux](https://learn.microsoft.com/en-us/windows/wsl/install), the easiest route
will likely be for you to open a WSL shell and jump to the [Linux
instructions](linux.md). By default WSL sets up an Ubuntu environment, where the
Ubuntu family's instructions should work.

The main thing that you'll lose with respect to a native environment, is the
ability to easily use `rust-analyzer` in your code editor. You could get there
by running a Linux editor (e.g. the Linux version of VSCode) inside of WSL, but
the UI will likely have a non-native look and sub-par responsiveness.

If you really want a native (MSVC-based) Rust development environment, read on.


## Rust and Visual Studio

The Windows system libraries and standard linker are distributed as part of the
[Visual Studio](https://visualstudio.microsoft.com/fr/) IDE[^1]. It comes in
several editions of very different pricing (from freeware to $250/month) and
licensing terms, and it is generally speaking a bit of a hassle to set up.

However, learning by writing code during this tutorial qualifies as personal
use, which means that you can use the free Community edition. And if you have
not set it up already yourself, you can let the Rust toolchain installer
automate away all the installation steps for you, as a prelude to the
installation of the Rust toolchain. All you will need to do, then is wait for
the various installation steps to complete. The process of installing Visual
Studio will take a good while, and seem stuck at time, but be patient, it will
eventually complete.

To get a Rust toolchain, and Visual Studio if you need it, just go to [the Rust
project's "Getting started" page](https://www.rust-lang.org/learn/get-started),
download the rustup-init tool (you will probably want the 64-bit version unless
your computer is very old), run it, and follow the instructions of the
installation wizard.

[^1]: Not to be confused with Visual Studio Code, which is just a code editor
      and does not contain the required tooling to build Windows software. There
      are some alternatives to Visual Studio for this purpose, like the
      GCC-based [MinGW-w64](https://www.mingw-w64.org/), but using them makes
      you a second-class citizen of the Windows developer ecosystem, e.g. you
      lose the ability to use pre-built libraries and convenient tools like
      vcpkg.


## `git` and `vcpkg`

In the dark ages of Windows software development, libraries were considered so
hard to build that the normal way to use them was to download pre-built binaries
from the author's website.

This wasn't great because different versions of the Visual Studio toolchain are
ABI-incompatible with each other, which means that you can only reliably use a
binary library with the Visual Studio version that has been used to compile it.

Library authors were thus expected to maintain binaries for every Visual Studio
release since the dawn of the universe, and understandably didn't. Which means
that sooner or later you would end up wanting to use two libraries whose
binaries had no Visual Studio release in common.

This problem is nowadays resolved by using [vcpkg](https://vcpkg.io/), which
brings some of the greatness of Unix-style package management to Windows. It can
build libraries automatically for your Visual Studio version, installing
dependencies as needed, like e.g. [Homebrew](https://formulae.brew.sh/) would on
macOS.

Before you can install `vcpkg`, however, you will need to install the `git`
version control software if you have not done so already. This can be done using
[the Windows installer from the official
website](https://git-scm.com/downloads/win). You will get asked many questions
during installation, but if in doubt, you can just go with the default choices,
they are quite reasonable.

Once `git` is installed, you can open a PowerShell terminal in the directory
where you would like to install vcpkg (preferably a subdirectory of your user
directory so you don't encounter permission issues and can update it easily) and
run the following PowerShell script:

```powershell
Invoke-Command -ScriptBlock {
      $ErrorActionPreference="Stop"
      git clone https://github.com/microsoft/vcpkg.git
      Set-Location vcpkg
      .\bootstrap-vcpkg.bat
}
```

To make the resulting vcpkg installation easier to use, it is a good idea to go
to the environment variables settings panel (which you can easily find by typing
"Variables" in the Windows search bar), and modify the user-specific environment
variables as follows:

- Create a new `VCPKG_ROOT` variable that points to your vcpkg installation.
- Add `%VCPKG_ROOT%` to the `Path` list.

After this is done, you should be able to close all your PowerShell windows,
open a new one and type in `vcpkg --help`. You will then get a description
of the vcpkg command-line options in return.


## `pkgconf`, `hwloc` and `hdf5`

With `vcpkg` at hand, we can now easily install our remaining dependencies:

```powershell
vcpkg install pkgconf hwloc hdf5
```

However, these dependencies end up somewhere deep inside `vcpkg`'s installation
directory, where build scripts won't find them. We need to tweak
our user environment variables again to fix that:

- Add all of the following to the `Path` list:
  * `%VCPKG_ROOT%\installed\x64-windows\bin`
  * `%VCPKG_ROOT%\installed\x64-windows\tools\pkgconf`
- Add a new variable called `PKG_CONFIG_PATH` with the value
  `%VCPKG_ROOT%\installed\x64-windows\lib\pkgconfig`.

Again, after adjusting the variables, you will need to close all your PowerShell
windows and open a new one for the environment changes to take effect.

Finally, `vcpkg` builds hwloc as `hwloc.lib`, whereas the generated pkgconfig
file tells the program linker to look for `libhwloc.lib`, and the same goes for
`hdf5.lib`. In an ideal world, we would work around this with a symlink. But
considering that symlinks are a privileged admin feature on Windows for reasons
only known to Microsoft, a dumb copy will be a lot easier and almost as good:

```powershell
Copy-Item $env:VCPKG_ROOT\installed\x64-windows\lib\hdf5.lib  `
          $env:VCPKG_ROOT\installed\x64-windows\lib\libhdf5.lib
Copy-Item $env:VCPKG_ROOT\installed\x64-windows\lib\hwloc.lib  `
          $env:VCPKG_ROOT\installed\x64-windows\lib\libhwloc.lib
```

And with this, your Rust build environment should be ready.


{{#include data-to-pics.md}}


## Environment test

Your Rust development environment should now be ready for this course's
practical work. I highly advise testing it by using the following PowerShell script:

```powershell
Invoke-Command -ScriptBlock {
      $ErrorActionPreference="Stop"

      Invoke-WebRequest https://plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/-/archive/solution/numerical-rust-cpu-solution.zip  `
                        -OutFile solution.zip
      Expand-Archive solution.zip -DestinationPath .
      Remove-Item solution.zip

      Set-Location numerical-rust-cpu-solution/exercises
      cargo run -- -n3
      New-Item -Name pics -ItemType directory
      data-to-pics -o pics
      Set-Location ..\..
      Remove-Item numerical-rust-cpu-solution -Recurse
}
```

It downloads, builds and runs the expected source code at the end of the last
chapter of this course, then renders the associated images, and finally cleans
up after itself by deleting everything.
