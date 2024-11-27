//! Basic simulation data handling

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
