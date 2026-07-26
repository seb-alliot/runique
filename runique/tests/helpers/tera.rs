//! Helpers for calling Tera 2 filters/functions directly from unit tests.
//!
//! Tera 2 hands filters and functions a `Kwargs` and a `&State` instead of a plain
//! `HashMap`. Both are cheap to build but verbose, so the scaffolding lives here
//! rather than being copy-pasted into every test.
//!
//! ```rust
//! let ctx = Context::new();
//! let state = State::new(&ctx);            // must be a local: State borrows it
//! let args = kwargs([("link", Value::from("index"))]);
//! let url = LinkFunction { url_registry }.call(args, &state).unwrap();
//! ```

use std::sync::Arc;
use tera::value::{Key, Map};
use tera::{Kwargs, Value};

/// Builds a `Kwargs` from `(name, value)` pairs.
pub fn kwargs<const N: usize>(pairs: [(&'static str, Value); N]) -> Kwargs {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert(Key::from(key), value);
    }
    Kwargs::new(Arc::new(map))
}

/// Builds an empty `Kwargs` — for filters and functions called without arguments.
pub fn no_kwargs() -> Kwargs {
    Kwargs::new(Arc::new(Map::new()))
}
