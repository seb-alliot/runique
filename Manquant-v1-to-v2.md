# 🔧 Guide d'Implémentation : Restauration des Pages d'Erreur Runique

## 📦 Fichiers à Copier

### Fichier 1 : `error_context.rs`
**Source** : `/home/claude/final_error_context.rs`
**Destination** : `runique/src/utils/error_context.rs`

```bash
# Copier le fichier
cp /home/claude/final_error_context.rs runique/src/utils/error_context.rs
```

### Fichier 2 : `error_handler.rs` (remplacement complet)
**Source** : `/home/claude/final_error_handler.rs`
**Destination** : `runique/src/gardefou/composant_middleware/error_handler.rs`

```bash
# Sauvegarder l'ancien fichier (optionnel)
cp runique/src/gardefou/composant_middleware/error_handler.rs runique/src/gardefou/composant_middleware/error_handler.rs.backup

# Remplacer par la nouvelle version
cp /home/claude/final_error_handler.rs runique/src/gardefou/composant_middleware/error_handler.rs
```

---

## 📝 Modifications des Modules

### Étape 1 : Mettre à jour `utils/mod.rs`

**Fichier** : `runique/src/utils/mod.rs`

**Ajouter** :
```rust
pub mod error_context;
```

**Dans la section des ré-exports** :
```rust
pub use error_context::{
    ErrorContext, ErrorType, TemplateInfo, RequestInfo,
    StackFrame, EnvironmentInfo
};
```

**Résultat final** (exemple) :
```rust
// runique/src/utils/mod.rs

pub mod csrf;
pub mod error_context;  // ✅ NOUVEAU
pub mod response_helpers;

pub use csrf::{CsrfToken, generate_csrf_token, verify_csrf_token};
pub use error_context::{   // ✅ NOUVEAU
    ErrorContext, ErrorType, TemplateInfo, RequestInfo,
    StackFrame, EnvironmentInfo
};
pub use response_helpers::*;
```

---

### Étape 2 : Mettre à jour `gardefou/composant_middleware/mod.rs`

**Fichier** : `runique/src/gardefou/composant_middleware/mod.rs`

**Vérifier que les exports incluent** :
```rust
pub use error_handler::{
    error_handler_middleware,
    render_404,
    render_500,
    render_index,
    render_template,
};
```

**Résultat final** (exemple) :
```rust
// runique/src/gardefou/composant_middleware/mod.rs

pub mod csrf;
pub mod csp;
pub mod error_handler;
pub mod flash_message;
pub mod middleware_sanetiser;

pub use csrf::csrf_middleware;
pub use csp::{security_headers_middleware, CspConfig};
pub use error_handler::{
    error_handler_middleware,
    render_404,
    render_500,
    render_index,
    render_template,
};
pub use flash_message::flash_middleware;
pub use middleware_sanetiser::sanitize_middleware;
```

---

### Étape 3 : Mettre à jour le prelude principal

**Fichier** : `runique/src/lib.rs`

**Dans le module `prelude`** :
```rust
pub mod prelude {
    // ... autres exports ...

    // Error handling - ✅ NOUVEAU
    pub use crate::utils::error_context::{
        ErrorContext, ErrorType, TemplateInfo, RequestInfo,
        StackFrame, EnvironmentInfo
    };
    pub use crate::gardefou::composant_middleware::error_handler::{
        render_404, render_500, render_index, render_template
    };

    // ... reste des exports ...
}
```

---

## 🔍 Vérification des Dépendances

### Vérifier `Cargo.toml`

Assurez-vous que ces dépendances sont présentes :

```toml
[dependencies]
# ... autres dépendances ...

# Pour ErrorContext
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"

# Pour les templates
tera = "1.19"

# Pour Axum
axum = "0.7"

# Pour logging
tracing = "0.1"
```

---

## ✅ Tests à Effectuer

### Test 1 : Compilation

```bash
cd runique
cargo build
```

**Résultat attendu** : ✅ Compilation réussie sans erreurs

---

### Test 2 : Page 404

