pub mod helper_context;

pub use helper_context::ContextHelper;
// runique/src/macro_runique/context_macro/context_macro.rs

#[macro_export]
macro_rules! context {

    () => {
        $crate::macro_runique::context_macro::helper_context::ContextHelper::new()
    };

    { $($key:expr, $value:expr);* $(;)? } => {{
        let mut ctx = $crate::macro_runique::context_macro::helper_context::ContextHelper::new();
        $(
            ctx = ctx.add($key, $value);
        )*
        ctx
    }};

    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut ctx = $crate::macro_runique::context_macro::helper_context::ContextHelper::new();
        $(
            ctx = ctx.add($key, $value);
        )*
        ctx
    }};
}

#[macro_export]
macro_rules! context_update {
    ($($key:expr => $value:expr),* $(,)?) => {
        vec![
            $( ($key, serde_json::json!($value)) ),*
        ]
    };
}
