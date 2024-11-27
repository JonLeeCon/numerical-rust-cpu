# Gray-Scott introduction

We are now ready to introduce the final ~boss~ computation of this course: the
Gray-Scott reaction simulation. In this chapter, you will be taken through a
rapid tour of the pre-made setup that is provided to you for the purpose of
input initialization, kernel benchmarking, and output HDF5 production. We will
then conclude this tour by showing how one would implement a simple, unoptimized
version of the simulation using `ndarray`.

Along the way, you will also get a quick glimpse of how Rust's structs and
methods work. We did not get to explore this area of the language to fit the
course's short format, but in a nutshell they can be used to implement
encapsulated objects like C++ classes, and that's what we do here.


## Input initialization

In the Gray-Scott school that this course originates from, the reference C++
implementation of the Gray-Scott simulation would hardcode the initial input
state of the simulation. For the sake of keeping things simple and comparable,
we will do the same:

```rust,ignore
use ndarray::Array2;

/// Computation precision
///
/// It is a good idea to make this easy to change in your programs, especially
/// if you care about hardware portability or have ill-specified output
/// precision requirements.
///
/// Notice the "pub" keyword for exposing this to the outside world. All types,
/// functions... are private to the current code module by default in Rust.
pub type Float = f32;

/// Storage for the concentrations of the U and V chemical species
pub struct UV {
    pub u: Array2<Float>,
    pub v: Array2<Float>,
}
//
/// impl blocks like this let us add methods to types in Rust
impl UV {
    /// Set up the hardcoded chemical species concentration
    ///
    /// Notice the `Self` syntax which allows you to refer to the type for which
    /// the method is implemented.
    fn new(num_rows: usize, num_cols: usize) -> Self {
        let shape = [num_rows, num_cols];
        let pattern = |row, col| {
            (row >= (7 * num_rows / 16).saturating_sub(4)
                && row < (8 * num_rows / 16).saturating_sub(4)
                && col >= 7 * num_cols / 16
                && col < 8 * num_cols / 16) as u8 as Float
        };
        let u = Array2::from_shape_fn(shape, |(row, col)| 1.0 - pattern(row, col));
        let v = Array2::from_shape_fn(shape, |(row, col)| pattern(row, col));
        Self { u, v }
    }

    /// Set up an all-zeroes chemical species concentration
    ///
    /// This can be faster than `new()`, especially on operating systems like
    /// Linux where all allocated memory is guaranteed to be initially zeroed
    /// out for security reasons.
    fn zeroes(num_rows: usize, num_cols: usize) -> Self {
        let shape = [num_rows, num_cols];
        let u = Array2::zeros(shape);
        let v = Array2::zeros(shape);
        Self { u, v }
    }

    /// Get the number of rows and columns of the simulation domain
    ///
    /// Notice the `&self` syntax for borrowing the object on which the method
    /// is being called by shared reference.
    pub fn shape(&self) -> [usize; 2] {
        let shape = self.u.shape();
        [shape[0], shape[1]]
    }
}
```


## Double buffering

The Gray-Scott reaction is simulated by updating the concentrations of the U and
V chemical species many times. This is done by reading the old concentrations of
the chemical species from one array, and writing the new concentrations to
another array.

We could be creating a new array of concentrations every time we do this, but
this would require performing one memory allocation per simulation step, which
can be expensive. Instead, it is more efficient to use the double buffering
pattern. In this pattern, we keep two versions of the concentration in an array,
and on every step of the simulation, we read from one of the array slots and
write to the other array slot. Then we flip the role of the array slots for the
next simulation step.

We can translate this pattern into a simple encapsulated object...

