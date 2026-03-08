# Architecture d'un projet Runique

Un projet Runique est une crate binaire Rust standard. `runique new` génère la structure de base, que tu fais évoluer selon tes besoins.

## Structure type

```text
mon-projet/
├── src/
│   ├── main.rs          # Point d'entrée — RuniqueApp builder
│   ├── admin.rs         # Déclaration admin!{} (si admin activé)
│   ├── urls.rs          # urlpatterns! — table de routage
│   ├── views.rs         # Handlers (fonctions async)
│   ├── forms.rs         # Structs RuniqueForm (ou dossier forms/)
│   ├── models.rs        # Structs métier (ou dossier models/)
│   └── admins/          # Généré par le daemon — ne pas modifier
│       ├── generated.rs
│       └── router.rs
├── templates/           # Templates Tera (.html)
├── static/              # Fichiers statiques (CSS, JS, images)
│   └── media/           # Uploads (FileField)
├── migration/           # Migrations SeaORM
│   └── src/
│       └── lib.rs
├── .env                 # Variables d'environnement
└── Cargo.toml
```

---

## Rôle de chaque fichier

**`main.rs`** — Configure et lance l'application via le builder :

```rust
#[macro_use]
extern crate runique;
use runique::prelude::*;

mod forms;
mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env();
    let db = DatabaseConfig::from_env()?.build().connect().await?;

    password_init(PasswordConfig::auto());

    RuniqueAppBuilder::new(config)
        .routes(urls::routes())
        .with_database(db)
        .statics()
        .build()
        .await?
        .run()
        .await?;

    Ok(())
}
```

**`urls.rs`** — Déclare les routes via `urlpatterns!` :

```rust
use runique::prelude::*;
use crate::views;

pub fn routes() -> Router {
    urlpatterns![
        get  "/",           views::index,
        get  "/register",   views::register,
        post "/register",   views::register,
    ]
}
```

**`views.rs`** — Handlers de requêtes :

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => { "title" => "Accueil" });
    request.render("index.html")
}
```

**`forms.rs`** — Formulaires typés :

```rust
pub struct RegisterForm { pub form: Forms }

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Nom").required());
        form.field(&TextField::email("email").label("Email").required());
    }
    impl_form_access!();
}
```

**`admin.rs`** — Déclaration de la vue admin (fichiers générés dans `src/admins/`) :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"],
    }
}
```

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Concepts clés](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/concepts/concepts.md) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/macros/macros.md) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/tera/tera.md) | Tags Django-like, filtres, fonctions |
| [Stack middleware](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/middleware/middleware.md) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/lifecycle/lifecycle.md) | Cycle de vie, bonnes pratiques |

---

## Prochaines étapes

← [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md) | [**Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md) →
