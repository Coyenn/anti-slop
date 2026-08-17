fn id() -> std::any::TypeId {
    std::any::TypeId::of::<u32>()
}

fn name() -> &'static str {
    std::any::type_name::<u32>()
}

fn of_val(x: &u32) -> &'static str {
    std::any::type_name_of_val(x)
}

fn down(x: &Dummy) -> Option<&u32> {
    x.downcast_ref::<u32>()
}

struct Dummy;
