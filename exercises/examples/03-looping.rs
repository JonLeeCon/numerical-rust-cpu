//! Loops, iterations, and array/vec/slice manipulations

fn main() {
    // First, let's compute an arithmetic sequence of integers:
    let max_n = 100;
    let sequence = (1..=max_n).into_iter().collect::<Vec<_>>();

    // TODO: Compute the sum of this sequence using either a for loop or an
    //       iterator method, at your preference.
    let sum = ...;
    assert_eq!(sum, max_n * (max_n + 1) / 2);

    // Now, here's a small array of floating-point numbers...
    let floats = [1.2, 3.4, 5.6, 7.8, 9.0];

    // TODO: Compute a `Vec` of three elements, where the output element at
    //       index i is the average of the input array elements at index i, i+1,
    //       and i+2. Try to write the code in a manner that...
    //       - Would still work if the input array was changed (different
    //         values, different length...).
    //       - Minimizes use of explicit indexing operations. You will still
    //         need some of them at this stage of your learning process, but in
    //         a few chapters, we will see how to completely get rid of them.
    let boxcar = ...;
    assert_eq!(
        boxcar,
        [
            (1.2 + 3.4 + 5.6) / 3.0,
            (3.4 + 5.6 + 7.8) / 3.0,
            (5.6 + 7.8 + 9.0) / 3.0,
        ]
    );

    // Show "cargo examples" command for skipping to the next example.
    grayscott_exercises::show_skip_command!();
}
