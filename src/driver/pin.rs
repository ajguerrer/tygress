// Taken from https://docs.rs/tokio/latest/tokio/macro.pin.html, this allows polling of a `async fn`
// pinning the future to the stack.
#[macro_export]
macro_rules! pin {
    ($($x:ident),*) => { $(
        // Move the value to ensure that it is owned
        let mut $x = $x;
        // Shadow the original binding so that it can't be directly accessed ever again.
        #[allow(unused_mut, unsafe_code)]
        let mut $x = unsafe {
            ::core::pin::Pin::new_unchecked(&mut $x)
        };
    )* };
}
