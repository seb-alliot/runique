#[macro_export]
macro_rules! context {

    () => {
        $crate::macros::helper::ContextHelper::new()
    };

    { $($key:expr, $value:expr);* $(;)? } => {{
        let mut ctx = $crate::macros::helper::ContextHelper::new();
        $(
            ctx = ctx.add($key, $value);
        )*
        ctx
    }};

    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut ctx = $crate::macros::helper::ContextHelper::new();
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
