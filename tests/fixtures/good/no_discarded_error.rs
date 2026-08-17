fn keep(r: Result<i32, &'static str>) -> Result<i32, &'static str> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => Err(e),
    }
}

fn propagate(r: Result<i32, &'static str>) -> Result<i32, &'static str> {
    let v = r?;
    Ok(v)
}
