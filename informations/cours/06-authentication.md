# Cours 6 : Authentification et Sessions

## 🎯 Objectif

Créer un système d'authentification avec sessions et middlewares de protection.

## 📚 Concepts de base

### Architecture

```
Session (tower-sessions)
  ├── Stockage des données utilisateur
  ├── ID de session unique
  └── Expiration automatique

Middleware
  ├── login_required : Protège les routes
  └── redirect_if_authenticated : Redirige si déjà connecté
```

## 🔧 Implémentation étape par étape

### Étape 1 : Vérifier l'authentification

```rust
use tower_sessions::Session;

pub async fn is_authenticated(session: &Session) -> bool {
    session
        .get::<i32>("user_id")
        .await
        .ok()
        .flatten()
        .is_some()
}
```

### Étape 2 : Récupérer l'ID utilisateur

```rust
pub async fn get_user_id(session: &Session) -> Option<i32> {
    session
        .get::<i32>("user_id")
        .await
        .ok()
        .flatten()
}
```

### Étape 3 : Middleware login_required

```rust
use axum::{middleware::Next, Request, Response, response::Redirect};

pub async fn login_required(
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // Vérifier si authentifié
    if is_authenticated(&session).await {
        // Utilisateur connecté, continuer
        next.run(request).await
    } else {
        // Non connecté, rediriger vers login
        Redirect::to("/login").into_response()
    }
}
```

### Étape 4 : Middleware redirect_if_authenticated

```rust
pub async fn redirect_if_authenticated(
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // Si déjà connecté, rediriger vers dashboard
    if is_authenticated(&session).await {
        Redirect::to("/dashboard").into_response()
    } else {
        // Pas connecté, permettre l'accès (page login)
        next.run(request).await
    }
}
```

### Étape 5 : Fonction de login

```rust
pub async fn login_user(
    session: &Session,
    user_id: i32,
    username: String,
) -> Result<(), String> {
    // Stocker les données utilisateur dans la session
    session.insert("user_id", user_id).await
        .map_err(|_| "Erreur lors de la création de session".to_string())?;

    session.insert("username", username).await
        .map_err(|_| "Erreur lors de la création de session".to_string())?;

    Ok(())
}
```

### Étape 6 : Fonction de logout

```rust
pub async fn logout_user(session: &Session) {
    // Supprimer les données de session
    let _ = session.remove::<i32>("user_id").await;
    let _ = session.remove::<String>("username").await;
}
```

### Étape 7 : Handler de login

```rust
use axum::{Form, response::Redirect};
use std::collections::HashMap;

async fn login_handler(
    session: Session,
    Form(data): Form<HashMap<String, String>>,
) -> Response {
    let username = data.get("username").unwrap_or(&String::new());
    let password = data.get("password").unwrap_or(&String::new());

    // TODO: Vérifier les credentials dans la DB
    // Pour l'exemple, on simule :
    if username == "admin" && password == "password" {
        // Utilisateur trouvé, créer la session
        if let Err(_) = login_user(&session, 1, username.clone()).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Erreur de session").into_response();
        }

        Redirect::to("/dashboard").into_response()
    } else {
        // Credentials invalides
        Redirect::to("/login?error=invalid").into_response()
    }
}
```

### Étape 8 : Utilisation dans l'application

```rust
use axum::middleware;

let app = Router::new()
    // Routes publiques
    .route("/login", post(login_handler))
    .route("/register", post(register_handler))

    // Routes protégées
    .route("/dashboard", get(dashboard_handler))
    .route("/profile", get(profile_handler))
    .layer(middleware::from_fn(login_required))

    // Page login (redirige si déjà connecté)
    .route("/login", get(login_page))
    .layer(middleware::from_fn(redirect_if_authenticated));
```

## 🎓 Exercices

### Exercice 1 : Vérification de permissions

Implémentez un système de permissions :
```rust
pub async fn has_permission(
    session: &Session,
    permission: &str
) -> bool {
    // Récupérer les permissions depuis la DB
    // Vérifier si l'utilisateur a la permission
}
```

### Exercice 2 : Remember me

Ajoutez une fonctionnalité "Se souvenir de moi" :
```rust
// Cookie avec expiration longue (30 jours)
session_layer.with_expiry(Expiry::OnInactivity(Duration::days(30)));
```

### Exercice 3 : Rate limiting sur login

Limitez les tentatives de login :
```rust
// Max 5 tentatives par IP par heure
```

## 💡 Bonnes pratiques

1. **Sessions sécurisées** : Utilisez HTTPS en production
2. **Expiration** : Définissez une expiration raisonnable
3. **Validation** : Validez toujours les credentials côté serveur
4. **Logout** : Permettez toujours le logout

## 🔗 Ressources

- [tower-sessions](https://docs.rs/tower-sessions/)
- [OWASP Session Management](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
