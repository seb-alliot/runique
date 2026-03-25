#[macro_export]
macro_rules! search {
    // ============================================================
    // INCLUSION / EXCLUSION (+ / -)
    // ============================================================

    // +Col = val (inclure)
    ($entity:ty => + $col:ident = $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.eq($val))
    }};

    // -Col = val (exclure)
    ($entity:ty => - $col:ident = $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .exclude(<$entity as EntityTrait>::Column::$col.eq($val))
    }};

    // +Col = [v1, v2] (IN)
    ($entity:ty => + $col:ident = [$($val:expr),+ $(,)?]) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let conds = vec![$(<$entity as EntityTrait>::Column::$col.eq($val)),+];
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::any(conds))
    }};

    // -Col = [v1, v2] (NOT IN)
    ($entity:ty => - $col:ident = [$($val:expr),+ $(,)?]) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let conds = vec![$(<$entity as EntityTrait>::Column::$col.eq($val)),+];
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .exclude(Condition::any(conds))
    }};

    // +Col = between(a, b)
    ($entity:ty => + $col:ident = between($start:expr, $end:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::all([
                <$entity as EntityTrait>::Column::$col.gte($start),
                <$entity as EntityTrait>::Column::$col.lte($end),
            ]))
    }};

    // -Col = between(a, b)
    ($entity:ty => - $col:ident = between($start:expr, $end:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::any([
                <$entity as EntityTrait>::Column::$col.lt($start),
                <$entity as EntityTrait>::Column::$col.gt($end),
            ]))
    }};

    // ============================================================
    // OPÉRATEURS DE COMPARAISON (sans + / -)
    // ============================================================

    // >
    ($entity:ty => $col:ident > $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.gt($val))
    }};

    // <
    ($entity:ty => $col:ident < $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.lt($val))
    }};

    // >=
    ($entity:ty => $col:ident >= $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.gte($val))
    }};

    // <=
    ($entity:ty => $col:ident <= $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.lte($val))
    }};

    // LIKE (~)
    ($entity:ty => $col:ident ~ $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.like($val))
    }};

    // NOT LIKE (!~)
    ($entity:ty => $col:ident !~ $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.not_like($val))
    }};

    // IS NULL
    ($entity:ty => $col:ident = null) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_null())
    }};

    // IS NOT NULL
    ($entity:ty => $col:ident != null) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_not_null())
    }};

    // ============================================================
    // MULTIPLES CONDITIONS (mix + / - et opérateurs)
    // ============================================================

    ($entity:ty => $($sign:tt $col:ident = $val:tt),+ $(,)?) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut b = $crate::macros::bdd::objects::Objects::<$entity>::new().all();
        $(
            b = $crate::search_apply!(b, $entity, $sign, $col, $val);
        )+
        b
    }};
}

// Helper interne
#[macro_export]
#[doc(hidden)]
macro_rules! search_apply {
    // +Col = val (inclure)
    ($b:expr, $entity:ty, +, $col:ident, $val:expr) => {
        $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val))
    };

    // -Col = val (exclure)
    ($b:expr, $entity:ty, -, $col:ident, $val:expr) => {
        $b.exclude(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val))
    };

    // +Col = [v1, v2] (IN)
    ($b:expr, $entity:ty, +, $col:ident, [$($val:expr),+]) => {{
        let conds = vec![$(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val)),+];
        $b.filter(sea_orm::Condition::any(conds))
    }};

    // -Col = [v1, v2] (NOT IN)
    ($b:expr, $entity:ty, -, $col:ident, [$($val:expr),+]) => {{
        let conds = vec![$(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val)),+];
        $b.exclude(sea_orm::Condition::any(conds))
    }};

    // +Col = between(a, b)
    ($b:expr, $entity:ty, +, $col:ident, between($start:expr, $end:expr)) => {{
        $b.filter(sea_orm::Condition::all([
            <$entity as sea_orm::EntityTrait>::Column::$col.gte($start),
            <$entity as sea_orm::EntityTrait>::Column::$col.lte($end),
        ]))
    }};

    // -Col = between(a, b)
    ($b:expr, $entity:ty, -, $col:ident, between($start:expr, $end:expr)) => {{
        $b.filter(sea_orm::Condition::any([
            <$entity as sea_orm::EntityTrait>::Column::$col.lt($start),
            <$entity as sea_orm::EntityTrait>::Column::$col.gt($end),
        ]))
    }};
}
