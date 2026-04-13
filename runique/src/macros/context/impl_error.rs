//! Macro `impl_from_error!` — generates `From<Err>` to `AppError` implementations in a single line.

// Factorization of conversions with a simple internal macro
#[macro_export]
macro_rules! impl_from_error {
    ($($err:ty => $method:ident),*) => {
        $(
            impl From<$err> for AppError {
                fn from(err: $err) -> Self { Self { context: ErrorContext::$method(&err) } }
            }
            impl From<$err> for Box<AppError> {
                fn from(err: $err) -> Self { Box::new(AppError::from(err)) }
            }
        )*
    };
}
