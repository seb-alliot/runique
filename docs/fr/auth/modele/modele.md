# Modèle utilisateur

## Modèle built-in

Runique inclut un modèle utilisateur prêt à l'emploi, sans aucune configuration.

**Table générée :** `eihwaz_users`

| Champ | Type | Description |
|---------------|----------|----------------------------------------|
| `id` | `UserId` | Clé primaire (`i32` par défaut, `i64` avec la feature `big-pk`) |
| `username` | `String` | Nom d'utilisateur unique |
| `email` | `String` | Adresse email |
| `password` | `String` | Hash Argon2 — jamais en clair |
| `is_active` | `bool` | Compte actif |
| `is_staff` | `bool` | Accès au panneau admin (limité) |
| `is_superuser` | `bool` | Accès complet, bypass toutes les règles |
| `roles` | `String` | Rôles personnalisés (nullable) |
| `created_at` | datetime | Date de création |
| `updated_at` | datetime | Date de mise à jour |

Pour créer le premier superutilisateur :

```bash
runique create-superuser
```

### Clé primaire i64 (BigAutoField)

Par défaut, la clé primaire est un `i32`. Pour passer à `i64` (comme Django depuis 3.2) :

```toml
# Cargo.toml du projet
runique = { version = "...", features = ["big-pk"] }
```

---

## Trait RuniqueUser

Si vous utilisez votre propre modèle utilisateur à la place du built-in, vous devez implémenter `RuniqueUser`.

```rust
use runique::middleware::auth::RuniqueUser;
use runique::prelude::UserId;

impl RuniqueUser for users::Model {
    fn user_id(&self) -> UserId      { self.id }
    fn username(&self) -> &str       { &self.username }
    fn email(&self) -> &str          { &self.email }
    fn password_hash(&self) -> &str  { &self.password }
    fn is_active(&self) -> bool      { self.is_active }
    fn is_staff(&self) -> bool       { self.is_staff }
    fn is_superuser(&self) -> bool   { self.is_superuser }

    // Optionnel — logique d'accès admin sur mesure
    fn can_access_admin(&self) -> bool {
        self.is_active() && (self.is_staff() || self.is_superuser())
    }
}
```

### Méthodes obligatoires

| Méthode | Retour | Description |
|------------------|-------------|--------------------------------|
| `user_id()` | `UserId` | Identifiant unique |
| `username()` | `&str` | Nom d'utilisateur |
| `email()` | `&str` | Adresse email |
| `password_hash()` | `&str` | Hash du mot de passe |
| `is_active()` | `bool` | Compte actif |
| `is_staff()` | `bool` | Accès admin limité |
| `is_superuser()` | `bool` | Accès admin complet |

### Méthodes avec valeur par défaut

| Méthode | Défaut | Description |
|--------------------|-------------------------------------|------------------------------|
| `roles()` | `vec![]` | Rôles personnalisés |
| `can_access_admin()` | `is_active && (is_staff \|\| is_superuser)` | Logique d'accès admin |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Helpers de session](/docs/fr/auth/session) | `login`, `auth_login`, `logout` |
| [Middlewares & CurrentUser](/docs/fr/auth/middleware) | Protection des routes |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
