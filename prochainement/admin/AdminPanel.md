
```markdown
# AdminPanel — Architecture & Décisions Validées

## Vision Architecturale

### Concept Principal
AdminPanel = **Application séparée mais cohabitante** avec le projet du développeur.

```
Server Runique (processus unique)
  ├─ Application User (projet dev)     → /*, /users, /blog, etc.
  └─ Admin Panel (framework)           → /admin/*
```

**Deux applications distinctes** :
- Même serveur HTTP
- Même processus
- Isolation complète (routing, middleware, templates)
- Convention stricte : prefix `/admin` obligatoire

### Avantages de cette approche
✅ **Isolation mentale** : Le dev ne mélange pas logique métier et admin
✅ **Performance** : Middleware admin ne s'applique que sur `/admin/*`
✅ **Sécurité** : Surface d'attaque isolée, auth séparé
✅ **Maintenance** : Évolutions admin sans toucher au code métier

---

## Structure Physique

### Fichiers Projet Utilisateur

```
mon-projet/
  src/
    admin.rs          ← Déclarations admin (dev écrit ici)
    main.rs           ← Point d'activation .with_admin()
    models/           ← Models utilisés par admin
    forms/            ← Formulaires utilisés par admin

  target/             ← Dossier de build Rust
    runique/          ← Code généré par Runique
      admin/
        generated.rs  ← Handlers typés générés
        registry.json ← Métadonnées parsées
```

### Pourquoi `target/runique/` ?
✅ Automatiquement `.gitignore` (convention Rust)
✅ `cargo clean` nettoie tout
✅ Pas de pollution du code source
✅ Séparation claire génération vs source

---

## Déclarations Admin

### Format dans `src/admin.rs`

```rust
// Syntaxe de base
admin!(UserModel => UserForm);

// Avec métadonnées enrichies
admin!(UserModel => UserForm,
    title: "Utilisateurs",
    icon: "user",
    permissions: ["admin", "staff"]
);

admin!(BlogModel => BlogForm,
    title: "Articles de blog",
    icon: "edit"
);

admin!(ProductModel => ProductForm,
    title: "Produits",
    icon: "shopping-cart"
);
```

### Métadonnées supportées
- `title` : Nom affiché dans l'interface admin
- `icon` : Icône pour la navigation (optionnel)
- `permissions` : Rôles requis pour accès (optionnel)

### Règles de déclaration
- **Un seul model** et **un seul form principal** par déclaration
- **Ordre de déclaration** = ordre d'affichage dans l'admin
- **Pas de doublon** : un model = un formulaire admin
- **Convention table user** : nom obligatoire pour cohérence

---

## Flow de Génération

### Timeline Complète

```
1. Développeur écrit src/admin.rs
   ↓
2. CLI `runique run` lit src/main.rs
   ↓
3. Détecte .with_admin() dans main.rs
   ↓
4. Lance démon en background
   ↓
5. Démon parse admin.rs
   ↓
6. Génère handlers typés dans target/runique/admin/
   ↓
7. cargo build compile :
   - Projet dev d'abord (models, forms)
   - Code admin ensuite (utilise types du dev)
   ↓
8. Au runtime, AdminStaging construit AdminPanel
   ↓
9. Serveur démarre avec les deux apps
```

### Pourquoi cet ordre ?

**Admin a besoin du code du dev** :
```rust
// admin.rs déclare
admin!(crate::models::users::Model => crate::forms::users::UserForm);

// Le démon génère (dans target/runique/admin/generated.rs)
async fn admin_users_handler(
    Prisme(form): Prisme<crate::forms::users::UserForm>
) -> Response {
    // Utilise les types du projet dev !
}
```

**Donc** : Projet dev compile → Admin peut référencer ses types → Admin compile

---

## Structure AdminPanel

### Struct Principale

```rust
pub struct AdminPanel {
    /// Router isolé pour toutes les routes /admin/*
    router: Router,

    /// Registre des formulaires et métadonnées (JSON parsé)
    registry: AdminRegistry,

    /// Moteur de templates dédié à l'admin
    templates: TemplateEngine,

    /// Middleware d'authentification admin
    middleware: AdminAuth,

    /// Assets CSS/JS de l'interface admin
    assets: AdminAssets,
}
```

### Routes Générées Automatiquement

```
/admin/login              ← Authentification admin
/admin/dashboard          ← Tableau de bord
/admin/users              ← CRUD User (GET + POST)
/admin/blog               ← CRUD Blog (GET + POST)
/admin/products           ← CRUD Products (GET + POST)
```

Chaque route a son **handler typé** généré :
```rust
async fn admin_users_handler(Prisme(form): Prisme<UserForm>) { ... }
async fn admin_blog_handler(Prisme(form): Prisme<BlogForm>) { ... }
```

---

## Intégration Builder Intelligent

### AdminStaging dans le Pipeline

AdminStaging s'intègre comme les autres stagings (Core, Middleware, Statics) :

```rust
RuniqueApp::builder(config)
    .core(|c| c.with_database(db))
    .routes(app_routes)
    .middleware(|m| m.with_csp(true))
    .with_admin(|admin| {
        admin
            .hot_reload(true)
            .prefix("/admin")
            .permissions(["admin"])
    })
    .build().await?
```

### Ordre de Construction

**CRITIQUE** : Admin se construit **en dernier** car il dépend du code du dev.

```
1. CoreStaging (DB, templates)
2. MiddlewareStaging (session, CSRF, etc.)
3. StaticStaging (fichiers statiques)
4. Routes du dev
5. AdminStaging ← EN DERNIER
   ├─ Lit target/runique/admin/generated.rs (déjà compilé)
   ├─ Construit AdminPanel
   └─ Valide tout
6. Validation globale
7. Application finale (nest admin router)
```

### Router Imbriqué

```rust
// En interne dans le builder
let app = Router::new()
    .merge(user_routes)                    // Routes du dev
    .nest("/admin", admin_panel.router);   // Admin isolé

// Résultat :
// GET  /users      → handler du dev
// GET  /blog       → handler du dev
// GET  /admin/users → handler admin généré
```

**Isolation complète** : Middleware admin s'applique uniquement sur `/admin/*`

---

## CLI Runique

### Commande `runique run`

```bash
runique run
```

**Flow d'exécution** :

```rust
1. Lit src/main.rs
   ↓
2. Cherche .with_admin dans le contenu
   if found {
       3a. Spawn démon en background
       3b. Démon parse admin.rs
       3c. Génère code dans target/runique/
   }
   ↓
4. Exec cargo run
   ↓
5. Serveur démarre
   ↓
6. Ctrl+C → Tue démon + serveur proprement
```

### Détection `.with_admin()`

```rust
fn should_run_admin_daemon() -> bool {
    std::fs::read_to_string("src/main.rs")
        .map(|content| content.contains(".with_admin"))
        .unwrap_or(false)
}
```

**Convention stricte** : `.with_admin()` **doit** être dans `main.rs`.

### Cas Particuliers

- `.with_admin` commenté → Démon lance quand même (acceptable)
- `.with_admin` dans une string → Faux positif (rare, acceptable)
- Pas de `.with_admin` → Juste `cargo run`, pas de démon

---

## Hot Reload (Mode Dev)

### Configuration

```rust
.with_admin(|admin| {
    admin.hot_reload(true)  // Active le hot reload
})
```

### Workflow Hot Reload

```
1. Dev modifie src/admin.rs
   ↓
2. Démon détecte changement (via notify)
   ↓
3. Parse et régénère code dans target/runique/
   ↓
4. Trigger rebuild (touch sentinel file)
   ↓
5. cargo recompile automatiquement
   ↓
6. Serveur redémarre avec nouvelles routes
```

**Temps typique** : ~2-3 secondes du save au serveur opérationnel.

### Mode Production

En production, **pas de démon** :
- Code généré une fois pendant build
- Aucun watching
- Zéro overhead runtime

---

## Validation & Health Checks

### AdminStaging.validate()

Vérifie **avant construction** :

```
✓ src/admin.rs existe
✓ Syntaxe admin!() valide
✓ Models référencés existent
✓ Forms référencés existent
✓ Pas de doublon (model déclaré 2x)
✓ target/runique/admin/generated.rs présent
✓ registry.json valide
```

### AdminStaging.health_check()

Vérifie **après construction** :

```
✓ Routes /admin/* répondent
✓ Middleware auth en bonne position
✓ Templates admin accessibles
✓ Permissions configurées correctement
✓ Handlers peuvent instancier formulaires
```

**Budget estimé** : ~25% du temps Builder Intelligent

---

## Sécurité

### Middleware Auth Isolé

L'admin a son propre middleware d'authentification :

```rust
pub struct AdminAuth {
    required_roles: Vec<String>,
    check_superuser: bool,
}
```

Appliqué **uniquement** sur le router admin via nesting :

```rust
let admin_router = Router::new()
    .route("/users", post(admin_users_handler))
    .layer(admin_auth_middleware);  // Appliqué ici seulement

Router::new().nest("/admin", admin_router);
```

**Pas de slot global** : Le middleware admin n'affecte pas les routes du dev.

### Convention Table User

**Obligatoire** : La table user doit s'appeler `users` (convention).

**Pourquoi** : L'admin a besoin d'une table user de référence pour :
- Authentification superuser
- Gestion des permissions
- Affichage des utilisateurs dans l'admin

**Champs requis** :
```rust
struct User {
    id: i32,
    email: String,
    password_hash: String,
    is_superuser: bool,
    is_staff: bool,
    is_active: bool,
}
```

### CLI createsuperuser

Pour créer le premier admin :

```bash
runique createsuperuser --username admin --email admin@example.com
```

Génère un superuser avec :
- `is_superuser = true`
- `is_staff = true`
- `is_active = true`
- Mot de passe hashé (Argon2)

---

## Templates Admin

### Structure

```
runique/src/admin/
  templates/
    login.html          ← Page de connexion
    dashboard.html      ← Tableau de bord
    form_list.html      ← Liste des formulaires
    form_detail.html    ← CRUD d'un formulaire
```

**Séparation totale** : Les templates admin ne sont pas dans le projet dev.

### Variables Contexte Tera

```html
<!-- Disponibles dans tous les templates admin -->
{{ admin_forms }}        <!-- Liste des formulaires -->
{{ user }}               <!-- User connecté -->
{{ permissions }}        <!-- Permissions du user -->
```

### Boucle d'Affichage

```html
{% for form_meta in admin_forms %}
  <div class="admin-form">
    <h3>{{ form_meta.title }}</h3>
    <a href="/admin/{{ form_meta.key }}">Gérer</a>
  </div>
{% endfor %}
```

---

## Registry JSON

### Format

```json
{
  "users": {
    "model": "crate::models::users::Model",
    "form": "crate::forms::users::UserForm",
    "title": "Utilisateurs",
    "icon": "user",
    "permissions": ["admin", "staff"],
    "route": "/admin/users"
  },
  "blog": {
    "model": "crate::models::blog::Model",
    "form": "crate::forms::blog::BlogForm",
    "title": "Articles de blog",
    "icon": "edit",
    "permissions": null,
    "route": "/admin/blog"
  }
}
```

### Usage

- **Généré par le démon** après parsing de admin.rs
- **Lu par AdminStaging** pour construire le router
- **Exposé au contexte Tera** pour l'UI admin

---

## Comparaison Model ↔ Form

### Démon de Surveillance

Le démon compare automatiquement :

```
Source de vérité : Model struct
  ↓
Extraction champs Model
  ↓
Extraction champs Form (via register_fields())
  ↓
Comparaison (exclusions : id, created_at, updated_at)
  ↓
Diagnostics publiés (.runique/diagnostics.json)
```

### Diagnostics

```json
{
  "timestamp": "2026-02-09T14:30:00Z",
  "errors": [
    {
      "model": "UserModel",
      "form": "UserForm",
      "issue": "missing_field",
      "field": "phone_number",
      "message": "Champ 'phone_number' présent dans Model mais absent du Form"
    }
  ],
  "warnings": [
    {
      "model": "BlogModel",
      "form": "BlogForm",
      "issue": "extra_field",
      "field": "temp_data",
      "message": "Champ 'temp_data' présent dans Form mais absent du Model"
    }
  ]
}
```

**Feedback temps réel** : ~100-200ms après sauvegarde.

---

## Points en Suspens

### Décisions Finales à Prendre

1. **build.rs vs démon pur**
   - Option A : build.rs seulement (simple, pas de hot reload)
   - Option B : build.rs + démon (meilleure UX dev)
   - Option C : démon pur (dépend de `runique run`)

2. **Health checks spécifiques**
   - Quels checks sont critiques vs nice-to-have ?
   - Combien de temps allouer (~25% budget Builder) ?

3. **Gestion erreurs admin.rs invalide**
   - Message d'erreur clair
   - Suggestions de correction
   - Fallback : désactiver admin ou crash ?

4. **Permissions granulaires**
   - Par formulaire ? Par action (read/write/delete) ?
   - Rôles custom du dev ou rôles imposés ?

---

## Innovations Runique

### Ce qui n'existe nulle part ailleurs

1. **Admin auto-généré avec typage fort**
   - Django : admin dynamique mais Python (pas de types)
   - Rails : admin via gems mais conventions lâches
   - **Runique** : handlers typés (`Prisme<T>`) générés automatiquement

2. **Comparaison Model ↔ Form en temps réel**
   - Aucun framework ne compare automatiquement
   - Feedback instantané sur les écarts
   - Évite les bugs de champs manquants

3. **Intégration Builder Intelligent**
   - Admin = staging comme les autres
   - Ordre flexible, validation stricte
   - Health checks après assembly

4. **Convention forte avec flexibilité**
   - Convention : table `users`, prefix `/admin`
   - Flexibilité : métadonnées, permissions custom
   - Équilibre optimal

---

## Récapitulatif Final

### Ce qui est validé ✅

- ✅ Architecture : app séparée, router imbriqué
- ✅ Génération : démon → target/runique/ → compilation
- ✅ CLI : `runique run` avec détection `.with_admin`
- ✅ Staging : AdminStaging construit en dernier
- ✅ Typage : handlers avec `Prisme<ConcreteForm>`
- ✅ Métadonnées : JSON registry pour UI
- ✅ Hot reload : démon + notify + rebuild
- ✅ Sécurité : auth isolé, middleware dédié

### Ce qui reste à décider 🤔

- 🤔 build.rs vs démon pur
- 🤔 Niveau de détail health checks
- 🤔 Gestion erreurs parsing admin.rs
- 🤔 Granularité permissions

---

**Dernière mise à jour** : 2026-02-09
**Statut** : Architecture validée, implémentation à venir
```

---
