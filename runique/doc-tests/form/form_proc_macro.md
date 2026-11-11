# Formulaire basé modèle avec `#[form(...)]`

La macro `#[form(...)]` génère la struct, `impl FormAccess` (champs du modèle et accesseurs) et `impl ModelForm`.
Le dev n'écrit que `impl RuniqueForm`, avec sa logique métier.

```rust,no_run
use runique::prelude::*;

mod article {
    use runique::prelude::*;

    model! {
        Article,
        table: "articles",
        pk: id => i32,
        {
            title: text [max_length: 100, required],
            body:  textarea [required],
        }
    }
}

#[form(schema = article, fields = [title, body])]
pub struct ArticleForm;

impl RuniqueForm for ArticleForm {}

fn main() {}
```

## Avec validation métier

`#[async_trait]` est requis dès qu'on surcharge une méthode async :

```rust,no_run
use runique::prelude::*;

mod article {
    use runique::prelude::*;

    model! {
        Article,
        table: "articles",
        pk: id => i32,
        {
            title: text [max_length: 100, required],
            body:  textarea [required],
        }
    }
}

#[form(schema = article, fields = [title, body])]
pub struct ArticleForm;

#[async_trait]
impl RuniqueForm for ArticleForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.cleaned::<String>("title").unwrap_or_default().len() < 3 {
            errors.insert("title".to_string(), "Minimum 3 caractères".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn main() {}
```
