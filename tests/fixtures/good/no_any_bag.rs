enum Payload {
    Text(String),
    Count(u64),
}

fn take(value: Payload) -> Payload {
    value
}

struct Owner {
    inner: Payload,
}
