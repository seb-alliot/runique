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
let mut form = Forms::new("csrf_token");
form.field(&TextField::text("username").label("Pseudo"));
form.field(&TextField::email("email").label("Email"));
```

### Utiliser l'ORM

```rust
impl_objects!(User);
let users = User::objects.all(&db).await?;
```

### Créer une route

```rust
#[urlpatterns]
pub fn routes() -> Vec<Route> {
    vec![
        Route::get("/", views::home),
        Route::post("/register", views::register),
    ]
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
