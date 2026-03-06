# Exemple

```rust
use std::env;
env::set_var("TEST_ENV_VAR", "valeur");
assert_eq!(runique::utils::config::lecture_env::env_or_default("TEST_ENV_VAR", "defaut"), "valeur");
assert_eq!(runique::utils::config::lecture_env::env_or_default("INEXISTANTE_XYZ", "defaut"), "defaut");
```
