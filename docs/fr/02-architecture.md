# 🏗️ Architecture

## Vue d'ensemble

Runique 2.0 est organisée en **modules fonctionnels** basés sur la responsabilité:

```
runique/src/
├── config_runique/          # ⚙️ Configuration & Settings
│   ├── config_struct.rs
│   └── mod.rs
├── data_base_runique/       # 🗄️ ORM & Database
│   ├── config.rs
│   ├── orm_wrapper.rs
│   └── mod.rs
├── formulaire/              # 📋 Form System
│   ├── builder_form/
│   ├── utils/
│   └── mod.rs
├── gardefou/                # 🛡️ Middleware (Sécurité)
│   ├── composant_middleware/
│   ├── utils_gardefou/
│   └── mod.rs
├── macro_runique/           # 🎯 Macros Utilitaires
│   ├── context_macro/
│   ├── flash_message/
│   ├── router/
│   └── mod.rs
├── moteur_engine/           # ⚡ Main Engine
│   ├── engine_struct.rs
│   └── mod.rs
├── request_context/         # 📨 Request Context
│   ├── composant_request/
│   ├── tera_tool/
│   ├── request_struct.rs
│   ├── template_context.rs
│   ├── processor.rs
│   └── mod.rs
├── runique_body/            # 🏭 App Builder
│   ├── composant_app/
│   └── mod.rs
├── utils/                   # 🛠️ Utilities
│   ├── generate_token.rs
│   ├── parse_html.rs
│   ├── csp_nonce.rs
│   └── response_helpers.rs
├── lib.rs
└── prelude.rs
```

---

## Concepts Clés

### 1. RuniqueEngine

**État principal** de l'application (remplace l'ancien `AppState`).

```rust
pub struct RuniqueEngine {
    pub db: Arc<DatabaseConnection>,
    pub tera: Arc<Tera>,
    pub config: Arc<RuniqueConfig>,
}
```

**Utilisé par:**
- `RuniqueContext` - Disponible dans les handlers
- Injection d'extensions Axum

### 2. RuniqueContext

**Contexte de requête** injecté dans chaque handler.

```rust
pub struct RuniqueContext {
    pub engine: Arc<RuniqueEngine>,
    pub flash: FlashManager,
}

// Extracteur FromRequestParts
pub async fn my_handler(ctx: RuniqueContext) -> Response {
    ctx.engine.db.clone()      // Arc<DatabaseConnection>
    ctx.engine.tera.clone()    // Arc<Tera>
}
```

### 3. TemplateContext

**Contexte pour templates** avec auto-injection de `debug` et `csrf_token`.

```rust
pub struct TemplateContext {
    pub engine: Arc<RuniqueEngine>,
    pub flash: FlashManager,
    pub csrf_token: String,
}

// Render automatique
template.render("page.html", &context! {
    "title" => "Page"
    // csrf_token et debug injectés automatiquement
})
```

### 4. ExtractForm<T>

**Extracteur Axum** pour les formulaires.

```rust
// Automatiquement:
// 1. Parse le body
// 2. Crée une instance de MyForm
// 3. Injecte le CSRF token
// 4. Remplit les données

pub async fn handler(
    ExtractForm(form): ExtractForm<MyForm>
) -> Response { }
```

---

## Flux de Requête

```
HTTP Request
    ↓
[Middleware Stack - REVERSE ORDER]
    ├→ extension_injection (injecte Tera, Config, Engine, Session)
    ├→ error_handler_middleware
    ├→ flash_middleware
    ├→ csrf_middleware (validation + token generation)
    ├→ sanitize_middleware
    ├→ session_layer (from tower_sessions)
    ↓
[Handler]
    ├→ RuniqueContext injected
    ├→ TemplateContext injected
    ├→ ExtractForm available
    ↓
[Rendering]
    ├→ template.render() auto-injects csrf_token, debug
    ↓
Response HTTP
```

**Important:** Middleware declared first = Executed last!

---

