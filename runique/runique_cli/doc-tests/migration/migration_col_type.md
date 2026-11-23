# Exemple

```rust
use runique_cli::migration::col_type_to_method;
assert_eq!(col_type_to_method("Text"), "text()");
assert_eq!(col_type_to_method("TinyInteger"), "tiny_integer()");
assert_eq!(col_type_to_method("Inconnu"), "string()");
```
