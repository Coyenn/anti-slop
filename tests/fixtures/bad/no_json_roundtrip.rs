fn nested(x: u32) {
    let _y = serde_json::from_value(serde_json::to_value(x));
}

fn bound(x: u32) {
    let tmp = serde_json::to_value(x);
    let _y = serde_json::from_value(tmp);
}
