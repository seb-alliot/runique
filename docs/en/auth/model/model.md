# User Model

## Built-in Model

Runique includes a ready-to-use user model that requires no configuration.

**Generated table:** `eihwaz_users`

| Field | Type | Description |
|---------------|----------|----------------------------------------|
| `id` | `i32` | Primary key |
| `username` | `String` | Unique username |
| `email` | `String` | Email address |
| `password` | `String` | Argon2 hash — never stored in plain text |
| `is_active` | `bool` | Account active |
| `is_staff` | `bool` | Admin panel access (limited) |
| `is_superuser` | `bool` | Full access, bypasses all rules |
| `roles` | `JSON` | Custom roles e.g. `["editor"]` |
| `created_at` | datetime | Creation timestamp |
| `updated_at` | datetime | Last update timestamp |

To create the first superuser:

```bash
runique create-superuser
```

To access the model from your code:

```rust
use runique::prelude::user::{Model, ActiveModel};
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

| Method | Return | Description |
|------------------|-------------|--------------------------------|
| `user_id()` | `i32` | Unique identifier |
| `username()` | `&str` | Username |
| `email()` | `&str` | Email address |
| `password_hash()` | `&str` | Password hash |
| `is_active()` | `bool` | Account is active |
| `is_staff()` | `bool` | Limited admin access |
| `is_superuser()` | `bool` | Full admin access |

### Methods With Default Implementation

| Method | Default | Description |
|--------------------|-------------------------------------|------------------------------|
| `roles()` | `vec![]` | Custom roles |
| `can_access_admin()` | `is_active && (is_staff \|\| is_superuser)` | Admin access logic |

---

## See also

| Section | Description |
| --- | --- |
| [Session helpers](/docs/en/auth/session) | `login`, `logout`, checks |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
