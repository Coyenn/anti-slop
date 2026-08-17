trait Store {
    fn get(&self) -> u32;
}

struct MemoryStore {
    value: u32,
}

impl Store for MemoryStore {
    fn get(&self) -> u32 {
        self.value
    }
}

fn use_store(store: &dyn Store) -> u32 {
    store.get()
}