**Créer un handler de test** :
```rust
// Dans votre projet
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();

    RuniqueApp::builder(config)
        .routes(Router::new())  // Routes vides pour tester 404
        .build()
        .await
        .unwrap()
        .run()
        .await
        .unwrap();
}
```

**Tester** :
```bash
# Démarrer l'app
cargo run

# Dans un autre terminal
curl http://localhost:3000/page-inexistante
```

**Résultat attendu** :
- ✅ En mode debug (`DEBUG=true`) : Page de debug détaillée avec ErrorContext
- ✅ En mode prod (`DEBUG=false`) : Page 404 élégante avec bouton "Go to Homepage"

---

### Test 3 : Erreur de Template

**Créer un handler qui force une erreur** :
```rust
use runique::prelude::*;

async fn test_error(ctx: TemplateContext) -> Response {
    // Template inexistant
    ctx.render("template-qui-nexiste-pas.html")
}

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();

    let routes = Router::new()
        .route("/test-error", get(test_error));

    RuniqueApp::builder(config)
        .routes(routes)
        .build()
        .await
        .unwrap()
        .run()
        .await
        .unwrap();
}
```

**Tester** :
```bash
curl http://localhost:3000/test-error
```

**Résultat attendu** (en mode debug) :
- ✅ Page de debug complète avec :
  - Titre : "Template Rendering Error"
  - Message d'erreur détaillé
  - Liste des templates disponibles
  - Stack trace
  - Informations de requête (method, path, headers)
  - Informations d'environnement (Rust version, debug mode)

---

### Test 4 : Page d'Accueil par Défaut

```bash
curl http://localhost:3000/
```

**Résultat attendu** :
- ✅ Si `templates/index.html` existe → rendu de ce template
- ✅ Sinon, template `base_index` de Runique
- ✅ Sinon, fallback HTML élégant "Welcome to Runique"

---

## 🎨 Templates d'Erreur (Optionnels)

### Template de Debug Personnalisé

**Créer** : `templates/errors/debug_error.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{{ title }} - Debug</title>
    <link rel="stylesheet" href="{{ static_runique }}/css/error-debug.css">
</head>
<body>
    <div class="error-container">
        <h1>{{ title }}</h1>
        <p class="error-message">{{ message }}</p>

        {% if details %}
        <div class="details">
            <h2>Details</h2>
            <pre>{{ details }}</pre>
        </div>
        {% endif %}

        {% if template_info %}
        <div class="template-info">
            <h2>Template Information</h2>
            <p><strong>Template:</strong> {{ template_info.name }}</p>

            {% if template_info.source %}
            <div class="source">
                <h3>Template Source</h3>
                <pre>{{ template_info.source }}</pre>
            </div>
            {% endif %}

            <div class="available-templates">
                <h3>Available Templates</h3>
                <ul>
                {% for template in template_info.available_templates %}
                    <li>{{ template }}</li>
                {% endfor %}
                </ul>
            </div>
        </div>
        {% endif %}

        {% if request_info %}
        <div class="request-info">
            <h2>Request Information</h2>
            <p><strong>Method:</strong> {{ request_info.method }}</p>
            <p><strong>Path:</strong> {{ request_info.path }}</p>
            {% if request_info.query %}
            <p><strong>Query:</strong> {{ request_info.query }}</p>
            {% endif %}

            {% if request_info.headers %}
            <div class="headers">
                <h3>Headers</h3>
                <table>
                {% for key, value in request_info.headers %}
                    <tr>
                        <td>{{ key }}</td>
                        <td>{{ value }}</td>
                    </tr>
                {% endfor %}
                </table>
            </div>
            {% endif %}
        </div>
        {% endif %}

        {% if stack_trace %}
        <div class="stack-trace">
            <h2>Stack Trace</h2>
            <ol>
            {% for frame in stack_trace %}
                <li>
                    <strong>Level {{ frame.level }}:</strong> {{ frame.message }}
                    {% if frame.location %}
                    <span class="location">{{ frame.location }}</span>
                    {% endif %}
                </li>
            {% endfor %}
            </ol>
        </div>
        {% endif %}

        <div class="environment">
            <h2>Environment</h2>
            <p><strong>Debug Mode:</strong> {{ environment.debug_mode }}</p>
            <p><strong>Rust Version:</strong> {{ environment.rust_version }}</p>
            <p><strong>App Version:</strong> {{ environment.app_version }}</p>
            <p><strong>Timestamp:</strong> {{ timestamp }}</p>
        </div>
    </div>
</body>
</html>
```

