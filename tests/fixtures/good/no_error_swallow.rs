fn handle(r: Result<i32, &'static str>) -> Result<i32, &'static str> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => Err(e),
    }
}

fn named(r: Result<i32, &'static str>) {
    match r {
        Ok(_) => {}
        Err(e) => {
            let _err = e;
        }
    }
}
