# Training-day instructions

## Expectations and conventions

Welcome to this practical about writing high performance computing in Rust!

This course assumes that the reader has basic familiarity with C (especially
number types, arithmetic operations, string literals and stack vs heap). It will
thus not explain concepts which are rigorously identical between Rust and C for
the sake of concision. If this is not your case, feel free to ask the teacher
about any surprising construct in the course's material.

We will also compare Rust with C++ where they differ, so that readers familiar
with C++ can get a good picture of Rust specificities. But previous knowledge of
C++ should not be necessary to get a good understanding of Rust via this course.

Finally, we will make heavy use of "C/++" abbreviation as a shorter alternative
to "C and C++" when discussing common properties of C and C++, and how they
compare to Rust.


## Exercises source code

At the time where you registered, you should have been directed to [instructions
for setting up your development environment](pathfinding.md). If you did
not follow these instructions yet, now is the right time!

Now that the course has begun, we will download a up-to-date copy of [the
exercises' source code](exercises.zip) and unpack it somewhere inside of your
development environement. This will create a subdirectory called `exercises/` in
which we will be working during the rest of the course.

Please pick your environement below in order to get appropriate instructions:

{{#tabs}}
{{#tab name="Native Windows"}}

Get a PowerShell terminal, then `cd` into the place where you would like to download
the exercises' source code and run the following script:

```powershell
Invoke-Command -ScriptBlock {
      $ErrorActionPreference="Stop"
      if (Test-Path exercises) {
            throw "ERROR: Please move or delete the existing 'exercises' subdirectory"
      }
      Invoke-WebRequest https://numerical-rust-cpu-d1379d.pages.math.cnrs.fr/setup/exercises.zip  `
                        -OutFile exercises.zip
      Expand-Archive exercises.zip -DestinationPath .
      Remove-Item exercises.zip
      Set-Location exercises
}
```

{{#endtab}}
{{#tab name="Linux container"}}

From a shell **inside of the container**[^1], run the following
sequence of commands to update the exercises source code that you have already
downloaded during container setup.

Beware that any change to the previously downloaded code will be lost in the
process.

```bash
cd ~
# Can't use rm -rf exercises because we must keep the bind mount alive
for f in $(ls -A exercises); do rm -rf exercises/$f; done  \
&& wget https://numerical-rust-cpu-d1379d.pages.math.cnrs.fr/setup/exercises.zip  \
&& unzip -u exercises.zip  \
&& rm exercises.zip  \
&& cd exercises
```

[^1]: If you're using `rust_code_server`, this means using the terminal
      pane of the web-based VSCode editor.

{{#endtab}}
{{#tab name="Other (Devana, local Linux/macOS, WSL)"}}

Get a shell in your intended develoment environement[^2], then `cd` into the
place where you would like to download the exercises' source code and run the
following script:

```bash
if [ -e exercises ]; then
    echo "ERROR: Please move or delete the existing 'exercises' subdirectory"
else
    wget https://numerical-rust-cpu-d1379d.pages.math.cnrs.fr/setup/exercises.zip  \
    && unzip exercises.zip  \
    && rm exercises.zip  \
    && cd exercises
fi
```

[^2]: That would be a regular shell for a local Linux/macOS installation, an SSH
      connexion with the appropriate environment modules loaded for Devana, and
      a Windows Subsystem for Linux shell for WSL.

{{#endtab}}
{{#endtabs}}


## General advice

The exercises are based on code examples that are purposely incorrect.
Therefore, any code example within the provided `exercises` Rust project, except
for `00-hello.rs`, will either fail to compile or fail to run to completion. A
TODO code comment or ... symbol will indicate where failures are expected, and
your goal in the exercises will be to modify the code in such a way that the
associated example will compile and run. For runtime failures, you should not
need to change the failing assertion, instead you will need to change other code
such that the assertion passes.

If you encounter any failure which does not seem expected, or if you otherwise
get stuck, please call the trainer for guidance!

With that being said, let's get started with actual Rust code. You can move to
the next page, or any other page within the course for that matter, through the
following means:

- Left and right keyboard arrow keys will switch to the previous/next page.
  Equivalently, arrow buttons will be displayed at the end of each page, doing
  the same thing.
- There is a menu on the left (not shown by default on small screen, use the
  top-left button to show it) that allows you to quickly jump to any page of the
  course. Note, however, that the course material is designed to be read in
  order.
- With the magnifying glass icon in the top-left corner, or the "S" keyboard
  shortcut, you can open a search bar that lets you look up content by keywords.
