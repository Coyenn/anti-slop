fn a(x: i32) -> i32 {
    // SAFETY: trust me
    x
}

fn b(x: i32) -> i32 {
    // SAFETY:
    x
}

fn c(x: i32) -> i32 {
    // SAFETY: it's fine
    x
}
