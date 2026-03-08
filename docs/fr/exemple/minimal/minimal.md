# Application minimale

## Arborescence

```
mon_app/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── url.rs
│   └── views.rs
├── templates/
│   └── index.html
└── static/
    └── css/
        └── main.css
```

---

## main.rs

```rust
#[macro_use]
extern crate runique;

mod url;
mod views;

use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    password_init(PasswordConfig::auto_with(Manual::Argon2));

    let config = RuniqueConfig::from_env();

    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

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

---

## url.rs

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",
        "/about" => view!{ GET => views::about }, name = "about",
    }
}
```

---

## views.rs

```rust
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
        "message" => "Bienvenue sur mon app Runique !",
    });
    request.render("index.html")
}

pub async fn about(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "À propos",
    });
    request.render("about.html")
}
```

---

## templates/index.html

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    {% messages %}
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
    <a href='{% link "about" %}'>À propos</a>
</body>
</html>
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/formulaires/formulaires.md) | CRUD avec formulaires |
| [Upload](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/upload/upload.md) | Upload de fichier |

## Retour au sommaire

- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md)