---

## 🐛 Dépannage

### Problème 1 : "module `error_context` not found"

**Solution** :
```bash
# Vérifier que le fichier existe
ls runique/src/utils/error_context.rs

# Vérifier que mod.rs est à jour
cat runique/src/utils/mod.rs | grep error_context
```

---

### Problème 2 : "function `render_404` not found"

**Solution** :
```bash
# Vérifier les exports dans gardefou/composant_middleware/mod.rs
cat runique/src/gardefou/composant_middleware/mod.rs | grep render_404
```

---

### Problème 3 : Pages d'erreur toujours vides

**Vérifier** :
1. Le middleware est bien activé dans `RuniqueApp`
2. La configuration `debug` est correcte
3. Les templates d'erreur sont chargés

**Debug** :
```rust
// Ajouter des logs
tracing::info!("Debug mode: {}", config.debug);
tracing::info!("Available templates: {:?}", tera.get_template_names().collect::<Vec<_>>());
```

---

### Problème 4 : Template d'erreur ne s'affiche pas

**Vérifier l'ordre de chargement** des templates :

```rust
// Dans RuniqueApp::new() ou équivalent
let mut tera = Tera::default();

// 1. Charger les templates internes de Runique d'abord
Self::load_internal_templates(&mut tera)?;

// 2. Charger les templates utilisateur ensuite
// (ils peuvent override les templates internes)
for template_dir in &config.templates_dir {
    // ...
}
```

---

## 📊 Comparaison Avant/Après

### ❌ Avant (Nouvelle Architecture Sans Fix)

```
Page d'erreur :
- Mode debug : ❌ Page vide ou 500 générique
- Templates disponibles : ❌ Non affichés
- Stack trace : ❌ Absente
- Request info : ❌ Absente
- Fallback élégant : ⚠️ Basique
```

### ✅ Après (Avec Restauration)

```
Page d'erreur :
- Mode debug : ✅ Page détaillée complète
- Templates disponibles : ✅ Liste exhaustive
- Stack trace : ✅ Complète avec niveaux
- Request info : ✅ Method, path, headers
- Fallback élégant : ✅ Design soigné
- Environment info : ✅ Rust version, debug mode
- Timestamp : ✅ ISO 8601
```

---

## 🚀 Résumé des Actions

- [x] Copier `final_error_context.rs` → `runique/src/utils/error_context.rs`
- [x] Copier `final_error_handler.rs` → `runique/src/gardefou/composant_middleware/error_handler.rs`
- [ ] Mettre à jour `utils/mod.rs`
- [ ] Mettre à jour `gardefou/composant_middleware/mod.rs`
- [ ] Mettre à jour `lib.rs` (prelude)
- [ ] Tester la compilation : `cargo build`
- [ ] Tester page 404 : `curl http://localhost:3000/404`
- [ ] Tester erreur template en mode debug
- [ ] Tester page d'accueil par défaut
- [ ] (Optionnel) Personnaliser les templates d'erreur

---

## 📚 Ressources

- **Ancienne architecture** : `/mnt/user-data/uploads/error.rs`
- **Nouvelle ErrorContext** : `/home/claude/final_error_context.rs`
- **Nouveau Error Handler** : `/home/claude/final_error_handler.rs`
- **Analyse comparative** : `/home/claude/analyse_comparative.md`

---

**Auteur** : Migration Runique v0.1.2+
**Date** : 2026-01-22
**Statut** : ✅ Prêt pour implémentation