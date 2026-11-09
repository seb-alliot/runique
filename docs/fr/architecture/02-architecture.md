# Architecture d'un projet Runique

Un projet Runique est une crate binaire Rust standard. `runique new` génère la structure de base, que tu fais évoluer selon tes besoins.

## Structure type

```text
mon_projet/
├── src/
│   ├── main.rs            # Point d'entrée — builder RuniqueApp
│   ├── url.rs             # urlpatterns! — table de routage
│   ├── views.rs           # Handlers (fonctions async)
│   ├── forms/             # Formulaires (#[form] ou déclarés à la main)
│   ├── entities/          # Modèles model!{} / extend!{}, lus par makemigrations
│   └── admin.rs           # ModelAdmin + AdminSite (feature `admin` seulement)
│
├── templates/             # Templates Tera (.html)
├── static/                # Fichiers statiques (CSS, JS, images)
├── media/                 # Fichiers envoyés par les utilisateurs (FileField) ;
│                          # MEDIA_ROOT peut pointer ailleurs
├── migration/             # Crate des migrations (workspace)
│   ├── Cargo.toml
│   └── src/               # main.rs, lib.rs, migrations, snapshots/
├── .env                   # Variables d'environnement
└── Cargo.toml             # Workspace : l'application + migration
```

`src/entities/` ne doit contenir que des `model!{}` et `extend!{}` : `makemigrations` lit ce dossier et génère une migration pour chaque modèle qu'il y trouve. Un formulaire déclaré à la main va dans `src/forms/`.

---

## Rôle de chaque fichier

**`main.rs`** — Configure et lance l'application via le builder :

```rust
use runique::prelude::*;

mod entities;
mod forms;
mod url;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    password_init(PasswordConfig::auto_with(Manual::Argon2));

    let config: RuniqueConfig = RuniqueConfig::from_env();
    let db_config = DatabaseConfig::from_env()?.build();
    let db: DatabaseConnection = db_config.connect().await?;

    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .build()
        .await?
        .run()
        .await?;

    Ok(())
}
```

**`url.rs`** — Déclare les routes via `urlpatterns!` :

```rust
use crate::views;
use runique::prelude::*;

pub fn routes() -> Routes {
    urlpatterns! {
        "/"       => view!{ views::index },  name = "index",
        "/signup" => view!{ views::signup }, name = "signup",
    }
}
```

**`views.rs`** — Handlers de requêtes :

```rust
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => { "title" => "Accueil" });
    request.render("index.html")
}
```

**`forms/`** — Formulaires typés. Un formulaire déclaré à la main :

```rust
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct ContactForm {
    pub form: Forms,
}

impl FormAccess for ContactForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("name").label("Nom").required());
        form.field(&TextField::email("email").label("Email").required());
    }
    impl_form_access!();
}

impl RuniqueForm for ContactForm {}
```

Un formulaire tiré d'un modèle s'écrit en une ligne : `#[form(schema = blog, fields = [title, content])]`. Voir [Formulaires](/docs/fr/formulaire).

**`admin.rs`** — Déclaration typée de l'admin (`ModelAdmin` + `AdminSite`) :

```rust
use crate::entities::demo_category;
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use sea_orm::Order;

pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .site(site)
}

fn site(site: AdminSite) -> AdminSite {
    site.register(demo_category())
}

fn demo_category() -> ModelAdmin<demo_category::Entity> {
    use demo_category::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, BackLinkLabel, BackLinkUrl, SortOrder])
            .ordering([(SortOrder, Order::Asc)])
            .search([Title])
            .empty_value("—")
    })
}
```

Câblé dans `main.rs` par `.with_admin(admin::config)`.

> **L'admin est une feature à activer.** `runique::admin`, `AdminStaging` et `.with_admin(...)` n'existent que si `runique` est compilé avec la feature `admin` :
>
> ```toml
> runique = { version = "4.0.0", features = ["postgres", "admin"] }
> ```
>
> Sans elle, le compilateur signale `unresolved import runique::admin` ou `no method named with_admin`. Tout `model!{}` du projet doit être enregistré dans l'`AdminSite` : les droits d'un modèle absent sont supprimés au démarrage.

---

## Démarrer un nouveau projet

```bash
runique new mon_projet
cd mon_projet
runique migration up
cargo run
```

Ce que chaque partie du projet généré vous appartient de modifier :

| Fichier / dossier | À modifier | Rôle |
| --- | --- | --- |
| `src/main.rs` | Oui | Configure le builder et déclare les modules |
| `src/url.rs` | Oui | Table de routage |
| `src/views.rs` | Oui | Handlers de requêtes |
| `src/entities/` | Oui | Modèles (`model!{}`, `extend!{}`), lus par `makemigrations` |
| `src/forms/` | Oui | Formulaires et validation |
| `src/admin.rs` | Oui (feature `admin`) | `ModelAdmin` + enregistrement `AdminSite` |
| `templates/` | Oui | Templates Tera |
| `static/` | Oui | CSS, JS, images |
| `migration/` | Non | Tenu par `runique makemigrations` (`lib.rs`, migrations, snapshots). À la main : seulement le `up()` / `down()` d'un changement de type de colonne, que `makemigrations` ne sait pas encore écrire |
| `.env` | Oui | Variables d'environnement |

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Concepts clés](/docs/fr/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form("nom")` |
| [Fonctionnement interne de Makemigrations](/docs/fr/architecture/makemigrations) | Pipeline en trois phases : extraction AST, diff, génération SeaQuery |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](/docs/fr/architecture/lifecycle) | Cycle de vie, bonnes pratiques |
| [Tests](/docs/fr/architecture/testing) | `runique_test` : logique métier testée contre la vraie base, transaction annulée ; `runique test` ; contrôles au démarrage |

---

## Prochaines étapes

← [Installation](/docs/fr/installation) | [**Configuration**](/docs/fr/configuration) →
