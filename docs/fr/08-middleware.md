# 🛡️ Middleware & Sécurité

## Stack Middleware

Le middleware s'exécute en **ORDRE INVERSE** de déclaration. Runique configure automatiquement le stack via le builder:

```rust
use runique::prelude::*;

// Le builder configure automatiquement le stack de middlewares
let app = RuniqueAppBuilder::new(config)
    .with_routes(routes)
    .with_error_handler(true)      // Optionnel
    .with_sanitize(true)           // Optionnel
    .build()
    .await?;

// Stack appliqué automatiquement (ordre inverse):
// 1. RequestExtensions injection (Tera, Config, Engine)
// 2. Static files (runique internal)
// 3. Error handler (si activé)
// 4. Sessions (avec MemoryStore par défaut)
// 5. CSRF protection
// 6. Sanitize (si activé)
// 7. Routes utilisateur
```

---

## CSRF Protection

### Automatique pour POST/PUT/PATCH/DELETE

Le middleware CSRF est configuré automatiquement par `RuniqueAppBuilder`. Il génère et vérifie les tokens pour toutes les requêtes POST/PUT/PATCH/DELETE.

**Fonctionnement:**
- Token généré avec contexte utilisateur (user_id si authentifié, sinon session_id)
- Double Submit Cookie pattern avec masquage du token
- Vérification automatique via header `X-CSRF-Token` ou champ de formulaire
- Intégration avec `Prisme` pour validation des formulaires

### Dans les Formulaires

```html
<form method="post">
    {% csrf %}
    <!-- Génère automatiquement: -->
    <!-- <input type="hidden" name="csrf_token" value="masked_token"> -->
</form>
```

**Note:** Le token est automatiquement masqué (Double Submit Cookie pattern) et vérifié par le middleware + `Prisme`.

### Endpoints AJAX

```javascript
// Le token est automatiquement envoyé dans le header de réponse
fetch('/api/users', {
    method: 'POST',
    headers: {
        'X-CSRF-Token': document.querySelector('[name="csrf_token"]').value,
        'Content-Type': 'application/json'
    },
    body: JSON.stringify({username: "john"})
});
```

---

## ALLOWED_HOSTS

Protection contre Host Header Injection (inspiré de Django):

```rust
use runique::middleware::AllowedHostsValidator;

// Configuration dans RuniqueConfig
let validator = AllowedHostsValidator::from_settings(&config);

// Le validateur supporte:
// - Correspondance exacte: "exemple.com"
// - Wildcard complet: "*" (dangereux en production!)
// - Wildcard sous-domaines: ".exemple.com" match "api.exemple.com", etc.

impl AllowedHostsValidator {
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.debug { return true; }

        let host = host.split(':').next().unwrap_or(host);

        self.allowed_hosts.iter().any(|allowed| {
            if allowed == "*" {
                true
            } else if allowed.starts_with('.') {
                // Sous-domaines
                host == &allowed[1..] ||
                (host.ends_with(allowed) && host.as_bytes()[host.len() - allowed.len()] == b'.')
            } else {
                allowed == host
            }
        })
    }
}
```

**Usage:**
```rust
// Validation automatique dans les requêtes
match validator.validate(&headers) {
    Ok(_) => { /* OK */ }
    Err((status, msg)) => return (status, msg).into_response(),
}
```

---

## Sessions

### Configuration

```rust
use tower_sessions::{SessionManagerLayer, MemoryStore, Expiry};
use time::Duration;

// Le builder configure automatiquement les sessions
let session_layer = SessionManagerLayer::new(MemoryStore::default())
    .with_secure(!config.debug)          // HTTPS en production
    .with_http_only(!config.debug)        // Pas d'accès JS
    .with_expiry(Expiry::OnInactivity(Duration::hours(2)));

// Appliqué automatiquement dans RuniqueAppBuilder
```

### Utiliser les Sessions

```rust
use tower_sessions::Session;

// Méthode 1: Extraction directe de Session
async fn login(
    session: Session,
    Form(credentials): Form<LoginForm>,
) -> Response {
    // Note: authenticate() est un exemple - cette fonction sera fournie dans une future version
    if let Ok(Some(user)) = authenticate(&credentials).await {
        session.insert("user_id", user.id).await.ok();
        session.insert("username", &user.username).await.ok();

        Redirect::to("/dashboard").into_response()
    } else {
        Redirect::to("/login?error=1").into_response()
    }
}

async fn dashboard(session: Session) -> Response {
    let user_id: Option<i32> = session.get("user_id").await.ok().flatten();
    match user_id {
        Some(id) => format!("User ID: {}", id).into_response(),
        None => Redirect::to("/login").into_response()
    }
}

// Méthode 2: Accès via TemplateContext (session est inclus)
async fn profile(template: TemplateContext) -> Response {
    let user_id: Option<i32> = template.session.get("user_id").await.ok().flatten();

    template.context.insert("user_id", &user_id);
    template.render("profile.html")
}

async fn logout(session: Session) -> Response {
    session.flush().await.ok();
    Redirect::to("/").into_response()
}
```

---

## CSP - Content Security Policy

### Génération de Nonce

```rust
use runique::utils::csp_nonce::CspNonce;

// Le nonce est automatiquement généré et injecté dans TemplateContext
async fn index(
    template: TemplateContext,
) -> Response {
    // template.csp_nonce est déjà disponible
    template.render("index.html")
}
```

### Dans les Templates

```html
<!DOCTYPE html>
<html>
<head>
    <style nonce="{{ csp_nonce }}">
        body { color: #333; }
    </style>
</head>
<body>
    <script nonce="{{ csp_nonce }}">
        console.log("Script sécurisé");
    </script>
</body>
</html>
```

### Configuration CSP

```rust
use runique::middleware::CspConfig;

// Profils prédéfinis
let strict_csp = CspConfig::strict();      // Strict avec nonces
let permissive = CspConfig::permissive();  // Plus souple
let default = CspConfig::default();        // Équilibré

// Générer le header
let nonce = CspNonce::generate();
let header_value = csp_config.to_header_value(Some(nonce.as_str()));

// Exemple de header généré:
// "default-src 'self'; script-src 'self' 'nonce-ABC123'; style-src 'self' 'nonce-ABC123'"
```

---

## Authentification Personnalisée

```rust
// Middleware d'authentification
pub async fn auth_middleware(
    template: TemplateContext,
) -> Result<Response> {
    let user_id: Option<i32> = template.session.get("user_id").ok().flatten();

    if user_id.is_none() && !is_public_route(request.uri()) {
        return Err(Redirect::to("/login").into_response());
    }

    Ok(next.run(request).await)
}

// Extractor personnalisé
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
}

#[async_trait]
impl FromRequest for CurrentUser {
    async fn from_request(request: &mut Request) -> Result<Self> {
        let session: Session = request.extract().await?;

        let user_id: i32 = session.get("user_id")?
            .ok_or("Not authenticated")?;

        let username = session.get("username")?
            .ok_or("Not authenticated")?;

        Ok(CurrentUser { id: user_id, username })
    }
}

// Utiliser:
async fn dashboard(user: CurrentUser) -> Response {
    format!("Bienvenue, {}!", user.username)
}
```

---

## Prochaines étapes

← [**ORM & Base de Données**](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/fr/09-flash-messages.md) →
