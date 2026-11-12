# Exemple

```rust
use runique::utils::forms::sanitizer::sanitize_strict;
assert_eq!(sanitize_strict("<b>test</b>"), "test");
// Text stays as typed: only tags go. A link's value comes from `TextField::url`.
assert_eq!(sanitize_strict("data: 2024 <i>final</i>"), "data: 2024 final");
```
