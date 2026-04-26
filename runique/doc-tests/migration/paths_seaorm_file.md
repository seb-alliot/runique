# Exemple

```rust
use runique::migration::utils::paths::seaorm_create_file_path;
assert_eq!(seaorm_create_file_path("migrations", "20260219", "blog"), "migrations/m20260219_create_blog_table.rs");
```
