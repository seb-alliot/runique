# 🏗️ Architecture

## Vue d'ensemble

Runique 1.1.11 est organisée en **modules fonctionnels** basés sur la responsabilité:

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

### 2. TemplateContext

**Contexte de template** injecté dans chaque handler pour le rendu.

```rust
pub struct TemplateContext {
    pub context: Context,
    // Access to Tera for rendering
}

// Extracteur FromRequestParts
pub async fn my_handler(
    mut template: TemplateContext,
) -> Response {
    template.context.insert("title", "Bienvenu sur Runique");
    template.render("vue.html")
}
```

### 3. TemplateContext

**Contexte pour templates** avec auto-injection de la session, CSRF token et CSP nonce.

```rust
pub struct TemplateContext {
    pub engine: Arc<RuniqueEngine>,
    pub session: Session,
    pub notices: Message,
    pub messages: Vec<FlashMessage>,
    pub csrf_token: CsrfToken,
    pub csp_nonce: String,
    pub context: Context,
}

// Render automatique
    context_update!(template => {
        "title" => "Votre titre ici ",
        "form" => &form,
    });
    template.render("vue.html")

```

### 4. Prisme<T> - Extracteur de Formulaire

**Extracteur Axum** pour les formulaires avec validation et injection CSRF automatiques.

```rust
// Automatiquement:
// 1. Parse le body
// 2. Crée une instance de MyForm
// 3. Injecte le CSRF token
// 4. Remplit les données

pub async fn handler(
    mut template: TemplateContext,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let db = template.engine.db.clone();
    if form.is_valid().await {
        // Traiter le formulaire
        match form.save(&db).await {
            Ok(_) => { /* succès */ },
            Err(e) => { /* erreur */ }
        }
    }
    Ok(template.render("form.html"))
}
```

---


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
    Prisme(form): Prisme<MyForm>
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
- Prisme extractor

### middleware/
Middleware de sécurité:
- CSRF protection
- ALLOWED_HOSTS validation
- Nonce
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

⚠️  le nonce est ajouter manuellement dans le builder de votre application via

```rust

.layer(middleware::from_fn_with_state(
    engine.clone(),
    security_headers_middleware,
))
```
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
    template: TemplateContext,
) -> AppResult<Response> { }
```

---

## Lifecycle

### App Startup

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration de l'application
    let config = RuniqueConfig::from_env();

    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    // Créer et lancer l'application
    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .build()
        .await?
        .run()
        .await?;

    Ok(())
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
       let db = template.engine.db.clone();
   ```

2. **Formulaires = copies:**
   ```rust
       let form = template.form::<Form>();

   // Pas de state partagé
   ```

3. **Templates auto-context:**
   ```rust
   template.context.insert("data", value);
   template.render("page.html")
   // csrf_token auto-injecté dans le contexte
   ```

4. **Flash messages:**
   ```rust
   Message(mut messages): Message,
   messages.success(format!("Bienvenue {}, votre compte a été créé !", user.username));
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

→ [**Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/fr/03-configuration.md)
