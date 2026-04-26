//! Macro `search_apply_op!` — maps Django operators to SeaORM conditions (eq, icontains, gte…).

// ─────────────────────────────────────────────────────────────────────────────
// search_apply_op! — maps a Django operator name to a SeaORM condition
// ─────────────────────────────────────────────────────────────────────────────

#[macro_export]
#[doc(hidden)]
macro_rules! search_apply_op {
    ($col:expr, eq,         $val:expr) => {
        $col.eq($val)
    };
    ($col:expr, exact,      $val:expr) => {
        $col.eq($val)
    };
    ($col:expr, ne,         $val:expr) => {
        $col.ne($val)
    };
    ($col:expr, gt,         $val:expr) => {
        $col.gt($val)
    };
    ($col:expr, lt,         $val:expr) => {
        $col.lt($val)
    };
    ($col:expr, gte,        $val:expr) => {
        $col.gte($val)
    };
    ($col:expr, lte,        $val:expr) => {
        $col.lte($val)
    };
    ($col:expr, like,       $val:expr) => {
        $col.like($val)
    };
    ($col:expr, ilike,      $val:expr) => {
        $col.ilike($val)
    };
    ($col:expr, not_like,   $val:expr) => {
        $col.not_like($val)
    };
    ($col:expr, not_ilike,  $val:expr) => {
        $col.not_ilike($val)
    };
    ($col:expr, contains,   $val:expr) => {
        $col.like(format!("%{}%", $val))
    };
    ($col:expr, icontains,  $val:expr) => {
        $col.ilike(format!("%{}%", $val))
    };
    ($col:expr, startswith, $val:expr) => {
        $col.like(format!("{}%", $val))
    };
    ($col:expr, endswith,   $val:expr) => {
        $col.like(format!("%{}", $val))
    };
    ($col:expr, iexact,     $val:expr) => {
        $col.ilike($val)
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// search_munch! — TT muncher: iterates through conditions separated by ','
//
// Normalization: the input always receives a final comma added by
// the multi-condition arm of `search!`. Each arm consumes an item + ',' and
// recursively calls itself on the rest.
// ─────────────────────────────────────────────────────────────────────────────

#[macro_export]
#[doc(hidden)]
macro_rules! search_munch {
    // ── Base cases ──────────────────────────────────────────────────────────
    ($b:expr, $entity:ty;) => {};
    ($b:expr, $entity:ty; ,) => {};

    // ── join Relation — INNER JOIN ────────────────────────────────
    ($b:expr, $entity:ty; join $rel:ident , $($rest:tt)*) => {
        {
            use sea_orm::RelationTrait;
            $b = $b.join(
                <$entity as sea_orm::EntityTrait>::Relation::$rel.def()
            );
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── left_join Relation — LEFT JOIN ───────────────────────────
    ($b:expr, $entity:ty; left_join $rel:ident , $($rest:tt)*) => {
        {
            use sea_orm::RelationTrait;
            $b = $b.left_join(
                <$entity as sea_orm::EntityTrait>::Relation::$rel.def()
            );
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── mod::Col op val — filter on relation ─────────────────────
    ($b:expr, $entity:ty; $mod:ident :: $col:ident $op:ident $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(
                $crate::search_apply_op!($mod::Column::$col, $op, $val)
            );
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };
    // ── Col isnull ────────────────────────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident isnull , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.is_null());
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col not_null ──────────────────────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident not_null , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.is_not_null());
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col in [v1, v2] — Literal IN ────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident in [$($val:expr),+ $(,)?] , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let conds = vec![$(<$entity as sea_orm::EntityTrait>::Column::$col.eq($val)),+];
            $b = $b.filter(sea_orm::Condition::any(conds));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col not_in [v1, v2] — Literal NOT IN ────────────────────────────────
    ($b:expr, $entity:ty; $col:ident not_in [$($val:expr),+ $(,)?] , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(
                <$entity as sea_orm::EntityTrait>::Column::$col.is_not_in(vec![$($val),+])
            );
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col in (expr) — Dynamic IN ─────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident in ($val:expr) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.is_in($val));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col not_in (expr) — Dynamic NOT IN ─────────────────────────────────
    ($b:expr, $entity:ty; $col:ident not_in ($val:expr) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(<$entity as sea_orm::EntityTrait>::Column::$col.is_not_in($val));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col range (a, b) — BETWEEN ───────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident range ($start:expr, $end:expr) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(sea_orm::Condition::all([
                <$entity as sea_orm::EntityTrait>::Column::$col.gte($start),
                <$entity as sea_orm::EntityTrait>::Column::$col.lte($end),
            ]));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col not_range (a, b) — NOT BETWEEN ───────────────────────────────────
    ($b:expr, $entity:ty; $col:ident not_range ($start:expr, $end:expr) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(sea_orm::Condition::any([
                <$entity as sea_orm::EntityTrait>::Column::$col.lt($start),
                <$entity as sea_orm::EntityTrait>::Column::$col.gt($end),
            ]));
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── or(Col1 op val, Col2 op val) — multi-column OR ─────────────────────
    ($b:expr, $entity:ty; or($($col:ident $op:ident $val:expr),+ $(,)?) , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            let mut cond = sea_orm::Condition::any();
            $(
                cond = cond.add(
                    $crate::search_apply_op!(<$entity as sea_orm::EntityTrait>::Column::$col, $op, $val)
                );
            )+
            $b = $b.filter(cond);
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── ! Col op val — exclusion ──────────────────────────────────────────────
    ($b:expr, $entity:ty; ! $col:ident $op:ident $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.exclude(
                $crate::search_apply_op!(<$entity as sea_orm::EntityTrait>::Column::$col, $op, $val)
            );
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── asc Col / desc Col — sorting ──────────────────────────────────────────────
    ($b:expr, $entity:ty; asc $col:ident , $($rest:tt)*) => {
        $b = $b.order_by_asc(<$entity as sea_orm::EntityTrait>::Column::$col);
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    ($b:expr, $entity:ty; desc $col:ident , $($rest:tt)*) => {
        $b = $b.order_by_desc(<$entity as sea_orm::EntityTrait>::Column::$col);
        $crate::search_munch!($b, $entity; $($rest)*);
    };

    // ── Col op val — general filter ───────────────────────────────────────────
    ($b:expr, $entity:ty; $col:ident $op:ident $val:expr , $($rest:tt)*) => {
        {
            use sea_orm::ColumnTrait;
            $b = $b.filter(
                $crate::search_apply_op!(<$entity as sea_orm::EntityTrait>::Column::$col, $op, $val)
            );
        }
        $crate::search_munch!($b, $entity; $($rest)*);
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// search! — SeaORM filtering DSL, Django syntax
//
// Available operators (Col op val) :
//   eq, exact, ne, gt, lt, gte, lte
//   like, ilike, not_like, not_ilike
//   contains, icontains, startswith, endswith, iexact
//
// Special forms :
//   Col isnull           → IS NULL
//   Col not_null         → IS NOT NULL
//   Col in [v1, v2]      → IN (literal)
//   Col in (expr)        → IN (dynamic)
//   Col not_in [v1, v2]  → NOT IN (literal)
//   Col not_in (expr)    → NOT IN (dynamic)
//   Col range (a, b)     → BETWEEN a AND b
//   Col not_range (a, b) → NOT BETWEEN
//   or(C1 op v, C2 op v) → multi-column OR
//   ! Col op val         → exclusion (NOT)
//   search!(Entity)      → fetch all without filter
// ─────────────────────────────────────────────────────────────────────────────

#[macro_export]
macro_rules! search {
    // ── @Form => ... (delegates to the associated Entity type via FormEntity) ──────────
    (@ $form:ty => $($tt:tt)+) => {{
        $crate::search!(<$form as $crate::forms::FormEntity>::Entity => $($tt)+)
    }};

    // ── Fetch all ─────────────────────────────────────────────────────────────
    ($entity:ty) => {{
        $crate::macros::bdd::objects::Objects::<$entity>::new().all()
    }};

    // ── Col isnull ────────────────────────────────────────────────────────────
    ($entity:ty => $col:ident isnull) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_null())
    }};

    // ── Col not_null ──────────────────────────────────────────────────────────
    ($entity:ty => $col:ident not_null) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_not_null())
    }};

    // ── Col in [v1, v2] — Literal IN ────────────────────────────────────────
    ($entity:ty => $col:ident in [$($val:expr),+ $(,)?]) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let conds = vec![$(<$entity as EntityTrait>::Column::$col.eq($val)),+];
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::any(conds))
    }};

    // ── Col not_in [v1, v2] — Literal NOT IN ────────────────────────────────
    ($entity:ty => $col:ident not_in [$($val:expr),+ $(,)?]) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_not_in(vec![$($val),+]))
    }};

    // ── Col in (expr) — Dynamic IN ─────────────────────────────────────────
    ($entity:ty => $col:ident in ($val:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_in($val))
    }};

    // ── Col not_in (expr) — Dynamic NOT IN ─────────────────────────────────
    ($entity:ty => $col:ident not_in ($val:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(<$entity as EntityTrait>::Column::$col.is_not_in($val))
    }};

    // ── Col range (a, b) — BETWEEN ───────────────────────────────────────────
    ($entity:ty => $col:ident range ($start:expr, $end:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::all([
                <$entity as EntityTrait>::Column::$col.gte($start),
                <$entity as EntityTrait>::Column::$col.lte($end),
            ]))
    }};

    // ── Col not_range (a, b) — NOT BETWEEN ───────────────────────────────────
    ($entity:ty => $col:ident not_range ($start:expr, $end:expr)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter(Condition::any([
                <$entity as EntityTrait>::Column::$col.lt($start),
                <$entity as EntityTrait>::Column::$col.gt($end),
            ]))
    }};

    // ── or(Col1 op val, Col2 op val) — multi-column OR ─────────────────────
    ($entity:ty => or($($col:ident $op:ident $val:expr),+ $(,)?)) => {{
        use sea_orm::{EntityTrait, ColumnTrait, Condition};
        let mut cond = Condition::any();
        $(
            cond = cond.add(
                $crate::search_apply_op!(<$entity as EntityTrait>::Column::$col, $op, $val)
            );
        )+
        $crate::macros::bdd::objects::Objects::<$entity>::new().filter(cond)
    }};

    // ── ! Col op val — exclusion ──────────────────────────────────────────────
    ($entity:ty => ! $col:ident $op:ident $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .exclude($crate::search_apply_op!(<$entity as EntityTrait>::Column::$col, $op, $val))
    }};

    // ── Col op val — single condition ─────────────────────────────────────────
    ($entity:ty => $col:ident $op:ident $val:expr) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        $crate::macros::bdd::objects::Objects::<$entity>::new()
            .filter($crate::search_apply_op!(<$entity as EntityTrait>::Column::$col, $op, $val))
    }};

    // ── Multi-conditions (TT muncher) ─────────────────────────────────────────
    // Captures all the rest. Adds a final comma to normalize the muncher.
    ($entity:ty => $($tt:tt)+) => {{
        use sea_orm::{EntityTrait, ColumnTrait};
        let mut b = $crate::macros::bdd::objects::Objects::<$entity>::new().all();
        $crate::search_munch!(b, $entity; $($tt)+ ,);
        b
    }};
}
