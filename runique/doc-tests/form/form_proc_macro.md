# Formulaire basé modèle avec `#[form(...)]`

La macro `#[form(...)]` génère la struct, `impl FormAccess` (champs du modèle et accesseurs) et `impl ModelForm`.
Le dev n'écrit que `impl RuniqueForm`, avec sa logique métier.

```rust,ignore
use runique::prelude::*;

#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;

impl RuniqueForm for RegisterForm {}
```

## Avec validation métier

`#[async_trait]` requis uniquement quand on override une méthode async :

```rust,ignore
use runique::prelude::*;

#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.get_string("username").len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 caractères".to_string());
        }
        if !self.get_string("email").contains('@') {
            errors.insert("email".to_string(), "Email invalide".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```
