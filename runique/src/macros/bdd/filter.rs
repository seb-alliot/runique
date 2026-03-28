// ─────────────────────────────────────────────────────────────────────────────
// search_op! — applique un opérateur SeaORM sur une colonne et une valeur
//
// Note : ~~ (ILIKE) et !~~ (NOT ILIKE) sont deux tokens chacun et ne peuvent
// pas être passés comme $op:tt — ils sont gérés par des bras dédiés.
// ─────────────────────────────────────────────────────────────────────────────

#[macro_export]
#[doc(hidden)]
macro_rules! search_op {
    ($col:expr, =,  $val:expr) => {
        $col.eq($val)
    };
    ($col:expr, !=, $val:expr) => {
        $col.ne($val)
    };
    ($col:expr, >,  $val:expr) => {
        $col.gt($val)
    };
    ($col:expr, <,  $val:expr) => {
        $col.lt($val)
    };
    ($col:expr, >=, $val:expr) => {
        $col.gte($val)
    };
    ($col:expr, <=, $val:expr) => {
        $col.lte($val)
    };
    ($col:expr, ~,  $val:expr) => {
        $col.like($val)
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// search_munch! — TT muncher : parcourt les conditions séparées par ','
//
// Normalisation : l'entrée reçoit toujours une virgule finale ajoutée par
// l'arm multi-condition de `search!`. Chaque arm consomme un item + ',' et
// rappelle récursivement sur le reste.
// ─────────────────────────────────────────────────────────────────────────────

#[macro_export]
#[doc(hidden)]
macro_rules! search_munch {
    // ── Cas de base ──────────────────────────────────────────────────────────
    ($b:expr, $entity:ty;) => {};
    ($b:expr, $entity:ty; ,) => {};

    // ── +Col = [v1, v2] (IN) ─────────────────────────────────────────────────
    ($b:expr, $entity:ty; + $col:ident = [$($val:expr),+ $(,)?] , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let conds = vec![$(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val)),+];
            $b = $b.filter(sea_orm::Condition::any(conds));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── -Col = [v1, v2] (NOT IN) ──────────────────────────────────────────────
    ($b:expr, $entity:ty; - $col:ident = [$($val:expr),+ $(,)?] , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let conds = vec![$(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val)),+];
            $b = $b.exclude(sea_orm::Condition::any(conds));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── +Col = between(a, b) ─────────────────────────────────────────────────
    ($b:expr, $entity:ty; + $col:ident = between($start:expr, $end:expr) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(sea_orm::Condition::all([
                <$entity as sea_orm::EntityTrait>::Column::$col.gte($start),
                <$entity as sea_orm::EntityTrait>::Column::$col.lte($end),
            ]));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── -Col = between(a, b) (NOT BETWEEN) ───────────────────────────────────
    ($b:expr, $entity:ty; - $col:ident = between($start:expr, $end:expr) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(sea_orm::Condition::any([
                <$entity as sea_orm::EntityTrait>::Column::$col.lt($start),
                <$entity as sea_orm::EntityTrait>::Column::$col.gt($end),
            ]));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col = null ────────────────────────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident = null , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.is_null());
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col != null ───────────────────────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident != null , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.is_not_null());
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── +Col = val ────────────────────────────────────────────────────────────
    ($b:expr, $entity:ty; + $col:ident = $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── -Col = val ────────────────────────────────────────────────────────────
    ($b:expr, $entity:ty; - $col:ident = $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.exclude(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── [(Col1 op v1, Col2 op v2) | (Col3 op v3)] — OR de groupes AND ────────
    ($b:expr, $entity:ty; [ $( ($($col:ident $op:tt $val:expr),+ $(,)?) )|+ ] , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut outer = sea_orm::Condition::any();
            $(
                let mut inner = sea_orm::Condition::all();
                $(
                    inner = inner.add(
                        $crate::search_op!(<$entity as sea_orm::EntityTrait>::Column::$col, $op, $val)
                    );
                )+
                outer = outer.add(inner);
            )+
            $b = $b.filter(outer);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── (Col1 ~~ v1 | Col2 ~~ v2) — OR ILIKE multi-colonnes ──────────────────
    ($b:expr, $entity:ty; ($($col:ident ~ ~ $val:tt)|+) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut cond = sea_orm::Condition::any();
            $(
                cond = cond.add(<$entity as sea_orm::EntityTrait>::Column::$col.ilike($val));
            )+
            $b = $b.filter(cond);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── (Col1 !~~ v1 | Col2 !~~ v2) — OR NOT ILIKE multi-colonnes ────────────
    ($b:expr, $entity:ty; ($($col:ident ! ~ ~ $val:tt)|+) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut cond = sea_orm::Condition::any();
            $(
                cond = cond.add(<$entity as sea_orm::EntityTrait>::Column::$col.not_ilike($val));
            )+
            $b = $b.filter(cond);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── (Col1 op v1 | Col2 op v2) — OR multi-colonnes ────────────────────────
    ($b:expr, $entity:ty; ($($col:ident $op:tt $val:tt)|+) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut cond = sea_orm::Condition::any();
            $(
                cond = cond.add(
                    $crate::search_op!(<$entity as sea_orm::EntityTrait>::Column::$col, $op, $val)
                );
            )+
            $b = $b.filter(cond);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col ~~ (v1 | v2) — OR ILIKE même colonne ─────────────────────────────
    ($b:expr, $entity:ty; $col:ident ~ ~ ($($val:tt)|+) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut cond = sea_orm::Condition::any();
            $(
                cond = cond.add(<$entity as sea_orm::EntityTrait>::Column::$col.ilike($val));
            )+
            $b = $b.filter(cond);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col !~~ (v1 | v2) — OR NOT ILIKE même colonne ────────────────────────
    ($b:expr, $entity:ty; $col:ident ! ~ ~ ($($val:tt)|+) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut cond = sea_orm::Condition::any();
            $(
                cond = cond.add(<$entity as sea_orm::EntityTrait>::Column::$col.not_ilike($val));
            )+
            $b = $b.filter(cond);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col op (v1 | v2 | ...) — OR même colonne ─────────────────────────────
    ($b:expr, $entity:ty; $col:ident $op:tt ($($val:tt)|+) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut cond = sea_orm::Condition::any();
            $(
                cond = cond.add(
                    $crate::search_op!(<$entity as sea_orm::EntityTrait>::Column::$col, $op, $val)
                );
            )+
            $b = $b.filter(cond);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col !~~ val (NOT ILIKE — trois tokens) ────────────────────────────────
    ($b:expr, $entity:ty; $col:ident ! ~ ~ $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.not_ilike($val));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col ~~ val (ILIKE — deux tokens) ─────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident ~ ~ $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.ilike($val));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col !~ val (NOT LIKE — deux tokens) ───────────────────────────────────
    ($b:expr, $entity:ty; $col:ident ! ~ $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.not_like($val));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col op val — comparaison générale ─────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident $op:tt $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(
                $crate::search_op!(<$entity as sea_orm::EntityTrait>::Column::$col, $op, $val)
            );
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// search! — DSL de filtrage SeaORM
// ─────────────────────────────────────────────────────────────────────────────

#[macro_export]
macro_rules! search {
    // ── @Form => ... (délègue au type Entity associé via FormEntity) ──────────
    (@ $form:ty => $($tt:tt)+) => {{
        $crate::search!(<$form as $crate::forms::FormEntity>::Entity => $($tt)+)
    }};

    // ── +Col = val ────────────────────────────────────────────────────────────
    ($entity:ty => + $col:ident = $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.eq($val))
    }};

    // ── -Col = val ────────────────────────────────────────────────────────────
    ($entity:ty => - $col:ident = $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .exclude(<$entity as EntityTrait>::Column::$col.eq($val))
    }};

    // ── +Col = [v1, v2] (IN) ─────────────────────────────────────────────────
    ($entity:ty => + $col:ident = [$($val:expr),+ $(,)?]) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let conds = vec![$(<$entity as EntityTrait>::Column::$col.eq($val)),+];
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::any(conds))
    }};

    // ── -Col = [v1, v2] (NOT IN) ──────────────────────────────────────────────
    ($entity:ty => - $col:ident = [$($val:expr),+ $(,)?]) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let conds = vec![$(<$entity as EntityTrait>::Column::$col.eq($val)),+];
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .exclude(Condition::any(conds))
    }};

    // ── +Col = between(a, b) ─────────────────────────────────────────────────
    ($entity:ty => + $col:ident = between($start:expr, $end:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::all([
                <$entity as EntityTrait>::Column::$col.gte($start),
                <$entity as EntityTrait>::Column::$col.lte($end),
            ]))
    }};

    // ── -Col = between(a, b) (NOT BETWEEN) ───────────────────────────────────
    ($entity:ty => - $col:ident = between($start:expr, $end:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::any([
                <$entity as EntityTrait>::Column::$col.lt($start),
                <$entity as EntityTrait>::Column::$col.gt($end),
            ]))
    }};

    // ── Col > val ─────────────────────────────────────────────────────────────
    ($entity:ty => $col:ident > $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.gt($val))
    }};

    // ── Col < val ─────────────────────────────────────────────────────────────
    ($entity:ty => $col:ident < $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.lt($val))
    }};

    // ── Col >= val ────────────────────────────────────────────────────────────
    ($entity:ty => $col:ident >= $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.gte($val))
    }};

    // ── Col <= val ────────────────────────────────────────────────────────────
    ($entity:ty => $col:ident <= $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.lte($val))
    }};

    // ── Col ~ val (LIKE) ──────────────────────────────────────────────────────
    ($entity:ty => $col:ident ~ $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.like($val))
    }};

    // ── Col !~ val (NOT LIKE) ─────────────────────────────────────────────────
    ($entity:ty => $col:ident ! ~ $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.not_like($val))
    }};

    // ── Col ~~ val (ILIKE) ────────────────────────────────────────────────────
    ($entity:ty => $col:ident ~ ~ $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.ilike($val))
    }};

    // ── Col !~~ val (NOT ILIKE) ───────────────────────────────────────────────
    ($entity:ty => $col:ident ! ~ ~ $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.not_ilike($val))
    }};

    // ── Col = null ────────────────────────────────────────────────────────────
    ($entity:ty => $col:ident = null) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_null())
    }};

    // ── Col != null ───────────────────────────────────────────────────────────
    ($entity:ty => $col:ident != null) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_not_null())
    }};

    // ── Col ~~ (v1 | v2) — OR ILIKE même colonne ─────────────────────────────
    ($entity:ty => $col:ident ~ ~ ($($val:tt)|+)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut cond = Condition::any();
        $(
            cond = cond.add(<$entity as EntityTrait>::Column::$col.ilike($val));
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(cond)
    }};

    // ── Col !~~ (v1 | v2) — OR NOT ILIKE même colonne ────────────────────────
    ($entity:ty => $col:ident ! ~ ~ ($($val:tt)|+)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut cond = Condition::any();
        $(
            cond = cond.add(<$entity as EntityTrait>::Column::$col.not_ilike($val));
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(cond)
    }};

    // ── Col op (v1 | v2 | ...) — OR même colonne ─────────────────────────────
    ($entity:ty => $col:ident $op:tt ($($val:tt)|+)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut cond = Condition::any();
        $(
            cond = cond.add(
                $crate::search_op!(<$entity as EntityTrait>::Column::$col, $op, $val)
            );
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(cond)
    }};

    // ── (Col1 ~~ v1 | Col2 ~~ v2) — OR ILIKE multi-colonnes ──────────────────
    ($entity:ty => ($($col:ident ~ ~ $val:tt)|+)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut cond = Condition::any();
        $(
            cond = cond.add(<$entity as EntityTrait>::Column::$col.ilike($val));
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(cond)
    }};

    // ── (Col1 !~~ v1 | Col2 !~~ v2) — OR NOT ILIKE multi-colonnes ────────────
    ($entity:ty => ($($col:ident ! ~ ~ $val:tt)|+)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut cond = Condition::any();
        $(
            cond = cond.add(<$entity as EntityTrait>::Column::$col.not_ilike($val));
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(cond)
    }};

    // ── (Col1 op v1 | Col2 op v2) — OR multi-colonnes ────────────────────────
    ($entity:ty => ($($col:ident $op:tt $val:tt)|+)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut cond = Condition::any();
        $(
            cond = cond.add(
                $crate::search_op!(<$entity as EntityTrait>::Column::$col, $op, $val)
            );
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(cond)
    }};

    // ── [(Col1 op v1, Col2 op v2) | (Col3 op v3)] — OR de groupes AND ────────
    ($entity:ty => [ $( ($($col:ident $op:tt $val:expr),+ $(,)?) )|+ ]) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut outer = Condition::any();
        $(
            let mut inner = Condition::all();
            $(
                inner = inner.add(
                    $crate::search_op!(<$entity as EntityTrait>::Column::$col, $op, $val)
                );
            )+
            outer = outer.add(inner);
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(outer)
    }};

    // ── Multi-conditions (TT muncher) ─────────────────────────────────────────
    // Capture tout le reste. Ajoute une virgule finale pour normaliser le muncher.
    ($entity:ty => $($tt:tt)+) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        let mut b = $crate::macros::bdd::objects::Objects::<$entity>::new().all();
        $crate::search_munch!(b, $entity; $($tt)+ ,);
        b
    }};
}

