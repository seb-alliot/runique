# Configuration des mots de passe

[← Builder](/docs/fr/configuration/builder)

---

`PasswordConfig` configure la stratégie de hachage et de vérification des mots de passe pour toute l'application. Elle s'initialise **une seule fois** au démarrage via `password_init()`.

## Initialisation dans `main.rs`

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Argon2 automatique (défaut recommandé)
    password_init(PasswordConfig::auto());

    let config = RuniqueConfig::from_env();

    RuniqueApp::builder(config)
        // ...
        .build()
        .await?
        .run()
        .await?;
    Ok(())
}
```

> **Ordre d'appel.** `password_init` ne compte qu'**une fois** : le premier appel gagne, les suivants sont ignorés (journalisés seulement avec le log `auth.password_init`). Appelez-le **en tout premier** dans `main`, avant `build()` et avant tout `hash()` / `verify()` :
>
> - un `hash()` ou un `verify()` appelé avant fige la configuration par défaut (`auto`) ;
> - `build()` appelle lui-même `password_init(config.password)` : un appel fait **après** `build()` est ignoré.
>
> Gardez aussi `config.password` égal à ce que vous passez à `password_init` : la page de réinitialisation intégrée hache avec `config.password`.

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

> **Reset intégré et admin :** la route de reset intégrée (`with_password_reset`) hache elle-même le nouveau mot de passe avec la configuration de l'application, quel que soit le mode (`Auto`, `Manual` ou `Custom`). En mode `Delegated`, elle refuse le reset : aucun mot de passe local n'est écrit, et le lien reste utilisable. Un compte créé depuis l'admin reçoit un mot de passe aléatoire, haché de la même façon, puis un lien pour choisir le sien.
>
> Dans **vos propres** formulaires, en revanche, `finalize()` ne hache qu'en mode `Auto` : en `Manual` ou `Custom`, appelez `hash()` avant d'écrire en base.

### `Delegated` — Authentification externe (OAuth / SSO)

Aucun mot de passe local n'est géré par Runique : `verify()` répond toujours `false`, `hash()` échoue, et le reset intégré est refusé.

> **Ce mode ne fournit pas encore l'authentification externe elle-même.** Les fournisseurs ci-dessous ne sont que des noms : Runique n'implémente aujourd'hui ni le flux OAuth (redirection, échange du code, lecture du profil), ni LDAP, ni SAML. `Delegated` sert à désactiver proprement les mots de passe locaux quand **votre application** gère la connexion par un autre moyen (une bibliothèque OAuth, un proxy d'authentification), puis ouvre la session avec `login(&request.session, &user, None, false)` une fois l'utilisateur identifié. L'intégration complète d'un fournisseur est une évolution prévue.

```rust
use runique::prelude::*;
use runique::utils::password::External;

password_init(PasswordConfig::Delegated(External::GoogleOAuth));
password_init(PasswordConfig::Delegated(External::Microsoft));
password_init(PasswordConfig::Delegated(External::Ldap("ldap://...".to_string())));
```

Fournisseurs disponibles : `GoogleOAuth`, `Microsoft`, `Apple`, `Ldap(url)`, `Saml(url)`, `Custom { name, authorize_url, token_url }`.

### `Custom` — Handler personnalisé

Implémente le trait `PasswordHandler` pour brancher ta propre logique de hachage/vérification.

```rust
use runique::prelude::{PasswordConfig, password_init};
use runique::utils::password::PasswordHandler;

#[derive(Clone)] // requis : PasswordHandler est cloné avec la config
struct MyHasher;

impl PasswordHandler for MyHasher {
    fn name(&self) -> &str { "my_hasher" }
    fn create_field(&self, name: &str) -> Box<dyn runique::forms::FormField> {
        Box::new(runique::forms::fields::text::TextField::password(name))
    }
    fn validate_input(&self, input: &str) -> Result<(), String> {
        if input.len() < 8 { Err("Trop court".into()) } else { Ok(()) }
    }
    fn transform(&self, input: &str) -> Result<String, String> {
        mon_algo::hash(input).map_err(|e| e.to_string())
    }
    fn verify(&self, input: &str, stored: &str) -> bool {
        // La comparaison doit se faire en temps constant (la fonction de
        // vérification de l'algorithme, ou `ct_eq`), jamais avec `==`.
        mon_algo::verify(input, stored)
    }
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
let ok = verify("mon_mdp", &user.password);
```

Ces fonctions utilisent automatiquement la `PasswordConfig` initialisée au démarrage.

---

## Dans les formulaires

Les champs `TextField::password()` sont hachés automatiquement dans `finalize()` en mode `Auto`. En mode `Manual` ou `Custom`, aucun hachage automatique n'a lieu : appelez `hash()` vous-même. Un champ marqué `.no_hash()` n'est jamais haché (un formulaire de connexion compare le mot de passe tel quel, avec `authenticate_user`).

En mode `Auto`, un mot de passe haché avec un autre algorithme que celui configuré est converti à la première connexion réussie.

Voir → [Types de champs — TextField](/docs/fr/formulaire/champs)

---

← [**Builder**](/docs/fr/configuration/builder)
