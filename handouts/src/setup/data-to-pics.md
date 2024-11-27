<!-- Common subset of unix.md and windows.md -->

## HDF5-to-PNG renderer

Throughout the second part of the course, we will be producing HDF5 files which
contain time series of tabulated chemical concentrations. For quick and dirty
debugging, it is convenient to render those tabulated concentrations into
colorful PNG images.

To this end, you can use the `data-to-pics` program which was developed as part
of a previous version of this course. It can be installed as follows:

```bash
cargo install --git https://github.com/HadrienG2/grayscott.git data-to-pics
```
