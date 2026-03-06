# Exemple

```rust
use runique::migration::utils::paths::table_applied_dir;
assert_eq!(table_applied_dir("migrations", "users"), "migrations/applied/users");
```
