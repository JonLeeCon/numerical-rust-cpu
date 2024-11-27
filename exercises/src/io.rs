//! File input and output

use crate::data::{Float, UV};
use hdf5::{Dataset, File};

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
