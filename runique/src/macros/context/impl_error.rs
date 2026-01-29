// Factorisation des conversions avec une macro interne simple
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
