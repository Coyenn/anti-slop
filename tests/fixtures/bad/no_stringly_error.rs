fn as_string() -> Result<u32, String> {
    Ok(1)
}

fn as_str() -> Result<u32, &'static str> {
    Ok(1)
}

fn as_cow() -> Result<u32, std::borrow::Cow<'static, str>> {
    Ok(1)
}
