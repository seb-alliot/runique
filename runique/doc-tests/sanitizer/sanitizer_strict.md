# Exemple

```rust
use runique::utils::forms::sanitizer::sanitize_strict;
assert_eq!(sanitize_strict("<b>test</b>"), "test");
assert_eq!(sanitize_strict("javascript:alert('xss')"), "alert('xss')");
```
