# Exemple

```rust
use runique::migration::utils::paths::snapshot_file_path;
assert_eq!(snapshot_file_path("migrations", "users"), "migrations/snapshots/users.rs");
```
