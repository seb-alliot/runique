# 🔐 Authentification

## Vue d'ensemble

Runique fournit un système d'authentification par session complet et extensible. Il repose sur trois concepts :

- **`RuniqueUser`** — le trait que votre modèle utilisateur doit implémenter
- **Helpers de session** — fonctions pour connecter, déconnecter et lire l'état de l'utilisateur
- **Middlewares Axum** — pour protéger les routes et charger le contexte utilisateur

---

## Modèle utilisateur built-in

Runique inclut un modèle utilisateur prêt à l'emploi, sans aucune configuration.

**Table générée :** `eihwaz_users`

| Champ         | Type     | Description                            |
|---------------|----------|----------------------------------------|
| `id`          | `i32`    | Clé primaire                           |
| `username`    | `String` | Nom d'utilisateur unique               |
| `email`       | `String` | Adresse email                          |
| `password`    | `String` | Hash Argon2 — jamais en clair          |
| `is_active`   | `bool`   | Compte actif                           |
| `is_staff`    | `bool`   | Accès au panneau admin (limité)        |
| `is_superuser`| `bool`   | Accès complet, bypass toutes les règles|
| `roles`       | `JSON`   | Rôles personnalisés ex. `["editor"]`   |
| `created_at`  | datetime | Date de création                       |
| `updated_at`  | datetime | Date de mise à jour                    |

Pour créer le premier superutilisateur :

```bash
runique create-superuser
```

---

## Trait RuniqueUser

Si vous utilisez votre propre modèle utilisateur à la place du built-in, vous devez implémenter `RuniqueUser`.

```rust
use runique::middleware::auth::RuniqueUser;

impl RuniqueUser for users::Model {
    fn user_id(&self) -> i32        { self.id }
    fn username(&self) -> &str      { &self.username }
    fn email(&self) -> &str         { &self.email }
    fn password_hash(&self) -> &str { &self.password }
    fn is_active(&self) -> bool     { self.is_active }
    fn is_staff(&self) -> bool      { self.is_staff }
    fn is_superuser(&self) -> bool  { self.is_superuser }

    // Optionnel — rôles personnalisés
    fn roles(&self) -> Vec<String> {
        self.roles.clone().unwrap_or_default()
    }

    // Optionnel — logique d'accès admin sur mesure
    fn can_access_admin(&self) -> bool {
        self.is_active() && (self.is_staff() || self.is_superuser())
    }
}
```

### Méthodes obligatoires

| Méthode          | Retour      | Description                    |
|------------------|-------------|--------------------------------|
| `user_id()`      | `i32`       | Identifiant unique              |
| `username()`     | `&str`      | Nom d'utilisateur               |
| `email()`        | `&str`      | Adresse email                   |
| `password_hash()`| `&str`      | Hash du mot de passe            |
| `is_active()`    | `bool`      | Compte actif                    |
| `is_staff()`     | `bool`      | Accès admin limité              |
| `is_superuser()` | `bool`      | Accès admin complet             |

### Méthodes avec valeur par défaut

| Méthode            | Défaut                              | Description                  |
|--------------------|-------------------------------------|------------------------------|
| `roles()`          | `vec![]`                            | Rôles personnalisés          |
| `can_access_admin()` | `is_active && (is_staff || is_superuser)` | Logique d'accès admin |

---

## Helpers de session

Importez depuis le module `auth` :

```rust
use runique::middleware::auth::{
    login, login_staff, logout,
    is_authenticated, get_user_id, get_username,
};
```

### Connexion

```rust
// Utilisateur classique — is_staff et is_superuser valent false par défaut
login(&session, user.id, &user.username).await?;

// Utilisateur staff/admin — avec droits et rôles personnalisés
login_staff(
    &session,
    user.id,
    &user.username,
    user.is_staff,
    user.is_superuser,
    user.roles(),
).await?;
```

### Déconnexion

```rust
logout(&session).await?;
```

### Vérifications

```rust
// L'utilisateur est-il connecté ?
if is_authenticated(&session).await {
    // ...
}

// Récupérer l'ID en session
if let Some(user_id) = get_user_id(&session).await {
    // ...
}

// Récupérer le username en session
if let Some(username) = get_username(&session).await {
    // ...
}
```

---

## Variables d'environnement

Ces variables contrôlent les redirections automatiques des middlewares :

