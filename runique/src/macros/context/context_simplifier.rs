//! `context!` macro — ergonomic constructor for `ContextBuilder` (Tera context).

#[macro_export]
macro_rules! context {

    () => {
        $crate::macros::helper::ContextBuilder::new()
    };

    { $($key:expr, $value:expr);* $(;)? } => {{
        let mut ctx = $crate::macros::helper::ContextBuilder::new();
        $(
            ctx = ctx.add($key, $value);
        )*
        ctx
    }};

    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut ctx = $crate::macros::helper::ContextBuilder::new();
        $(
            ctx = ctx.add($key, $value);
        )*
        ctx
    }};
}

#[macro_export]
macro_rules! context_update {
    ($template:expr => { $($key:expr => $value:expr),* $(,)? }) => {{
        $(
            $template.context.insert($key, &$value);
        )*
    }};
}
