enum Kind {
    A,
    B,
}

fn branch(kind: Kind) -> u32 {
    match kind {
        Kind::A => 1,
        Kind::B => 2,
    }
}
