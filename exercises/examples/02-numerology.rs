//! Numbers and arithmetic

// For historical reasons, some floating-point constants are stored in standard
// library modules named after the associated floating-point types.
use std::f32;

fn main() {
    // TODO: The compiler is quite unhappy with this computation.
    //       Find out why this happens and fix it!
    let x = 9_876_543_210;
    let y = 1_234_567_890;
    assert_eq!(x + y, 11_111_111_100);

    // TODO: Turn this C-style expression which does not compile into a valid
    //       Rust expression.
    let pi_over_3 = f32::consts::PI / 3;
    assert!((pi_over_3.cos() - 0.5).abs() < 1e-6);

    // TODO: For the purpose of this computation, integer wraparound on overflow
    //       is actually desired. Use `wrapping_add()` to express this.
    let wrapped = i8::MAX + 1;
    assert_eq!(wrapped, i8::MIN);

    // Show "cargo examples" command for skipping to the next example.
    grayscott_exercises::show_skip_command!();
}