| Variable              | Défaut | Description                                          |
|-----------------------|--------|------------------------------------------------------|
| `REDIRECT_ANONYMOUS`  | `/`    | Cible de redirection pour les utilisateurs non connectés |
| `USER_CONNECTED_URL`  | `/`    | Cible de redirection pour les utilisateurs déjà connectés |

---

## Middlewares de protection

### `login_required` — protéger une route

Redirige vers `REDIRECT_ANONYMOUS` si l'utilisateur n'est pas connecté.

```rust
use runique::middleware::auth::login_required;

let protected = Router::new()
    .route("/dashboard", get(dashboard))
    .layer(axum::middleware::from_fn(login_required));
```

### `redirect_if_authenticated` — page login/register

Redirige vers `USER_CONNECTED_URL` si l'utilisateur est déjà connecté. Utile pour éviter qu'un utilisateur connecté accède à `/login`.

```rust
use runique::middleware::auth::redirect_if_authenticated;

let public = Router::new()
    .route("/login", get(login_page).post(login_post))
    .layer(axum::middleware::from_fn(redirect_if_authenticated));
```

### `load_user_middleware` — charger le contexte utilisateur

Injecte un `CurrentUser` dans les extensions de la requête. Permet d'accéder aux informations de l'utilisateur dans vos handlers.

```rust
use runique::middleware::auth::load_user_middleware;

let app = Router::new()
    .route("/profile", get(profile))
    .layer(axum::middleware::from_fn(load_user_middleware));
```

Accès dans un handler :

```rust
use runique::middleware::auth::CurrentUser;
use runique::context::RequestExtensions;

async fn profile(req: RuniqueRequest) -> impl IntoResponse {
    if let Some(user) = req.extensions().current_user() {
        println!("Connecté : {}", user.username);
    }
}
```

---

## CurrentUser

Structure injectée par `load_user_middleware` dans les extensions de requête.

```rust
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub roles: Vec<String>,
}
```

### Méthodes disponibles

```rust
// Vérifier un rôle précis
user.has_role("editor")           // → bool

// Vérifier au moins un rôle parmi une liste
user.has_any_role(&["editor", "moderator"])  // → bool

// Accès à l'admin (is_staff || is_superuser)
user.can_access_admin()           // → bool

// Vérifier une permission admin (is_superuser bypass tout)
user.can_admin(&["editor"])       // → bool
```

---

## Exemple complet — Login / Logout

```rust
use runique::middleware::auth::{login_staff, logout, redirect_if_authenticated};
use runique::utils::password::TextField;
use tower_sessions::Session;

async fn login_post(session: Session, form: Prisme<LoginForm>) -> impl IntoResponse {
    let form = match form {
        Prisme::Valid(f) => f,
        Prisme::Invalid(f) => return render("login.html", context!{ form: f }),
    };

    // 1. Chercher l'utilisateur
    let user = match users::Entity::find_user(&db, &form.username).await {
        Some(u) => u,
        None => return render("login.html", context!{ error: "Identifiants invalides" }),
    };

    // 2. Vérifier le mot de passe
    if !TextField::verify_password(&form.password, &user.password) {
        return render("login.html", context!{ error: "Identifiants invalides" });
    }

    // 3. Ouvrir la session
    login_staff(&session, user.id, &user.username, user.is_staff, user.is_superuser, user.roles()).await.unwrap();

    Redirect::to("/dashboard").into_response()
}

async fn logout_view(session: Session) -> impl IntoResponse {
    logout(&session).await.ok();
    Redirect::to("/login").into_response()
}
```

---

## Authentification pour l'AdminPanel

Voir la documentation [11-Admin.md](11-Admin.md) pour brancher l'authentification au panneau d'administration.

### Avec le User built-in (zéro config)

```rust
use runique::middleware::auth::RuniqueAdminAuth;

.with_admin(|a| a.auth(RuniqueAdminAuth::new()))
```

### Avec un modèle custom

```rust
use runique::middleware::auth::{DefaultAdminAuth, UserEntity};

// 1. Implémenter UserEntity sur votre entité
impl UserEntity for users::Entity {
    type Model = users::Model;

    async fn find_by_username(db: &DatabaseConnection, username: &str) -> Option<Self::Model> {
        users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model> {
        users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await
            .ok()
            .flatten()
    }
}

// 2. Passer DefaultAdminAuth à la config admin
.with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
```
