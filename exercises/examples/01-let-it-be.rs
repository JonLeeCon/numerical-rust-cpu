//! Variable bindings, type inference, and console output

fn main() {
    let i = 1;

    // TODO: Make this operation compile
    i += 23;

    // You should not need to modify this line
    assert_eq!(i, 24);

    // TODO: This line should define an empty Vec<f32>. Make it compile.
    let v = Vec::new();

    // TODO: Use the dbg!() macro to print out the intermediate 1.0 / 3.0 ratio.
    //       Then extract it into a separate variable and use `eprintln!()` to
    //       write it down to stderr with 10 significant digits.
    let f = 1.0f32 / 3.0 * 3.0;
    println!("f is {f} aka {f:?}");

    // TODO: Finally, feel free to copy and paste any code example from the
    //       handouts here in order to run it on your own environement, with
    //       freedom to tweak it and see how that affects the result ;)

    // Once this example runs successfully we'll conclude execution by
    // displaying the "cargo examples" command variation that you would run in
    // order to skip all the way to the next exercise in the future.
    grayscott_exercises::show_skip_command!();
}