// ─────────────────────────────────────────────────────────────────────────────
// search_apply! — conservé pour compatibilité (utilisé dans des contextes
// externes éventuels). Préférer search_munch! pour les nouveaux usages.
// ─────────────────────────────────────────────────────────────────────────────

#[macro_export]
#[doc(hidden)]
macro_rules! search_apply {
    ($b:expr, $entity:ty, +, $col:ident, $val:expr) => {
        $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val))
    };
    ($b:expr, $entity:ty, -, $col:ident, $val:expr) => {
        $b.exclude(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val))
    };
    ($b:expr, $entity:ty, +, $col:ident, [$($val:expr),+]) => {{
        let conds = vec![$(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val)),+];
        $b.filter(sea_orm::Condition::any(conds))
    }};
    ($b:expr, $entity:ty, -, $col:ident, [$($val:expr),+]) => {{
        let conds = vec![$(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val)),+];
        $b.exclude(sea_orm::Condition::any(conds))
    }};
    ($b:expr, $entity:ty, +, $col:ident, between($start:expr, $end:expr)) => {{
        $b.filter(sea_orm::Condition::all([
            <$entity as sea_orm::EntityTrait>::Column::$col.gte($start),
            <$entity as sea_orm::EntityTrait>::Column::$col.lte($end),
        ]))
    }};
    ($b:expr, $entity:ty, -, $col:ident, between($start:expr, $end:expr)) => {{
        $b.filter(sea_orm::Condition::any([
            <$entity as sea_orm::EntityTrait>::Column::$col.lt($start),
            <$entity as sea_orm::EntityTrait>::Column::$col.gt($end),
        ]))
    }};
}
