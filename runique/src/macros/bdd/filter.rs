#[macro_export]
macro_rules! search {
    // 1 condition positive
    ($entity:ty => + $col:ident = $val:expr) => {{
        use sea_orm::EntityTrait;
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.eq($val))
    }};

    // 1 condition négative
    ($entity:ty => - $col:ident = $val:expr) => {{
        use sea_orm::EntityTrait;
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .exclude(<$entity as EntityTrait>::Column::$col.eq($val))
    }};

    // Multiple conditions
    ($entity:ty => $($sign:tt $col:ident = $val:expr),+ $(,)?) => {{
        use sea_orm::EntityTrait;
        let mut b = $crate::macros::bdd::objects::Objects::<$entity>::new().all();
        $(
            b = match $sign {
                + => b.filter(<$entity as EntityTrait>::Column::$col.eq($val)),
                - => b.exclude(<$entity as EntityTrait>::Column::$col.eq($val)),
                _ => b,
            };
        )+
        b
    }};
}