```rust,ignore
/// Double-buffered chemical species concentration storage
pub struct Concentrations {
    buffers: [UV; 2],
    src_is_1: bool,
}
//
impl Concentrations {
    /// Set up the simulation state
    pub fn new(num_rows: usize, num_cols: usize) -> Self {
        Self {
            buffers: [UV::new(num_rows, num_cols), UV::zeroes(num_rows, num_cols)],
            src_is_1: false,
        }
    }

    /// Get the number of rows and columns of the simulation domain
    pub fn shape(&self) -> [usize; 2] {
        self.buffers[0].shape()
    }

    /// Read out the current species concentrations
    pub fn current(&self) -> &UV {
        &self.buffers[self.src_is_1 as usize]
    }

    /// Run a simulation step
    ///
    /// The user callback function `step` will be called with two inputs UVs:
    /// one containing the initial species concentration at the start of the
    /// simulation step, and one to receive the final species concentration that
    /// the simulation step is in charge of generating.
    ///
    /// Notice the `&mut self` syntax for borrowing the object on which
    /// the method is being called by mutable reference.
    pub fn update(&mut self, step: impl FnOnce(&UV, &mut UV)) {
        let [ref mut uv_0, ref mut uv_1] = &mut self.buffers;
        if self.src_is_1 {
            step(uv_1, uv_0);
        } else {
            step(uv_0, uv_1);
        }
        self.src_is_1 = !self.src_is_1;
    }
}
```

...and then this object can be used to run the simulation like this:

```rust,ignore
// Set up the concentrations buffer
let mut concentrations = Concentrations::new(num_rows, num_cols);

// ... other initialization work ...

// Main simulation loop
let mut running = true;
while running {
    // Update the concentrations of the U and V chemical species
    concentrations.update(|start, end| {
        // TODO: Derive new "end" concentration from "start" concentration
        end.u.assign(&start.u);
        end.v.assign(&start.v);
    });

    // ... other per-step action, e.g. decide whether to keep running,
    // write the concentrations to disk from time to time ...
    running = false;
}

// Get the final concentrations at the end of the simulation
let result = concentrations.current();
println!("u is {:#?}", result.u);
println!("v is {:#?}", result.v);
```


## HDF5 output

The reference C++ simulation lets you write down the concentration of the V
chemical species to an HDF5 file every N computation steps. This can be used to
check that the simulation works properly, or to turn the evolving concentration
"pictures" into a video for visualization purposes.

