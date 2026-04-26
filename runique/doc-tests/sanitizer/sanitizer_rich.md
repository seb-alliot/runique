# Exemple

```rust
use runique::utils::forms::sanitizer::sanitize_rich;
let html = "<b>gras</b> <script>alert('xss')</script>";
let result = sanitize_rich(html);
assert!(result.contains("<b>gras</b>"));
assert!(!result.contains("<script>"));
```
