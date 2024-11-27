//! From zip to Hadamard

fn main() {
    // TODO: Implement a linear algebra vector sum function using the same style
    //       as the Hadamard product implementation provided by the course:
    fn sum(v1: Vec<f32>, v2: Vec<f32>) -> Vec<f32> {
        ...
    }
    assert_eq!(
        sum(vec![1.2, 3.4, 5.6], vec![9.8, 7.6, 5.4]),
        [
            1.2 + 9.8,
            3.4 + 7.6,
            5.6 + 5.4
        ]
    );

    // Show "cargo examples" command for skipping to the next example.
    grayscott_exercises::show_skip_command!();
}
