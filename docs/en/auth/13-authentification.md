# 🔐 Authentication

## Overview

Runique provides a complete, extensible session-based authentication system built on three concepts:

- **`RuniqueUser`** — the trait your user model must implement
- **Session helpers** — functions to log in, log out, and read user state
- **Axum middlewares** — to protect routes and load user context into requests

---

## Built-in User Model

Runique ships with a ready-to-use user model that requires no configuration.

**Generated table:** `eihwaz_users`

| Field          | Type     | Description                             |
|----------------|----------|-----------------------------------------|
| `id`           | `i32`    | Primary key                             |
| `username`     | `String` | Unique username                         |
| `email`        | `String` | Email address                           |
| `password`     | `String` | Argon2 hash — never stored in plain text|
| `is_active`    | `bool`   | Account active                          |
| `is_staff`     | `bool`   | Admin panel access (limited)            |
| `is_superuser` | `bool`   | Full access, bypasses all rules         |
| `roles`        | `JSON`   | Custom roles e.g. `["editor"]`          |
| `created_at`   | datetime | Creation timestamp                      |
| `updated_at`   | datetime | Last update timestamp                   |

To create the first superuser:

```bash
runique create-superuser
```

---

## RuniqueUser Trait

If you use your own user model instead of the built-in one, you must implement `RuniqueUser`.

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

    // Optional — custom roles
    fn roles(&self) -> Vec<String> {
        self.roles.clone().unwrap_or_default()
    }

    // Optional — custom admin access logic
    fn can_access_admin(&self) -> bool {
        self.is_active() && (self.is_staff() || self.is_superuser())
    }
}
```

### Required Methods

| Method           | Return      | Description            |
|------------------|-------------|------------------------|
| `user_id()`      | `i32`       | Unique identifier       |
| `username()`     | `&str`      | Username                |
| `email()`        | `&str`      | Email address           |
| `password_hash()`| `&str`      | Password hash           |
| `is_active()`    | `bool`      | Account is active       |
| `is_staff()`     | `bool`      | Limited admin access    |
| `is_superuser()` | `bool`      | Full admin access       |

### Methods With Default Implementation

| Method               | Default                                       | Description            |
|----------------------|-----------------------------------------------|------------------------|
| `roles()`            | `vec![]`                                      | Custom roles           |
| `can_access_admin()` | `is_active && (is_staff \|\| is_superuser)`   | Admin access logic     |

---

## Session Helpers

Import from the `auth` module:

```rust
use runique::middleware::auth::{
    login, login_staff, logout,
    is_authenticated, get_user_id, get_username,
};
```

### Login

```rust
// Regular user — is_staff and is_superuser default to false
login(&session, user.id, &user.username).await?;

// Staff/admin user — with explicit rights and custom roles
login_staff(
    &session,
    user.id,
    &user.username,
    user.is_staff,
    user.is_superuser,
    user.roles(),
).await?;
```

### Logout

```rust
logout(&session).await?;
```

### Checks

```rust
// Is the user authenticated?
if is_authenticated(&session).await {
    // ...
}

// Get user ID from session
if let Some(user_id) = get_user_id(&session).await {
    // ...
}

// Get username from session
if let Some(username) = get_username(&session).await {
    // ...
}
```

---

## Environment Variables

These variables control automatic redirects in the middlewares:

| Variable             | Default | Description                                          |
|----------------------|---------|------------------------------------------------------|
| `REDIRECT_ANONYMOUS` | `/`     | Redirect target for unauthenticated users            |
| `USER_CONNECTED_URL` | `/`     | Redirect target for already-authenticated users      |

---

## Protection Middlewares

### `login_required` — protect a route

Redirects to `REDIRECT_ANONYMOUS` if the user is not logged in.

```rust
use runique::middleware::auth::login_required;

let protected = Router::new()
    .route("/dashboard", get(dashboard))
    .layer(axum::middleware::from_fn(login_required));
```

### `redirect_if_authenticated` — login/register pages

Redirects to `USER_CONNECTED_URL` if the user is already logged in. Useful to prevent authenticated users from reaching `/login`.

```rust
use runique::middleware::auth::redirect_if_authenticated;

let public = Router::new()
    .route("/login", get(login_page).post(login_post))
    .layer(axum::middleware::from_fn(redirect_if_authenticated));
```

### `load_user_middleware` — load user context

Injects a `CurrentUser` into request extensions, making user information available in all handlers down the chain.

```rust
use runique::middleware::auth::load_user_middleware;

let app = Router::new()
    .route("/profile", get(profile))
    .layer(axum::middleware::from_fn(load_user_middleware));
```

Access in a handler:

```rust
use runique::middleware::auth::CurrentUser;
use runique::context::RequestExtensions;

async fn profile(req: RuniqueRequest) -> impl IntoResponse {
    if let Some(user) = req.extensions().current_user() {
        println!("Logged in as: {}", user.username);
    }
}
```

---

## CurrentUser

Struct injected by `load_user_middleware` into request extensions.

```rust
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub roles: Vec<String>,
}
```

### Available Methods

```rust
// Check a specific role
user.has_role("editor")                      // → bool

// Check for at least one role from a list
user.has_any_role(&["editor", "moderator"])  // → bool

// Admin panel access (is_staff || is_superuser)
user.can_access_admin()                      // → bool

// Check admin permission (is_superuser bypasses everything)
user.can_admin(&["editor"])                  // → bool
```

---

## Full Example — Login / Logout

```rust
use runique::middleware::auth::{login_staff, logout, redirect_if_authenticated};
use runique::utils::password::TextField;
use tower_sessions::Session;

async fn login_post(session: Session, form: Prisme<LoginForm>) -> impl IntoResponse {
    let form = match form {
        Prisme::Valid(f) => f,
        Prisme::Invalid(f) => return render("login.html", context!{ form: f }),
    };

    // 1. Find the user
    let user = match users::Entity::find_user(&db, &form.username).await {
        Some(u) => u,
        None => return render("login.html", context!{ error: "Invalid credentials" }),
    };

    // 2. Verify the password
    if !TextField::verify_password(&form.password, &user.password) {
        return render("login.html", context!{ error: "Invalid credentials" });
    }

    // 3. Open the session
    login_staff(&session, user.id, &user.username, user.is_staff, user.is_superuser, user.roles()).await.unwrap();

    Redirect::to("/dashboard").into_response()
}

async fn logout_view(session: Session) -> impl IntoResponse {
    logout(&session).await.ok();
    Redirect::to("/login").into_response()
}
```

---

## Authentication for the AdminPanel

See the [11-Admin.md](11-Admin.md) documentation to connect authentication to the admin panel.

### With the built-in User (zero config)

```rust
use runique::middleware::auth::RuniqueAdminAuth;

.with_admin(|a| a.auth(RuniqueAdminAuth::new()))
```

### With a custom model

```rust
use runique::middleware::auth::{DefaultAdminAuth, UserEntity};

// 1. Implement UserEntity on your entity
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

// 2. Pass DefaultAdminAuth to the admin config
.with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
```
