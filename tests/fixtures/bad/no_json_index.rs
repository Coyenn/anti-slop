fn silent_null() {
    let _ = serde_json::Value[0]["k"];
}

fn chained(val: serde_json::Value) {
    let _ = val[0]["k"];
}
