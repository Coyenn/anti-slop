fn branded(x: u32) -> u32 {
    // SAFETY: parse_lane already checked x against the table length at the I/O boundary.
    x
}
