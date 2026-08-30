# User Model

## Built-in Model

Runique includes a ready-to-use user model that requires no configuration.

**Generated table:** `eihwaz_users`

| Field | Type | Description |
|---------------|----------|----------------------------------------|
| `id` | `Pk` | Primary key (`i32` by default, `i64` with `big-pk`, `Uuid` with `pk-uuid`) |
| `username` | `String` | Unique username |
| `email` | `String` | Email address |
| `password` | `String` | Argon2 hash — never stored in plain text |
| `is_active` | `bool` | Account active |
| `is_staff` | `bool` | Admin panel access (limited) |
| `is_superuser` | `bool` | Full access, bypasses all rules |
| `created_at` | datetime | Creation timestamp |
| `updated_at` | datetime | Last update timestamp |

To create the first superuser:

```bash
runique create-superuser
```

### i64 or UUID primary key

By default, the primary key is `i32`. To switch to `i64` or `Uuid` — **only one feature at a
time**, enabling both together is a deliberate compile-time error:

```toml
# project Cargo.toml
runique = { version = "...", features = ["big-pk"] }    # Pk = i64
runique = { version = "...", features = ["pk-uuid"] }   # Pk = Uuid (Uuid::now_v7())
```

The choice must be made before the first migration — see
[`model!` DSL & `extend!`](/docs/en/model/dsl) for full details (compatible types, FK
constraints, switching mode after the fact).

---

## RuniqueUser Trait

If you use your own user model instead of the built-in one, you must implement `RuniqueUser`.

```rust
use runique::prelude::*;

impl RuniqueUser for users::Model {
    fn user_id(&self) -> Pk      { self.id }
    fn username(&self) -> &str       { &self.username }
    fn email(&self) -> &str          { &self.email }
    fn password_hash(&self) -> &str  { &self.password }
    fn is_active(&self) -> bool      { self.is_active }
    fn is_staff(&self) -> bool       { self.is_staff }
    fn is_superuser(&self) -> bool   { self.is_superuser }

    // Optional — custom admin access logic
    fn can_access_admin(&self) -> bool {
        self.is_active() && (self.is_staff() || self.is_superuser())
    }
}
```

### Required Methods

| Method | Return | Description |
|------------------|-------------|--------------------------------|
| `user_id()` | `Pk` | Unique identifier |
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
| [Session helpers](/docs/en/auth/session) | `login`, `auth_login`, `logout` |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
