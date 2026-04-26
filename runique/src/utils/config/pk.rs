//! User primary key type — `i32` by default, `i64` with the `big-pk` feature.
/// User primary key type.
///
/// Defaults to `i32`. Enable the `big-pk` feature to switch to `i64`:
///
/// ```toml
/// runique = { version = "...", features = ["big-pk"] }
/// ```
#[cfg(feature = "big-pk")]
pub type Pk = i64;

#[cfg(not(feature = "big-pk"))]
pub type Pk = i32;
