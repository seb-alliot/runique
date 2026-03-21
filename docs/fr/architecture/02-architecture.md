# Architecture d'un projet Runique

Un projet Runique est une crate binaire Rust standard. `runique new` génère la structure de base, que tu fais évoluer selon tes besoins.

## Structure type

```text
mon-projet/
├── src/
│   ├── entities/          # Déclaration des model
│   │   ├── users.rs       # Utilise l'ast de runique pour la cli makemigrations
│   │   └── blog.rs        # Le cli est non compatible avec les struct basique
│   │
│   ├── formulaire/        # Déclaration des formulaire
│   │   ├── inscription.rs # Utilisation du moteur de formulaire
│   │   └── blog.rs        # ou de macro pro attribu
│   │
│   ├── main.rs            # Point d'entrée — RuniqueApp builder
│   ├── admin.rs           # Déclaration admin!{} (si admin activé, necessaire pour la cli runique start)
│   ├── urls.rs            # urlpatterns! — table de routage
│   ├── views.rs           # Handlers (fonctions async)
│   └──  forms.rs          # Structs RuniqueForm (ou dossier forms/)
│
├── templates/             # Templates Tera (.html)
├── static/                # Fichiers statiques (CSS, JS, images)
│   └── media/             # Uploads (FileField)
│                          # Media peux etre dans un autre dossier
│
├── migration/             # Migrations SeaORM
│   └── src/
│       └── lib.rs
├── .env                   # Variables d'environnement
└── Cargo.toml
```

---

## Rôle de chaque fichier

**`main.rs`** — Configure et lance l'application via le builder :

```rust
#[macro_use]
extern crate runique;
use runique::prelude::*;

mod entities;
mod formulaire;
mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env();
    let db_config = DatabaseConfig::from_env()?.min_connections(1).build();
    let db: DatabaseConnection = db_config.connect().await?;

    password_init(PasswordConfig::auto());

    RuniqueApp::builder(config)
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
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/register" => view!{ views::register }, name = "register",
    }
}
```

**`views.rs`** — Handlers de requêtes :

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => { "title" => "Accueil" });
    request.render("index.html")
}
```

**`formulaire/`** — Formulaires typés :

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

## Démarrer un nouveau projet

```bash
runique new mon-projet
cd mon-projet
runique start
```

`runique new` génère la structure minimale décrite ci-dessus. Voici ce que chaque fichier t'appartient de modifier ou non :

| Fichier / dossier | À modifier | Rôle |
| --- | --- | --- |
| `src/main.rs` | Oui | Configure le builder et déclare les modules |
| `src/urls.rs` | Oui | Table de routage |
| `src/views.rs` | Oui | Handlers de requêtes |
| `src/entities/` | Oui | Déclarations de modèles (AST Runique, compatible `makemigrations`) |
| `src/formulaire/` | Oui | Formulaires et validation |
| `src/admin.rs` | Oui (si admin) | Déclaration `admin!{}`, requis pour `runique start` |
| `src/admins/` | **Non** | Généré par le daemon — ne pas modifier à la main |
| `templates/` | Oui | Templates Tera |
| `static/` | Oui | CSS, JS, images |
| `migration/` | Non (sauf ajout de tables) | Migrations SeaORM |
| `.env` | Oui | Variables d'environnement |

> `runique start` surveille `src/admin.rs` et régénère `src/admins/` à chaque changement. Pour une app sans vue admin, `cargo run` suffit.

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Concepts clés](/docs/fr/architecture/concepts) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](/docs/fr/architecture/lifecycle) | Cycle de vie, bonnes pratiques |

---

## Prochaines étapes

← [Installation](/docs/fr/installation) | [**Configuration**](/docs/fr/configuration) →
