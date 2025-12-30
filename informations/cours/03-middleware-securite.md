# Cours 3 : Middleware de Sécurité

## 🎯 Objectif

Implémenter des middlewares de sécurité : CSRF, CSP, validation des hosts, sanitization.

## 📚 Concepts de base

### Qu'est-ce qu'un middleware ?

Un middleware intercepte les requêtes HTTP avant qu'elles n'atteignent les handlers, et peut :
- Modifier la requête
- Modifier la réponse
- Bloquer la requête
- Ajouter des headers

**Flux :**
```
Requête → Middleware 1 → Middleware 2 → Handler → Middleware 2 → Middleware 1 → Réponse
```

## 🔧 Implémentations

### 1. CSRF Protection

#### Concept

Le CSRF (Cross-Site Request Forgery) protège contre les attaques où un site malveillant fait des requêtes en votre nom.

**Solution :** Token unique par session.

#### Implémentation

```rust
use tower_sessions::Session;
use axum::{middleware::Next, Request, Response};

pub async fn csrf_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();

    // 1. Vérifier si la méthode nécessite CSRF
    let requires_csrf = matches!(
        method,
        Method::POST | Method::PUT | Method::DELETE | Method::PATCH
    );

    if !requires_csrf {
        // GET, HEAD, OPTIONS n'ont pas besoin de CSRF
        return next.run(req).await;
    }

    // 2. Récupérer le token de session
    let session = req.extensions().get::<Session>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let session_token = session
        .get::<String>(CSRF_TOKEN_KEY)
        .await
        .ok()
        .flatten();

    // 3. Récupérer le token de la requête
    let request_token = req.headers()
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok());

    // 4. Vérifier la correspondance
    match (session_token, request_token) {
        (Some(st), Some(rt)) if constant_time_compare(&st, &rt) => {
            // Token valide, continuer
            next.run(req).await
        },
        _ => {
            // Token invalide, rejeter
            (StatusCode::BAD_REQUEST, "Invalid CSRF Token").into_response()
        }
    }
}
```

#### Génération de token

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_token(secret_key: &str, session_id: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");

    mac.update(b"rusti.middleware.csrf");
    mac.update(session_id.as_bytes());

    // Ajouter un timestamp pour l'unicité
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    mac.update(timestamp.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}
```

#### Comparaison en temps constant

```rust
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.bytes()
        .zip(b.bytes())
        .map(|(x, y)| x ^ y)
        .fold(0, |acc, x| acc | x) == 0
}
```

**Pourquoi ?** Évite les attaques par timing.

### 2. Content Security Policy (CSP)

#### Concept

CSP limite les ressources qu'un navigateur peut charger, protégeant contre XSS.

#### Implémentation

```rust
pub async fn security_headers_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // CSP
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'")
    );

    // X-Content-Type-Options
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff")
    );

    // X-Frame-Options
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY")
    );

    response
}
```

### 3. Validation des Hosts Autorisés

#### Concept

Protège contre les attaques Host Header Injection.

#### Implémentation

```rust
pub async fn allowed_hosts_middleware(
    Extension(settings): Extension<Arc<Settings>>,
    request: Request,
    next: Next,
) -> Response {
    let validator = AllowedHostsValidator::from_settings(&settings);

    // Valider le header Host
    if let Err((status, message)) = validator.validate(request.headers()) {
        return (status, message).into_response();
    }

    next.run(request).await
}

impl AllowedHostsValidator {
    pub fn is_host_allowed(&self, host: &str) -> bool {
        // Mode debug : tout autoriser
        if self.debug {
            return true;
        }

        // Retirer le port
        let host = host.split(':').next().unwrap_or(host);

        // Vérifier dans la liste
        self.allowed_hosts.iter().any(|allowed| {
            if allowed == "*" {
                true  // Wildcard complet
            } else if allowed.starts_with('.') {
                // Wildcard sous-domaine: ".example.com"
                host == &allowed[1..] ||
                (host.ends_with(allowed) &&
                 host.as_bytes()[host.len() - allowed.len()] == b'.')
            } else {
                allowed == host  // Correspondance exacte
            }
        })
    }
}
```

### 4. Sanitization

#### Concept

Nettoie les entrées utilisateur pour éviter XSS et injections.

#### Implémentation

```rust
pub fn auto_sanitize(input: &str) -> String {
    // 1. Échapper les caractères HTML
    let mut sanitized = String::with_capacity(input.len());

    for c in input.chars() {
        match c {
            '<' => sanitized.push_str("&lt;"),
            '>' => sanitized.push_str("&gt;"),
            '&' => sanitized.push_str("&amp;"),
            '"' => sanitized.push_str("&quot;"),
            '\'' => sanitized.push_str("&#x27;"),
            '/' => sanitized.push_str("&#x2F;"),
            _ => sanitized.push(c),
        }
    }

    sanitized
}
```

#### Middleware de sanitization

```rust
pub async fn sanitize_middleware(
    State(settings): State<Arc<Settings>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Si désactivé, passer
    if !settings.sanitize_inputs {
        return next.run(request).await;
    }

    // Récupérer le Content-Type
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Sanitizer selon le type
    if content_type.contains("application/x-www-form-urlencoded") {
        request = sanitize_form_urlencoded(request).await;
    } else if content_type.contains("application/json") {
        request = sanitize_json(request).await;
    }

    next.run(request).await
}
```

## 🎓 Exercices

### Exercice 1 : Améliorer CSRF

Ajoutez la vérification du token dans le body des formulaires :
```rust
// Chercher dans: <input name="csrf_token" value="...">
```

### Exercice 2 : CSP dynamique

Créez une configuration CSP flexible :
```rust
struct CspConfig {
    default_src: Vec<String>,
    script_src: Vec<String>,
    // ...
}
```

### Exercice 3 : Rate Limiting

Implémentez un middleware de rate limiting basique :
```rust
// Limiter à 100 requêtes par minute par IP
```

## 💡 Bonnes pratiques

1. **Fail secure** : En cas de doute, rejeter
2. **Temps constant** : Utilisez des comparaisons en temps constant pour les secrets
3. **Headers sécurisés** : Toujours définir les headers de sécurité
4. **Validation stricte** : Valider toutes les entrées utilisateur

## 🔗 Ressources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CSP Reference](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