## État Global vs Instance

### ❌ Ancien design (problématique)

```rust
// Formulaire partagé en state
struct AppState {
    form: MyForm,  // ⚠️ Race condition!
}

// Request 1 remplit le form
// Request 2 remplit le form
// Request 3 lit le form → ??? Conflits!
```

### ✅ Nouveau design (correct)

```rust
// Copie par requête
pub async fn handler(
    ExtractForm(form): ExtractForm<MyForm>
) -> Response {
    // Chaque requête = formulaire isolé
    // Zero concurrence
}
```

---

## Modules Détaillés

### config_runique/
Gestion de la configuration:
- Charger depuis `.env`
- Validation des settings
- Builder pattern

### data_base_runique/
Abstraction ORM:
- SeaORM wrapper
- Objects manager (django-like)
- Database connection management

### formulaire/
Système de formulaires:
- RuniqueForm derive macro
- Field types (text, email, textarea, etc.)
- Validation
- ExtractForm extractor

### gardefou/
Middleware de sécurité:
- CSRF protection
- ALLOWED_HOSTS validation
- Login required middleware
- Redirect if authenticated

### macro_runique/
Macros utilitaires:
- `context!` - Créer contexte template
- `success!`, `error!`, `warning!`, `info!` - Flash messages
- `urlpatterns!` - Définir routes

### moteur_engine/
Moteur principal:
- RuniqueEngine struct
- Initialization
- Extension injection

### request_context/
Contexte de requête:
- RuniqueContext extractor
- TemplateContext extractor
- Message extractor
- Tera tool filters

### runique_body/
Application builder:
- RuniqueApp struct
- `.with_database()`
- `.with_routes()`
- `.build()`
- `.run()`

### utils/
Utilitaires divers:
- CSRF token generation
- CSP nonce generation
- Response helpers (json, html, redirect)
- HTML parsing

---

## Injection de Dépendances

Via **Axum Extensions**:

```rust
// Enregistré dans middleware:
extension_injection
    .layer(Extension(engine))
    .layer(Extension(tera))
    .layer(Extension(config))
    .layer(Extension(session))

// Utilisé dans handlers:
pub async fn handler(
    ctx: RuniqueContext,           // Extrait automatiquement
    template: TemplateContext,     // Extrait automatiquement
    session: Session,              // From tower_sessions
) -> Response { }
```

---

## Lifecycle

### App Startup

```rust
#[tokio::main]
async fn main() {
    // 1. Charger config
    let config = RuniqueConfig::from_env()?;
    
    // 2. Créer RuniqueEngine
    let engine = RuniqueEngine::new(config).await?;
    
    // 3. Builder l'app
    let app = RuniqueApp::new(config)
        .with_database().await?          // Arc<DatabaseConnection>
        .with_routes(routes())            // Router
        .build().await?                   // Assemble middleware
        
    // 4. Lancer le serveur
    app.run("127.0.0.1:3000").await?;
}
```

### Request Handling

```
1. Middleware (reverse order)
2. Handler called with extractors
3. Handler returns response
4. Middleware (forward order)
5. HTTP response sent
```

---

## Bonnes Pratiques

1. **Cloner les Arc:**
   ```rust
   let db = ctx.engine.db.clone();
   ```

2. **Formulaires = copies:**
   ```rust
   let form = MyForm::build(ctx.engine.tera.clone());
   // Pas de state partagé
   ```

3. **Templates auto-context:**
   ```rust
   template.render("page.html", &context! {
       "data" => value
       // csrf_token et debug ajoutés automatiquement
   })
   ```

4. **Flash messages:**
   ```rust
   success!(ctx.flash => "Message");
   ```

5. **Middleware order:**
   ```rust
   // Declared first = Executed last!
   .layer(a)  // Executed 3rd
   .layer(b)  // Executed 2nd
   .layer(c)  // Executed 1st (entry point)
   ```

---

## Prochaines étapes

→ [**Configuration**](./03-configuration.md)
