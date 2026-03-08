# Modèle utilisateur

## Modèle built-in

Runique inclut un modèle utilisateur prêt à l'emploi, sans aucune configuration.

**Table générée :** `eihwaz_users`

| Champ | Type | Description |
|---------------|----------|----------------------------------------|
| `id` | `i32` | Clé primaire |
| `username` | `String` | Nom d'utilisateur unique |
| `email` | `String` | Adresse email |
| `password` | `String` | Hash Argon2 — jamais en clair |
| `is_active` | `bool` | Compte actif |
| `is_staff` | `bool` | Accès au panneau admin (limité) |
| `is_superuser` | `bool` | Accès complet, bypass toutes les règles |
| `roles` | `JSON` | Rôles personnalisés ex. `["editor"]` |
| `created_at` | datetime | Date de création |
| `updated_at` | datetime | Date de mise à jour |

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

| Méthode | Retour | Description |
|------------------|-------------|--------------------------------|
| `user_id()` | `i32` | Identifiant unique |
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
| [Helpers de session](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/session/session.md) | `login`, `logout`, vérifications |
| [Middlewares & CurrentUser](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/middleware/middleware.md) | Protection des routes |

## Retour au sommaire

- [Authentification](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md)
