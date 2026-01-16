pub mod parser;
pub mod transaction;
pub mod transport;

pub trait TestContext<T>: Sized {
    fn setup(args: T) -> Self {
        unimplemented!()
    }
    async fn setup_async(args: T) -> Self {
        unimplemented!()
    }
}
