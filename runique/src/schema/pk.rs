//! User primary key type — `i32` by default, `i64` with `big-pk`, `Uuid` (v7) with `pk-uuid`.

#[cfg(all(feature = "big-pk", feature = "pk-uuid"))]
compile_error!(
    "features `big-pk` and `pk-uuid` are mutually exclusive — enable at most one of them"
);

/// User primary key type.
///
/// Defaults to `i32`. Enable `big-pk` to switch to `i64`, or `pk-uuid` to switch to
/// `uuid::Uuid` (generated as UUIDv7 — time-ordered, avoids B-tree index fragmentation):
///
/// ```toml
/// runique = { version = "...", features = ["big-pk"] }       # i64
/// runique = { version = "...", features = ["pk-uuid"] }      # Uuid
/// ```
#[cfg(feature = "pk-uuid")]
pub type Pk = uuid::Uuid;

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
pub type Pk = i64;

#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
pub type Pk = i32;
