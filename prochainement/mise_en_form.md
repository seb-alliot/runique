
# AdminPanel - Architecture & Design

## Vue d'ensemble

**AdminPanel** est une interface d'administration auto-générée, type-safe et hot-reloadable pour Runique. Inspirée de Django Admin, elle combine la simplicité déclarative avec les garanties de typage de Rust.

### Principes fondamentaux

- **Séparation stricte** : AdminPanel = application cohabitante isolée
- **Type-safety** : Vérification compile-time des Models et Forms
- **Hot reload** : Modifications détectées et appliquées en temps réel
- **Défense en profondeur** : Sécurité à 3 niveaux (UI, Middleware, Handler)
- **Convention over configuration** : Syntaxe simple, comportement prévisible

---

## Architecture globale

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Runique                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐         ┌────────────────────┐         │
│  │   Application   │         │   AdminPanel       │         │
│  │   Métier        │         │   /admin/*         │         │
│  │                 │         │                    │         │
│  │  - Routes user  │         │  - Routes admin    │         │
│  │  - Middlewares  │         │  - Auth dédié      │         │
│  │  - Templates    │         │  - Templates admin │         │
│  └─────────────────┘         └────────────────────┘         │
│                                                             │
│           ↓                            ↓                    │
│     Router principal (.nest("/admin", admin_router))        │
└─────────────────────────────────────────────────────────────┘
```

### Composants principaux

```rust
pub struct AdminPanel {
    pub router: Router,              // Routes /admin/*
    pub registry: AdminRegistry,     // Catalogue des ressources
    pub config: AdminConfig,         // Configuration globale
}

pub struct AdminRegistry {
    pub resources: Vec<AdminResource>,
}

pub struct AdminResource {
    pub key: &'static str,              // "users"
    pub model_path: &'static str,       // "crate::models::users::Model"
    pub form_path: &'static str,        // "crate::forms::users::UserForm"
    pub title: &'static str,            // "Utilisateurs"
    pub permissions: ResourcePermissions,
    pub display: DisplayConfig,
}
```

---

## Syntaxe et déclarations

### Fichier unique : `src/admin.rs`

```rust
// ============================================================
// IMPORTS (style Rust standard)
// ============================================================
use crate::models::users::Model as User;
use crate::forms::users::UserForm;
use crate::models::blog::Model as Blog;
use crate::forms::blog::BlogForm;
use crate::models::comments::Model as Comment;
use crate::forms::comments::CommentForm;

// ============================================================
// DÉCLARATIONS ADMIN
// ============================================================
admin! {
    users: User => UserForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        icon: "user",
        pagination: 50
    }

    blog: Blog => BlogForm {
        title: "Articles de blog",
        permissions: ["admin", "editor"],
        icon: "edit",
        pagination: 100
    }

    comments: Comment => CommentForm {
        title: "Commentaires",
        permissions: ["admin", "moderator"]
    }
}
```

### Options de configuration

```rust
admin! {
    key: Model => Form {
        // Affichage
        title: "Nom affiché",            // Requis
        icon: "icon-name",               // Optionnel

        // Sécurité
        permissions: ["role1", "role2"], // Optionnel (défaut: ["admin"])

        // Pagination
        pagination: 50,                  // Optionnel (défaut: 50)

        // Colonnes affichées
        display: ["col1", "col2"],       // Optionnel (défaut: toutes)

        // Thème spécifique
        theme: "dark",                   // Optionnel (défaut: global)
    }
}
```

---

## Cycle de vie et génération

### 1. Développeur écrit

```rust
// src/admin.rs
use crate::models::users::Model as User;
use crate::forms::users::UserForm;

admin! {
    users: User => UserForm {
        title: "Utilisateurs"
    }
}
```

### 2. Daemon surveille et génère

```bash
$ runique run
[daemon] Watching src/admin.rs...
[daemon] Change detected
[daemon] Parsing imports...
[daemon] Parsing admin! declarations...
[daemon] Generating registry.json...
[daemon] Generating handlers...
[daemon] ✓ Done (150ms)
```

**Génère 2 fichiers** :

#### A) `target/runique/admin/registry.json`
```json
[
  {
    "key": "users",
    "model": "crate::models::users::Model",
    "form": "crate::forms::users::UserForm",
    "title": "Utilisateurs",
    "route": "/admin/users",
    "permissions": ["admin"],
    "pagination": 50
  }
]
```

#### B) `target/runique/admin/generated.rs`
```rust
use crate::models::users::Model as User;
use crate::forms::users::UserForm;

pub fn admin_config() -> AdminConfig {
    AdminConfig {
        resources: vec![
            AdminResource::new::<User, UserForm>(
                "users",
                "Utilisateurs",
                vec!["admin"]
            )
        ]
    }
}

pub async fn admin_users_list(
    State(db): State<DatabaseConnection>,
    Extension(user): Extension<AdminUser>,
) -> Response {
    // Validation permissions
    if !user.has_any_role(&["admin"]) {
        return (StatusCode::FORBIDDEN, "Insufficient permissions").into_response();
    }

    // Liste paginée
    let users = User::find().paginate(&db, 50).fetch_page(0).await?;
    render_admin_list("users", users)
}

// Handlers create, edit, delete...
```

### 3. Compilation vérifie les types

```bash
$ cargo build
[compiling] Checking types...
[compiling] ✅ User found in crate::models::users
[compiling] ✅ UserForm found in crate::forms::users
[compiling] Build complete
```

**Si erreur** :
```rust
use crate::models::users::Mdoel as User;  // ← Typo

// Résultat:
error[E0432]: unresolved import `crate::models::users::Mdoel`
```

### 4. Runtime : AdminPanel opérationnel

```bash
[server] Admin panel loaded
[server] ✅ /admin/users
[server] ✅ /admin/blog
[server] ✅ /admin/comments
```

---

## Sécurité : Défense en profondeur

### Niveau 1 : UI (Tera templates)

```html
{% if user.is_admin %}
  <a href="/admin/users/delete">Supprimer</a>
{% elif user.is_staff %}
  <a href="/admin/users/edit">Modifier</a>
{% endif %}
```

**Protection** : Cosmétique uniquement
**Contournable** : ✅ Oui (DevTools, curl)

---

### Niveau 2 : Middleware global

```rust
// Appliqué à TOUTES les routes /admin/*
pub struct AdminMiddleware {
    pub required_roles: Vec<String>,  // ["admin", "staff"]
}

// Dans le builder
router
    .nest("/admin", admin_routes)
    .layer(AdminAuthMiddleware::new(vec!["admin", "staff"]))
```

**Protection** : Filtre global
**Contournable** : ❌ Non

---

### Niveau 3 : Handler granulaire

```rust
// Généré automatiquement par daemon
pub async fn admin_users_delete(
    Extension(user): Extension<AdminUser>,
    Path(id): Path<i32>,
) -> Response {
    // Validation OBLIGATOIRE
    if !user.has_role("admin") {
        return (StatusCode::FORBIDDEN, "Admin only").into_response();
    }

    // Action autorisée
    UserEntity::delete_by_id(id).exec(&db).await?;
    redirect("/admin/users")
}
```

**Protection** : Par action (CRUD)
**Contournable** : ❌ Non

---

### Permissions granulaires

```rust
pub struct ResourcePermissions {
    pub list: Vec<String>,       // GET /admin/users
    pub view: Vec<String>,       // GET /admin/users/:id
    pub create: Vec<String>,     // POST /admin/users
    pub edit: Vec<String>,       // PUT /admin/users/:id
    pub delete: Vec<String>,     // DELETE /admin/users/:id
}
```

**Exemple** :
```rust
admin! {
    users: User => UserForm {
        permissions: {
            list: ["admin", "staff"],      // Staff peut lister
            view: ["admin", "staff"],      // Staff peut voir détails
            create: ["admin"],             // Admin seulement
            edit: ["admin"],               // Admin seulement
            delete: ["admin"]              // Admin seulement
        }
    }
}
```

---

## Configuration globale

```rust
pub struct AdminConfig {
    pub prefix: String,          // "/admin" (par défaut)
    pub theme: String,           // "default"
    pub hot_reload: bool,        // true en dev, false en prod
    pub pagination_default: usize, // 50
}
```

**Utilisation dans le builder** :
```rust
RuniqueApp::builder(config)
    .routes(app_routes)
    .with_admin(|admin| admin
        .prefix("/admin")
        .theme("dark")
        .hot_reload(cfg!(debug_assertions))
    )
    .build().await?
```

---

## Workflow développeur

### Ajouter une nouvelle ressource

```rust
// 1. Ajouter l'import
use crate::models::products::Model as Product;
use crate::forms::products::ProductForm;

// 2. Déclarer la ressource
admin! {
    products: Product => ProductForm {
        title: "Produits",
        permissions: ["admin", "inventory"]
    }
}

// 3. Sauvegarder
// → Daemon détecte automatiquement
// → Génère handlers
// → Cargo rebuild
// → Serveur redémarre
// → /admin/products disponible

// Total: 2-3 secondes
```

### Modifier une ressource existante

```rust
// Changer le titre
admin! {
    users: User => UserForm {
        title: "Utilisateurs et comptes",  // ← Modifié
        permissions: ["admin"]
    }
}

// Sauvegarder → Hot reload automatique
```

### Supprimer une ressource

```rust
// Commenter ou supprimer la déclaration
// admin! {
//     old_resource: OldModel => OldForm { ... }
// }

// Sauvegarder → Route disparaît automatiquement
```

---

## Comparaison Model ↔ Form

Le daemon vérifie automatiquement la cohérence :

```rust
// Model
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub phone: String,          // ← Champ dans Model
    pub created_at: DateTime,
}

// Form
impl RuniqueForm for UserForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("name"));
        form.field(&TextField::email("email"));
        // phone manquant !
    }
}
```

**Diagnostic généré** :
```json
{
  "errors": [{
    "model": "User",
    "form": "UserForm",
    "issue": "missing_field",
    "field": "phone",
    "message": "Field 'phone' in Model but missing in Form"
  }]
}
```

**Exclusions automatiques** : `id`, `created_at`, `updated_at`

---

## Health Checks

### Avant construction (AdminStaging::validate())

```rust
pub fn validate(&self) -> Vec<ValidationError> {
    vec![
        check_admin_rs_exists(),           // src/admin.rs présent ?
        check_imports_valid(),             // Tous les imports résolus ?
        check_no_duplicates(),             // Pas de clés en double ?
        check_registry_json_present(),     // Fichier généré existe ?
        check_generated_rs_present(),      // Handlers générés existent ?
    ]
}
```

### Après construction (AdminStaging::health_check())

```rust
pub async fn health_check(&self) -> Vec<HealthResult> {
    vec![
        test_routes_respond(),             // /admin/users → 200 ?
        test_permissions_enforced(),       // Middleware bloque bien ?
        test_handlers_compile(),           // Code généré valide ?
        test_registry_coherent(),          // JSON cohérent ?
    ]
}
```

---

## Conventions obligatoires

### Table `users`

```rust
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,      // Argon2
    pub is_superuser: bool,         // Accès total
    pub is_staff: bool,             // Accès admin limité
    pub is_active: bool,            // Compte actif
}
```

**CLI pour créer le superuser** :
```bash
$ runique createsuperuser --email admin@example.com
Password: ********
Superuser created successfully
```

### Structure projet

```
mon-projet/
├── src/
│   ├── main.rs
│   ├── admin.rs              ← Déclarations admin (UN SEUL FICHIER)
│   ├── models/
│   │   ├── users.rs
│   │   ├── blog.rs
│   │   └── comments.rs
│   └── forms/
│       ├── users.rs
│       ├── blog.rs
│       └── comments.rs
└── target/
    └── runique/
        └── admin/
            ├── registry.json      ← Généré (JSON pour UI)
            └── generated.rs       ← Généré (Handlers typés)
```

---

## Intégration dans le builder

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = RuniqueConfig::load()?;

    let app = RuniqueApp::builder(config)
        // 1. Core
        .core()

        // 2. Middlewares
        .middleware()

        // 3. Statics
        .statics()

        // 4. Routes métier
        .routes(app_routes)

        // 5. AdminPanel (TOUJOURS EN DERNIER)
        .with_admin(|admin| admin
            .prefix("/admin")
            .hot_reload(cfg!(debug_assertions))
        )

        // 6. Validation et assemblage
        .build().await?;

    app.serve().await
}
```

**Ordre critique** : Admin construit après le code métier (dépend des Models/Forms).

---

## Innovations uniques

### 1. Type-safety + Hot Reload
- Rust frameworks : soit type-safe (statique), soit hot-reload (dynamique)
- **Runique** : les deux simultanément

### 2. Comparaison Model ↔ Form automatique
- Aucun framework ne vérifie cette cohérence
- **Runique** : diagnostics temps réel

### 3. Génération de handlers typés
- Django : admin dynamique (Python)
- Rails : gems avec conventions lâches
- **Runique** : handlers `Prisme<ConcreteForm>` générés automatiquement

### 4. Défense en profondeur native
- Frameworks web : sécurité manuelle
- **Runique** : validation à 3 niveaux par défaut

---

## FAQ

### Pourquoi un seul fichier `admin.rs` ?

**Performance** : Daemon surveille 1 fichier au lieu de tout le projet
**Simplicité** : Pas d'ambiguïté sur où déclarer
**Convention** : Comme Django (`admin.py`), Rails (`routes.rb`)

### Pourquoi pas de proc-macro ?

**Build speed** : Pas de ralentissement compilation
**Flexibilité** : Daemon peut évoluer sans recompile framework
**Debugging** : Code généré visible et modifiable

### Peut-on avoir plusieurs AdminPanels ?

Non. Un seul AdminPanel par application. Pour multi-tenant, utiliser les permissions granulaires.

### Comment gérer 100+ ressources ?

**Option 1** : CLI scaffold
```bash
$ runique admin-scaffold
# Génère admin.rs avec toutes les ressources détectées
```

**Option 2** : Multiple fichiers
```rust
// src/admin.rs
include!("admin/users.rs");
include!("admin/blog.rs");
// ...
```

### AdminPanel fonctionne avec quel ORM ?

**SeaORM uniquement** pour le moment. Support Diesel/SQLx possible en V2.

---

## Roadmap

### Phase 1 : MVP (actuel)
- ✅ Architecture validée
- ✅ Design finalisé
- ⏳ Implémentation daemon
- ⏳ Génération handlers
- ⏳ Health checks

### Phase 2 : Enrichissement
- ⏳ Actions de masse
- ⏳ Filtres avancés
- ⏳ Export CSV/Excel
- ⏳ Hooks (before_save, after_delete)
- ⏳ Customisation templates

### Phase 3 : Avancé
- ⏳ Relations ForeignKey automatiques
- ⏳ Inline editing
- ⏳ Recherche full-text
- ⏳ Logs d'audit

---

## Crédits

**Inspiré de** :
- Django Admin (Python)
- Rails ActiveAdmin (Ruby)
- Laravel Nova (PHP)

**Innovant dans** :
- Type-safety compile-time
- Hot reload avec validation
- Génération de code Rust

---

## License

Partie intégrante du framework Runique.

---

**Version** : 1.0.0-alpha
**Date** : 2026-02-09
**Auteur** : Itsuki
```
