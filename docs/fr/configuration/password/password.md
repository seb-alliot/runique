# Configuration des mots de passe

[← Builder](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/builder/builder.md)

---

`PasswordConfig` configure la stratégie de hachage et de vérification des mots de passe pour toute l'application. Elle s'initialise **une seule fois** au démarrage via `password_init()`.

## Initialisation dans `main.rs`

```rust
use runique::prelude::{password_init, PasswordConfig, Manual};

#[tokio::main]
async fn main() {
    // Argon2 automatique (défaut recommandé)
    password_init(PasswordConfig::auto());

    RuniqueApp::new()
        // ...
        .run()
        .await;
}
```

> Si `password_init()` n'est pas appelé, Argon2 est utilisé par défaut.

---

## Modes disponibles

### `Auto` — Hachage automatique (recommandé)

Runique détecte si la valeur est déjà hachée et n'applique le hachage qu'une seule fois. L'algorithme est configurable.

```rust
// Argon2 par défaut
password_init(PasswordConfig::auto());

// Choisir l'algorithme
password_init(PasswordConfig::auto_with(Manual::Bcrypt));
password_init(PasswordConfig::auto_with(Manual::Scrypt));
```

Algorithmes supportés : `Manual::Argon2`, `Manual::Bcrypt`, `Manual::Scrypt`.

### `Manual` — Hachage explicite

Le hachage n'est **pas** appliqué automatiquement dans `finalize()`. C'est le développeur qui appelle `hash()` manuellement.

```rust
password_init(PasswordConfig::manual(Manual::Argon2));
```

> À utiliser si tu veux contrôler précisément quand et comment le mot de passe est haché.

### `Delegated` — Authentification externe (OAuth / SSO)

Aucun mot de passe n'est géré par Runique. L'authentification est déléguée à un fournisseur externe.

```rust
use runique::prelude::{External};

password_init(PasswordConfig::oauth(External::GoogleOAuth));
password_init(PasswordConfig::oauth(External::Microsoft));
password_init(PasswordConfig::oauth(External::Ldap("ldap://...".to_string())));
```

Fournisseurs disponibles : `GoogleOAuth`, `Microsoft`, `Apple`, `Ldap(url)`, `Saml(url)`, `Custom { name, authorize_url, token_url }`.

### `Custom` — Handler personnalisé

Implémente le trait `PasswordHandler` pour brancher ta propre logique de hachage/vérification.

```rust
use runique::prelude::{PasswordHandler, PasswordConfig};

struct MyHasher;

impl PasswordHandler for MyHasher {
    fn name(&self) -> &str { "my_hasher" }
    fn transform(&self, input: &str) -> Result<String, String> {
        // logique de hachage
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

## Utilisation dans le code

```rust
use runique::prelude::{hash, verify};

// Hacher un mot de passe (utilise la config globale)
let hashed = hash("mon_mdp")?;

// Vérifier un mot de passe contre un hash stocké
let ok = verify("mon_mdp", &user.password_hash);
```

Ces fonctions utilisent automatiquement la `PasswordConfig` initialisée au démarrage.

---

## Dans les formulaires

Les champs `TextField::password()` sont hachés automatiquement dans `finalize()` en mode `Auto`. En mode `Manual` ou `Delegated`, aucun hachage automatique n'a lieu.

Voir → [Types de champs — TextField](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/champs/champs.md)

---

← [**Builder**](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/builder/builder.md)
