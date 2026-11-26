# Exemple

```rust
use runique_cli::migration::{ColumnType, ParsedColumn, col_type_method};

let plain = ParsedColumn {
    col_type: ColumnType::Text,
    ..Default::default()
};
assert_eq!(col_type_method(&plain), "text()");

let sized = ParsedColumn {
    col_type: ColumnType::String,
    max_length: Some(100),
    ..Default::default()
};
assert_eq!(col_type_method(&sized), "string_len(100)");
```
