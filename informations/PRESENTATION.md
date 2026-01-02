# Hello World avec Rusti

## Introduction

Ce guide vous montre comment créer votre première application web avec Rusti en moins de 5 minutes.

---

## Prérequis

- Rust 1.9 ou supérieur installé
- Cargo (installé avec Rust)
- Un éditeur de texte

**Vérifier votre installation :**
```bash
rustc --version
cargo --version
```

---

## Étape 1 : Créer un nouveau projet
```bash
cargo new hello-rusti
cd hello-rusti
```

**Structure créée :**
```
hello-rusti/
├── Cargo.toml
└── src/
    └── main.rs
```

---

## Étape 2 : Ajouter Rusti

Modifiez `Cargo.toml` :
```toml
[package]
name = "hello-rusti"
version = "0.1.0"
edition = "2021"

[dependencies]
rusti = "1.0"
tokio = { version = "1", features = ["full"] }
```

---

## Étape 3 : Écrire le code

Remplacez le contenu de `src/main.rs` :
```rust
use rusti::prelude::*;

async fn hello() -> &'static str {
    "Hello, Rusti!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RustiApp::new(settings).await?
        .routes(Router::new().route("/", get(hello)))
        .run()
        .await?;

    Ok(())
}
```

**Explications :**
- `use rusti::prelude::*` : Importe les types essentiels
- `async fn hello()` : Handler qui retourne "Hello, Rusti!"
- `Settings::default_values()` : Configuration par défaut
- `RustiApp::new()` : Crée l'application
- `.routes()` : Définit les routes
- `.run()` : Lance le serveur

---

## Étape 4 : Lancer l'application
```bash
cargo run
```

**Sortie attendue :**
```
   Compiling hello-rusti v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 3.45s
     Running `target/debug/hello-rusti`
🦀 Rusti server running on http://127.0.0.1:3000
```

---

## Étape 5 : Tester

Ouvrez votre navigateur et allez sur :
```
http://127.0.0.1:3000
```

**Vous devriez voir :**
```
Hello, Rusti!
```

**Ou testez en ligne de commande :**
```bash
curl http://127.0.0.1:3000
```

---

## Comprendre le code

### 1. Import du prelude
```rust
use rusti::prelude::*;
```

Le prelude contient tous les types essentiels :
- `RustiApp` : L'application principale
- `Settings` : Configuration
- `Router` : Système de routing
- `Response` : Type de réponse
- Et plus encore...

### 2. Le handler
```rust
async fn hello() -> &'static str {
    "Hello, Rusti!"
}
```

Un **handler** est une fonction qui :
- Peut être asynchrone (`async`)
- Retourne une réponse (ici un texte)
- Sera appelée quand un utilisateur visite la route

**Types de retour possibles :**
- `&str` ou `String` : Texte brut
- `Html<String>` : HTML
- `Json<T>` : JSON
- `Response` : Réponse personnalisée

### 3. Configuration
```rust
let settings = Settings::default_values();
```

Configuration par défaut :
- **Adresse** : 127.0.0.1 (localhost)
- **Port** : 3000
- **Mode debug** : Activé
- **Templates** : `templates/`
- **Static files** : `static/`

### 4. Création de l'application
```rust
RustiApp::new(settings).await?
```

Initialise l'application avec :
- Configuration chargée
- Serveur HTTP prêt
- Middlewares de base

### 5. Définition des routes
```rust
.routes(Router::new().route("/", get(hello)))
```

- `Router::new()` : Crée un nouveau routeur
- `.route("/", ...)` : Définit la route pour "/"
- `get(hello)` : Utilise le handler `hello` pour les requêtes GET

### 6. Lancement du serveur
```rust
.run().await?;
```

Lance le serveur et attend les connexions.

---

## Évolution : Ajouter une deuxième route

Modifiez `src/main.rs` :
```rust
use rusti::prelude::*;

async fn hello() -> &'static str {
    "Hello, Rusti!"
}

async fn about() -> &'static str {
    "À propos de Rusti - Framework web moderne pour Rust"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    let routes = Router::new()
        .route("/", get(hello))
        .route("/about", get(about));

    RustiApp::new(settings).await?
        .routes(routes)
        .run()
        .await?;

    Ok(())
}
```

**Testez :**
- http://127.0.0.1:3000 → "Hello, Rusti!"
- http://127.0.0.1:3000/about → "À propos de Rusti..."

---

## Évolution : Retourner du JSON
```rust
use rusti::prelude::*;
use serde_json::json;

async fn hello() -> Response {
    let data = json!({
        "message": "Hello, Rusti!",
        "version": "1.0.0",
        "status": "ok"
    });

    (StatusCode::OK, Json(data)).into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RustiApp::new(settings).await?
        .routes(Router::new().route("/", get(hello)))
        .run()
        .await?;

    Ok(())
}
```

**Testez :**
```bash
curl http://127.0.0.1:3000
```

**Réponse :**
```json
{
  "message": "Hello, Rusti!",
  "version": "1.0.0",
  "status": "ok"
}
```

---

