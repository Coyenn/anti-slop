fn boxed() -> Result<u32, Box<dyn std::error::Error>> {
    Ok(1)
}

fn any() -> anyhow::Error {
    loop {}
}

fn report() -> eyre::Report {
    loop {}
}
