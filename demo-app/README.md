# 🎓 Demo App - Application d'exemple

Une application d'exemple complète utilisant le framework Runique.

## 📁 Structure

```
demo-app/
├── src/
│   ├── main.rs             # Point d'entrée
│   ├── forms.rs            # Définition des formulaires
│   ├── url.rs              # Configuration des routes
│   ├── views.rs            # Gestionnaires de requêtes
│   ├── prelude.rs          # Imports simplifiés
│   └── models/             # Modèles SeaORM
├── templates/              # Templates Tera
├── static/                 # Fichiers statiques (CSS, JS)
├── media/                  # Médias (images, etc.)
├── migration/              # Migrations BD
└── Cargo.toml
```

## 🚀 Démarrage

### 1. Installation des dépendances

```bash
cd demo-app
cargo build
```

### 2. Configuration

Créer un fichier `.env` :

```env
DATABASE_URL=sqlite:demo.db
RUNIQUE_DEBUG=true
```

### 3. Lancer l'app

```bash
cargo run
```

L'application sera accessible sur `http://localhost:8000`

## 📝 Fonctionnalités

- ✅ Formulaires (inscription, recherche, blog)
- ✅ Authentification utilisateur
- ✅ Gestion des utilisateurs
- ✅ CRUD pour blog posts
- ✅ Templates Tera

## 🎯 Pages principales

| Route | Description |
|-------|-------------|
| `/` | Accueil |
| `/inscription` | Formulaire d'inscription |
| `/search` | Recherche d'utilisateurs |
| `/blog` | Liste des articles blog |
| `/profile` | Profil utilisateur |

## 📚 Exemples de code

### Créer un formulaire

```rust
#[derive(RuniqueForm)]
pub struct UserForm {
    #[field(label = "Pseudo", required, min_length = 3)]
    pub username: String,

    #[field(label = "Email", required, input_type = "email")]
    pub email: String,
}

// Dans le handler
async fn handle_form(
    Prisme(mut form): Prisme<UserForm>,
    mut template: TemplateContext,
) -> Response {
    if form.is_valid().await {
        // Traiter le formulaire
    }
    template.context.insert("form", form);
    template.render("form.html")
}
```

### Utiliser l'ORM

```rust
use impl_objects;

// Auto-génère un Objects manager avec all(), filter(), etc.
impl_objects!(User);

async fn get_users(db: &DbConn) -> Result<Vec<Model>, Error> {
    User::objects.all(&db).await
}

async fn filter_users(db: &DbConn) -> Result<Vec<Model>, Error> {
    User::objects
        .filter(Column::Email.eq("test@test.com"))
        .all(&db)
        .await
}
```

### Créer une route

```rust
use axum::Router;
use axum::routing::{get, post};

fn routes() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/register", post(register))
        .route("/profile/:id", get(profile))
}
```

## 🧪 Tests

```bash
# Tests
cargo test

# Avec logs
RUST_LOG=debug cargo test
```

## 📊 État

- 📈 Complétude : 8.5/10
- ✅ Formulaires fonctionnels
- ✅ Routage complet
- ✅ Templates disponibles
- ✅ BD intégrée

## 📚 Documentation

- [Formulaires](../docs/en/05-forms.md)
- [Routage](../docs/en/04-routing.md)
- [Templates](../docs/en/06-templates.md)
- [ORM](../docs/en/07-orm.md)

## 🔧 Développement

### Ajouter une page

1. Créer une fonction dans `views.rs`
2. Ajouter une route dans `url.rs`
3. Créer un template dans `templates/`

### Ajouter un formulaire

1. Définir le formulaire dans `forms.rs`
2. Utiliser dans une view
3. Traiter la soumission

### Ajouter un modèle

1. Créer dans `models/`
2. Ajouter dans `models/mod.rs`
3. Utiliser avec l'ORM

## 💡 Conseils

- Vérifier `src/prelude.rs` pour les imports disponibles
- Consulter les exemples en docs/
- Utiliser `cargo check` pour vérifier rapidement
- Utiliser `cargo build` pour compiler

## 🚀 Production

Pour déployer :

```bash
cargo build --release
```

Le binaire sera dans `target/release/demo-app`

---

**Pour en savoir plus** : [Documentation complète](../docs/en/README.md)
