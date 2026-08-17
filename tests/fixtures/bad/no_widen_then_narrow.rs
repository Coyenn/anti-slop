fn trunc(x: u8) -> u8 {
    x as u32 as u8
}

fn via_try(x: u64) -> u32 {
    (x as u64).try_into().unwrap()
}

fn ptr_unknown(x: u64) -> u32 {
    x as usize as u32
}
