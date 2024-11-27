//! Closures, traits and higher-order functions

fn main() {
    // In the course, we have seen that the `map()` iterator adapter is a
    // higher-order function that can be used to transform iterator elements:
    let transformed = [1, 2, 3].into_iter().map(|x| 2 * x).collect::<Vec<_>>();
    assert_eq!(transformed, [2, 4, 6]);

    // The `filter()` iterator adapter is a different higher-order function
    // where instead of returning a new element, the function returns a boolean.
    // The elements for which that boolean is `true` will be kept in the output
    // iterator, and the elements for which that boolean is `false` will be
    // dropped:
    let odd = (1..10)
        .into_iter()
        .filter(|x| x % 2 == 0)
        .collect::<Vec<_>>();
    // And another cool iterator method is the `all()` reduction, which takes a
    // similar predicate function and returns a boolean that is true if the
    // predicate function returns `true` for all iterator elements.
    assert!(odd.into_iter().all(|x| x % 2 == 0));

    // TODO: Use `map()` followed by `filter()` to compute the square of all
    //       elements of the following `Vec`, then select those that are are
    //       smaller than or equal to 500.
    let v = (1..1000).into_iter().map(|x| x as f64).collect::<Vec<_>>();
    let result = v
        .into_iter()
        .map(|x| ...)
        // Notice the & sign here. It means that filter() takes a reference, and
        // we are dereferencing it. We'll explore this notion further later on.
        .filter(|&x| ...)
        .collect::<Vec<_>>();
    assert!(result
        .into_iter()
        .all(|x| { x <= 500.0 && x.sqrt().fract() == 0.0 }));

    // Show "cargo examples" command for skipping to the next example.
    grayscott_exercises::show_skip_command!();
}
