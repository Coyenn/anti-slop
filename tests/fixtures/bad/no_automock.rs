#[automock]
trait Store {
    fn get(&self) -> u32;
}

mockall::mock! {
    pub Foo {
        fn bar(&self) -> u32;
    }
}

fn test_double() {
    mock! {
        pub Bar {}
    }
}