## Évolution : Paramètres d'URL
```rust
use rusti::prelude::*;

async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RustiApp::new(settings).await?
        .routes(Router::new().route("/hello/{name}", get(greet)))
        .run()
        .await?;

    Ok(())
}
```

**Testez :**
- http://127.0.0.1:3000/hello/Alice → "Hello, Alice!"
- http://127.0.0.1:3000/hello/Bob → "Hello, Bob!"

---

## Configuration personnalisée

### Changer le port
```rust
let settings = Settings::builder()
    .server("127.0.0.1", 8080, "secret-key")
    .build();
```

### Activer le mode production
```rust
let settings = Settings::builder()
    .debug(false)
    .server("0.0.0.0", 8080, "your-secret-key")
    .build();
```

### Avec fichier .env

Créez `.env` :
```env
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=my-secret-key
```

Dans `main.rs` :
```rust
let settings = Settings::from_env();
```

---

## Comparaison avec d'autres frameworks

### Django (Python)

**Django :**
```python
# views.py
def hello(request):
    return HttpResponse("Hello, Django!")

# urls.py
urlpatterns = [
    path('', hello),
]
```

**Rusti :**
```rust
async fn hello() -> &'static str {
    "Hello, Rusti!"
}

Router::new().route("/", get(hello))
```

### Express (Node.js)

**Express :**
```javascript
const express = require('express');
const app = express();

app.get('/', (req, res) => {
  res.send('Hello, Express!');
});

app.listen(3000);
```

**Rusti :**
```rust
async fn hello() -> &'static str {
    "Hello, Rusti!"
}

RustiApp::new(settings).await?
    .routes(Router::new().route("/", get(hello)))
    .run()
    .await?;
```

### Flask (Python)

**Flask :**
```python
from flask import Flask
app = Flask(__name__)

@app.route('/')
def hello():
    return 'Hello, Flask!'

if __name__ == '__main__':
    app.run()
```

**Rusti :**
```rust
async fn hello() -> &'static str {
    "Hello, Rusti!"
}

RustiApp::new(settings).await?
    .routes(Router::new().route("/", get(hello)))
    .run()
    .await?;
```

---

## Pourquoi Rusti ?

### 1. Familier

Si vous connaissez Django, Flask ou Express, vous vous sentirez chez vous :
- Syntaxe claire et concise
- Concepts familiers (routes, handlers, middleware)
- Documentation complète

### 2. Performant

Basé sur Axum et Tokio :
- Asynchrone natif
- Performances exceptionnelles
- Consommation mémoire réduite

### 3. Sécurisé

Rust garantit :
- Pas de null pointer
- Pas de data races
- Memory safety
- Thread safety

### 4. Type-safe

Le compilateur vérifie :
- Types corrects
- Erreurs à la compilation
- Pas de bugs à l'exécution

---

## Prochaines étapes

Maintenant que vous maîtrisez Hello World, explorez :

### 1. Templates HTML
```rust
pub async fn index(template: Template) -> Response {
    let context = context! {
        "title", "Ma page"
    };
    template.render("index.html", &context)
}
```

**Voir :** [Guide des templates](../documentation%20french/TEMPLATES.md)

### 2. Base de données
```rust
let users = User::objects
    .filter(users::Column::Age.gte(18))
    .all(&db)
    .await?;
```

**Voir :** [Guide de la base de données](../documentation%20french/DATABASE.md)

### 3. Formulaires
```rust
#[derive(DeriveModelForm)]
pub struct UserForm {
    pub username: String,
    pub email: String,
}
```

**Voir :** [Guide des formulaires](../documentation%20french/FORMULAIRE.md)

### 4. API REST complète
```rust
urlpatterns! {
    "/api/users" => get(list_users),
    "/api/users/{id}" => get(get_user),
    "/api/users" => post(create_user),
    "/api/users/{id}" => put(update_user),
    "/api/users/{id}" => delete(delete_user),
}
```

---

## Ressources

### Documentation
- [Guide de démarrage complet](../documentation%20french/GETTING_STARTED.md)
- [Documentation complète](../documentation%20french/)
- [README principal](../documentation%20french/README.md)

### Exemples
- [Tests d'intégration](../tests/) - 50+ exemples
- [Demo app](../demo-app/) - Application complète

### Support
- [GitHub Issues](https://github.com/votre-repo/rusti/issues)
- [Discussions](https://github.com/votre-repo/rusti/discussions)

---

## Récapitulatif

**Vous avez appris à :**
- Créer un projet Rusti
- Écrire un handler simple
- Définir des routes
- Lancer le serveur
- Retourner du JSON
- Utiliser des paramètres d'URL
- Configurer l'application

**En seulement 10 lignes de code !**
```rust
use rusti::prelude::*;

async fn hello() -> &'static str {
    "Hello, Rusti!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RustiApp::new(Settings::default_values()).await?
        .routes(Router::new().route("/", get(hello)))
        .run()
        .await?;
    Ok(())
}
```

---

**Bienvenue dans l'écosystème Rusti !**

**Développé avec passion en Rust**