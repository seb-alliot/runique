# Password configuration

[← Builder](/docs/en/configuration/builder)

---

`PasswordConfig` defines the hashing and verification strategy for the entire application. It is initialized **once** at startup via `password_init()`.

## Initialization in `main.rs`

```rust
use runique::prelude::{password_init, PasswordConfig, Manual};

#[tokio::main]
async fn main() {
    // Argon2 automatic mode (recommended default)
    password_init(PasswordConfig::auto());

    RuniqueApp::builder(config)
        // ...
        .run()
        .await;
}
```

> If `password_init()` is not called, Argon2 is used by default.

---

## Available modes

### `Auto` — Automatic hashing (recommended)

Runique detects whether a value is already hashed and only hashes it once. The algorithm is configurable.

```rust
// Argon2 by default
password_init(PasswordConfig::auto());

// Choose the algorithm
password_init(PasswordConfig::auto_with(Manual::Bcrypt));
password_init(PasswordConfig::auto_with(Manual::Scrypt));
```

Supported algorithms: `Manual::Argon2`, `Manual::Bcrypt`, `Manual::Scrypt`.

### `Manual` — Explicit hashing

Hashing is **not** applied automatically in `finalize()`. The developer calls `hash()` manually.

```rust
password_init(PasswordConfig::manual(Manual::Argon2));
```

> Use this when you need precise control over when and how the password is hashed.

### `Delegated` — External authentication (OAuth / SSO)

No password is managed by Runique. Authentication is delegated to an external provider.

```rust
use runique::prelude::External;

password_init(PasswordConfig::oauth(External::GoogleOAuth));
password_init(PasswordConfig::oauth(External::Microsoft));
password_init(PasswordConfig::oauth(External::Ldap("ldap://...".to_string())));
```

Available providers: `GoogleOAuth`, `Microsoft`, `Apple`, `Ldap(url)`, `Saml(url)`, `Custom { name, authorize_url, token_url }`.

### `Custom` — Custom handler

Implement the `PasswordHandler` trait to plug in your own hashing/verification logic.

```rust
use runique::prelude::{PasswordHandler, PasswordConfig};

struct MyHasher;

impl PasswordHandler for MyHasher {
    fn name(&self) -> &str { "my_hasher" }
    fn transform(&self, input: &str) -> Result<String, String> {
        Ok(format!("hashed:{}", input))
    }
    fn verify(&self, input: &str, stored: &str) -> bool {
        stored == format!("hashed:{}", input)
    }
    // ...
}

password_init(PasswordConfig::custom(MyHasher));
```

---

## Usage in code

```rust
use runique::prelude::{hash, verify};

// Hash a password (uses the global config)
let hashed = hash("my_password")?;

// Verify a password against a stored hash
let ok = verify("my_password", &user.password_hash);
```

These functions automatically use the `PasswordConfig` initialized at startup.

---

## In forms

`TextField::password()` fields are automatically hashed in `finalize()` in `Auto` mode. In `Manual` or `Delegated` mode, no automatic hashing occurs.

See → [Field types — TextField](/docs/en/formulaire/fields)

---

← [**Builder**](/docs/en/configuration/builder)
