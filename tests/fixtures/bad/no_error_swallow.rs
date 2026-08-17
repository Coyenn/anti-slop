fn swallow(r: Result<i32, &'static str>) {
    match r {
        Ok(_) => {}
        Err(_) => {}
    }
}

fn unit(r: Result<i32, &'static str>) {
    match r {
        Ok(_) => {}
        Err(_) => (),
    }
}

fn let_discard(r: Result<i32, &'static str>) {
    match r {
        Ok(_) => {}
        Err(_) => {
            let _ = 1;
        }
    }
}