Following its example, we will use the [`hdf5`](https://docs.rs/hdf5) Rust
crate[^1] to write data to HDF5 too, using the same file layout conventions for
interoperability. Here too, we will use an encapsulated object design to keep
things easy to use correctly:

```rust,ignore
/// Mechanism to write down results to an HDF5 file
pub struct HDF5Writer {
    /// HDF5 file handle
    file: File,

    /// HDF5 dataset
    dataset: Dataset,

    /// Number of images that were written so far
    position: usize,
}

impl HDF5Writer {
    /// Create or truncate the file
    ///
    /// The file will be dimensioned to store a certain amount of V species
    /// concentration arrays.
    ///
    /// The `Result` return type indicates that this method can fail and the
    /// associated I/O errors must be handled somehow.
    pub fn create(file_name: &str, shape: [usize; 2], num_images: usize) -> hdf5::Result<Self> {
        // The ? syntax lets us propagate errors from an inner function call to
        // the caller, when we cannot handle them ourselves.
        let file = File::create(file_name)?;
        let [rows, cols] = shape;
        let dataset = file
            .new_dataset::<Float>()
            .chunk([1, rows, cols])
            .shape([num_images, rows, cols])
            .create("matrix")?;
        Ok(Self {
            file,
            dataset,
            position: 0,
        })
    }

    /// Write a new V species concentration table to the file
    pub fn write(&mut self, result: &UV) -> hdf5::Result<()> {
        self.dataset
            .write_slice(&result.v, (self.position, .., ..))?;
        self.position += 1;
        Ok(())
    }

    /// Flush remaining data to the underlying storage medium and close the file
    ///
    /// This should automatically happen on Drop, but doing it manually allows
    /// you to catch and handle I/O errors properly.
    pub fn close(self) -> hdf5::Result<()> {
        self.file.close()
    }
}

```

After adding this feature, our simulation code skeleton now looks like this:

```rust,ignore
// Set up the concentrations buffer
let mut concentrations = Concentrations::new(num_rows, num_cols);

// Set up HDF5 I/O
let mut hdf5 = HDF5Writer::create(file_name, concentrations.shape(), num_output_steps)?;

// Produce the requested amount of concentration tables
for _ in 0..num_output_steps {
    // Run a number of simulation steps
    for _ in 0..compute_steps_per_output_step {
        // Update the concentrations of the U and V chemical species
        concentrations.update(|start, end| {
            // TODO: Derive new "end" concentration from "start" concentration
            end.u.assign(&start.u);
            end.v.assign(&start.v);
        });
    }

    // Write down the current simulation output
    hdf5.write(concentrations.current())?;
}

// Close the HDF5 file
hdf5.close()?;
```


## Reusable simulation skeleton

Right now, our simulation's update function is a stub that simply copies the
input concentrations to the output concentrations without actually changing
them. At some point, we are going to need to compute the real updated chemical
species concentrations there.

However, we also know from our growing experience with software performance
optimization that we are going to need tweak this part of the code **a lot**. It
would be great if we could do this in a laser-focused function that is decoupled
from the rest of the code, so that we can easily do things like swapping
computation backends and seeing what it changes. As it turns out, a judiciously
placed callback interface lets us do just this:

```rust,ignore
/// Simulation runner options
pub struct RunnerOptions {
    /// Number of rows in the concentration table
    num_rows: usize,

    /// Number of columns in the concentration table
    num_cols: usize,

    /// Output file name
    file_name: String,

    /// Number of simulation steps to write to the output file
    num_output_steps: usize,

    /// Number of computation steps to run between each write
    compute_steps_per_output_step: usize,
}

/// Simulation runner, with a user-specified concentration update function
pub fn run_simulation(
    opts: &RunnerOptions,
    // Notice that we must use FnMut here because the update function can be
    // called multiple times, which FnOnce does not allow.
    mut update: impl FnMut(&UV, &mut UV),
) -> hdf5::Result<()> {
    // Set up the concentrations buffer
    let mut concentrations = Concentrations::new(opts.num_rows, opts.num_cols);

    // Set up HDF5 I/O
    let mut hdf5 = HDF5Writer::create(
        &opts.file_name,
        concentrations.shape(),
        opts.num_output_steps,
    )?;

    // Produce the requested amount of concentration tables
    for _ in 0..opts.num_output_steps {
        // Run a number of simulation steps
        for _ in 0..opts.compute_steps_per_output_step {
            // Update the concentrations of the U and V chemical species
            concentrations.update(&mut update);
        }

        // Write down the current simulation output
        hdf5.write(concentrations.current())?;
    }

    // Close the HDF5 file
    hdf5.close()
}
```


## Command-line options

Our simulation has a fair of tuning parameters. To those that we have already
listed in `RunnerOptions`, the computational chemistry of the Gray-Scott
reaction requires that we add the following tunable parameters:

- The speed at which V turns into P
- The speed at which U is added to the simulation and U, V and P are removed
- The amount of simulated time that passes between simulation steps

We could just hardcode all these parameters, but doing so would anger the gods
of software engineering and break feature parity with the reference C++ version.
So instead we will make these parameters configurable via command-line
parameters whose syntax and semantics strictly match those of the C++ version.

To this end, we can use the excellent [`clap`](https://docs.rs/clap/) library,
which provides the best API for parsing command line options that I have ever
seen in any programming language.

The first step, as usual, is to `clap` as a dependency to our project. We will
also enable the `derive` optional feature, which is the key to the
aforementioned nice API:

```bash
cargo add --features=derive clap
```

We will then add some annotations to the definition of our options structs,
explaining how they map to the command-line options that our program expects
(which follow the syntax and defaults of the C++ reference version for
interoperability):

```rust,ignore
use clap::Args;

/// Simulation runner options
#[derive(Debug, Args)]
pub struct RunnerOptions {
    /// Number of rows in the concentration table
    #[arg(short = 'r', long = "nbrow", default_value_t = 1080)]
    pub num_rows: usize,

    /// Number of columns in the concentration table
    #[arg(short = 'c', long = "nbcol", default_value_t = 1920)]
    pub num_cols: usize,

    /// Output file name
    #[arg(short = 'o', long = "output", default_value = "output.h5")]
    pub file_name: String,

    /// Number of simulation steps to write to the output file
    #[arg(short = 'n', long = "nbimage", default_value_t = 1000)]
    pub num_output_steps: usize,

    /// Number of computation steps to run between each write
    #[arg(short = 'e', long = "nbextrastep", default_value_t = 34)]
    pub compute_steps_per_output_step: usize,
}

/// Simulation update options
#[derive(Debug, Args)]
pub struct UpdateOptions {
    /// Speed at which U is added to the simulation and U, V and P are removed
    #[arg(short, long, default_value_t = 0.014)]
    pub feedrate: Float,

    /// Speed at which V turns into P
    #[arg(short, long, default_value_t = 0.054)]
    pub killrate: Float,

    /// Simulated time interval on each simulation step
    #[arg(short = 't', long, default_value_t = 1.0)]
    pub deltat: Float,
}
```

We then create a top-level struct which represents our full command-line
interface...

```rust,ignore
use clap::Parser;

/// Gray-Scott reaction simulation
///
/// This program simulates the Gray-Scott reaction through a finite difference
/// schema that gets integrated via the Euler method.
#[derive(Debug, Parser)]
#[command(version)]
pub struct Options {
    #[command(flatten)]
    runner: RunnerOptions,
    #[command(flatten)]
    pub update: UpdateOptions,
}
```

...and in the main function of our final application, we call the automatically
generated `parse()` method of that struct and retrieve the parsed command-line
options.

```rust,ignore
fn main() {
    let options = Options::parse();

    // ... now do something with "options" ...
}
```

That's it. With no extra work, `clap` will automatically provide our
simulation with a command-line interface that follows all standard Unix
conventions (e.g. supports both `--option value` and `--option=value`), handles
user errors, parses argument strings to their respective concrete Rust types,
and prints auto-generated help strings when `-h` or `--help` is passed.

Also, if you spend 10 more minutes on it, you can make as many of these options
as you want configurable via environment variables too. Which can be convenient
in scenarios where you cannot receive configuration through CLI parameters, like
inside of `criterion` microbenchmarks.


## Hardcoded parameters

Not all parameters of the C++ reference version are configurable. Some of them
are hardcoded, and can only be changed by altering the source code. Since we are
aiming for perfect user interface parity with the C++ version, we want to 
replicate this design in the Rust version.

For now, we will do this by adding a few constants with the hardcoded values to
the source code:

```rust
# type Float = f32;
#
/// Weights of the discrete convolution stencil
pub const STENCIL_WEIGHTS: [[Float; 3]; 3] = [
    [0.25, 0.5, 0.25],
    [0.5,  0.0, 0.5],
    [0.25, 0.5, 0.25]
];

/// Offset from the top-left corner of STENCIL_WEIGHTS to its center
pub const STENCIL_OFFSET: [usize; 2] = [1, 1];

/// Diffusion rate of the U species
pub const DIFFUSION_RATE_U: Float = 0.1;

/// Diffusion rate of the V species
pub const DIFFUSION_RATE_V: Float = 0.05;
```

In Rust, `const` items let you declare compile-time constants, much like
`constexpr` variables in C++, `parameter`s in Fortran, and `#define STUFF 123`
in C. We do not have the time to dive into the associated language
infrastructure, but for now, all you need to know is that the value of a const
will be copy-pasted on each point of use, which ensures that the compiler
optimizer can specialize the code for the value of the parameter of interest.


## Progress reporting

Simulations can take a long time to run. It is not nice to make users wait for
them to run to completion without any CLI output indicating how far along they
are and how much time remains until they are done. Especially when it is very
easy to add such reporting in Rust, thanks to the wonderful
[`indicatif`](https://docs.rs/indicatif) library.

To use it, we start by adding the library to our project's dependencies...

```bash
cargo add indicatif
```

Then in our main function, we create a progress bar with a number of steps
matching the number of computation steps...

```rust,ignore
use indicatif::ProgressBar;


let progress = ProgressBar::new(
    (options.runner.num_output_steps
        * options.runner.compute_steps_per_output_step) as u64,
);
```

...we increment it on each computation step...

```rust,ignore
progress.inc(1);
```

...and at the end of the simulation, we tell `indicatif` that we are done[^2]:

```rust,ignore
progress.finish();
```

That's all we need to add basic progress reporting to our simulation.



## Final code layout

This is not a huge amount of code overall, but it does get uncomfortably large
and unfocused for a single code module. So in the `exercises` Rust project, the
simulation code has been split over multiple code modules.

We do not have the time to cover the Rust module system in this course, but if
you are interested, feel free to skim through the code to get a rough idea of
how modularization is done, and ask any question that comes to your mind while
doing so.

Microbenchmarks can only access code from the main library (below the `src`
directory of the project, excluding the `bin/` subdirectory), therefore most of
the code lies there. In addition, we have added a simulation binary under
`src/bin/simulate.rs`, and a microbenchmark under `benches/simulate.rs`.


## Exercise

Here is a naïve implementation of a Gray-Scott simulation step implemented using
`ndarray`:

```rust,ignore
use crate::options::{DIFFUSION_RATE_U, DIFFUSION_RATE_V, STENCIL_OFFSET, STENCIL_WEIGHTS};

/// Simulation update function
pub fn update(opts: &UpdateOptions, start: &UV, end: &mut UV) {
    // Species concentration matrix shape
    let shape = start.shape();

    // Iterate over pixels of the species concentration matrices
    ndarray::azip!(
        (
            index (out_row, out_col),
            out_u in &mut end.u,
            out_v in &mut end.v,
            &u in &start.u,
            &v in &start.v
        ) {
            // Determine the stencil's input region
            let out_pos = [out_row, out_col];
            let stencil_start = array2(|i| out_pos[i].saturating_sub(STENCIL_OFFSET[i]));
            let stencil_end = array2(|i| (out_pos[i] + STENCIL_OFFSET[i] + 1).min(shape[i]));
            let stencil_range = array2(|i| stencil_start[i]..stencil_end[i]);
            let stencil_slice = ndarray::s![stencil_range[0].clone(), stencil_range[1].clone()];

            // Compute the diffusion gradient for U and V
            let [full_u, full_v] = (start.u.slice(stencil_slice).indexed_iter())
                .zip(start.v.slice(stencil_slice))
                .fold(
                    [0.; 2],
                    |[acc_u, acc_v], (((in_row, in_col), &stencil_u), &stencil_v)| {
                        let weight = STENCIL_WEIGHTS[in_row][in_col];
                        [acc_u + weight * (stencil_u - u), acc_v + weight * (stencil_v - v)]
                    },
                );

            // Deduce the change in U and V concentration
            let uv_square = u * v * v;
            let du = DIFFUSION_RATE_U * full_u - uv_square + opts.feedrate * (1.0 - u);
            let dv = DIFFUSION_RATE_V * full_v + uv_square
                - (opts.feedrate + opts.killrate) * v;
            *out_u = u + du * opts.deltat;
            *out_v = v + dv * opts.deltat;
        }
    );
}

/// Shorthand for creating a 2D Rust array from an index -> value mapping
fn array2<T>(f: impl FnMut(usize) -> T) -> [T; 2] {
    std::array::from_fn(f)
}
```

Please integrate it into the codebase such that it can is used by both the
simulation binary at `src/bin/simulate.rs` and the microbenchmark at
`benches/simulate.rs`. Then make sure everything works by running both of them
using the following commands:

```bash
# Must use -- to separate cargo options from program options
cargo run --release --bin simulate -- -n 5 -e 2
cargo bench --bench simulate
```

If you are using Devana, you will want to run these commands via `srun`, so that
the work is offloaded to a worker node. At this point in time, we are only using
a single CPU core.

It is expected that the last command will take a few minutes to complete. We are
just at the start of our journey, and there's a lot of optimization work to do.
But the set of benchmark configurations is designed to remain relevant by the
time where the simulation will be running much, much faster.

---

Also, starting at this chapter, the exercises are going to get significantly
more complex. Therefore, it is a good idea to keep track of old versions of your
work and have a way to get back to old versions. To do this, you can turn the
exercises codebase into a git repository...

```bash
cd ~/exercises
git init
git add --all
git commit -m "Initial commit"
```

...then save a commit at the end of each chapter, or more generally whenever you
feel like you have a codebase state that's worth keeping around for later.

```bash
git add --all
git commit -m "<Describe your code changes here>"
```


---

[^1]: More precisely the 
      [`hdf5-metno`](https://docs.rs/hdf5-metno/latest/hdf5_metno/) fork of that
      `hdf5` crate, because the author of the original crate sadly ceased
      maintenance without handing off his crates.io push rights to someone
      else...

[^2]: This step is needed because `indicatif` allows you to add more work to the
      progress bar.
