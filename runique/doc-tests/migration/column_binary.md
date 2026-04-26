# Exemples

```rust
use runique::migration::column::ColumnDef;
// Default: BINARY(255)
ColumnDef::new("token").binary();

// Override: BINARY(64)
ColumnDef::new("hash").binary_len(64);
```
