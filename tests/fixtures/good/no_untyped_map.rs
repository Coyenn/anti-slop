use std::collections::HashMap;

struct UserId(u64);
struct User;

fn take(map: HashMap<UserId, User>) -> HashMap<UserId, User> {
    map
}
