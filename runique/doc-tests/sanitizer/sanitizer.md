# Exemple

```rust
use runique::utils::forms::sanitizer::sanitize;
// Champ sans HTML riche : les balises sont supprimées
let result = sanitize("autre", "<b>gras</b>");
assert!(!result.contains("<b>"));
```
