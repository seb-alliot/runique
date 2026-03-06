# Exemple

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
let registry: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
assert!(registry.read().unwrap().is_empty());
```
