# Formulaire basé modèle avec `#[form(...)]` et `impl_form_access!(model)`

La macro `#[form(...)]` génère la struct et `impl ModelForm`.
Le dev écrit `impl RuniqueForm` avec `impl_form_access!(model)` pour brancher les champs du modèle.

```rust,ignore
use runique::prelude::*;

#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;

impl RuniqueForm for RegisterForm {
    impl_form_access!(model);
}
```

## Avec validation métier

`#[async_trait]` requis uniquement quand on override une méthode async :

```rust,ignore
use runique::prelude::*;

#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

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
