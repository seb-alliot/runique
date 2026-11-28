# Exemple

```rust
use runique::security::sanitizer::sanitize;
// Champ sans HTML riche : les balises sont supprimées
let result = sanitize("autre", "<b>gras</b>");
assert!(!result.contains("<b>"));
```
