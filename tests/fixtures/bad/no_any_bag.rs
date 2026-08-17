fn take(value: Box<dyn std::any::Any>) {}

fn give() -> std::sync::Arc<dyn std::any::Any> {
    loop {}
}

struct Bag {
    inner: std::rc::Rc<dyn std::any::Any>,
}

type Alias = dyn std::any::Any;
