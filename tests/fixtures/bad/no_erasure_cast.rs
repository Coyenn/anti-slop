fn widen(x: u64) -> usize {
    x as _
}

fn launder(p: *const u8) -> *const i32 {
    p as *const u8 as *const i32
}
