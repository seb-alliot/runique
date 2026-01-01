// rusti/src/macro_perso/context_macro/context_macro.rs

#[macro_export]
macro_rules! context {

    () => {
        $crate::macro_perso::context_macro::helper_context::ContextHelper::new()
    };

    { $($key:expr, $value:expr);* $(;)? } => {{
        let mut ctx = $crate::macro_perso::context_macro::helper_context::ContextHelper::new();
        $(
            ctx = ctx.add($key, $value);
        )*
        ctx
    }};

    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut ctx = $crate::macro_perso::context_macro::helper_context::ContextHelper::new();
        $(
            ctx = ctx.add($key, $value);
        )*
        ctx
    }};
}
