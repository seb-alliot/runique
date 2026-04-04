# Helpers de session

## Import

```rust
use runique::middleware::auth::{
    login, auth_login, logout,
    is_authenticated, get_user_id, get_username,
};
```

---

## Connexion

### `auth_login` — connexion par user_id (recommandé)

Raccourci générique : charge automatiquement les données depuis la DB à partir du seul `user_id`. Adapté à tous les flux d'authentification (inscription, OAuth, magic link…).

```rust
auth_login(&session, &db, user.id).await?;
```

### `login` — connexion complète

Pour les cas où vous avez déjà toutes les données et souhaitez contrôler la persistance DB et la connexion exclusive.

```rust
login(
    &session,
    &db,
    user.id,
    &user.username,
    user.is_staff,
    user.is_superuser,
    None,    // Option<&RuniqueSessionStore> — persistance multi-appareils
    false,   // exclusive — invalider les autres sessions
).await?;
```

### Connexion exclusive

Pour n'autoriser qu'un seul appareil connecté à la fois, passer `exclusive: true` :

```rust
login(&session, &db, user.id, &user.username, false, false, Some(&store), true).await?;
```

Ou via le builder pour activer globalement :

```rust
RuniqueApp::builder(config)
    .middleware(|m| m.with_exclusive_login(true))
```

---

## Déconnexion

```rust
logout(&session, None).await?;

// Avec suppression de la session DB (multi-appareils)
logout(&session, Some(&store)).await?;
```

---

## Vérifications

```rust
// L'utilisateur est-il connecté ?
if is_authenticated(&session).await {
    // ...
}

// Récupérer l'ID en session (retourne UserId = i32 ou i64)
if let Some(user_id) = get_user_id(&session).await {
    // ...
}

// Récupérer le username en session
if let Some(username) = get_username(&session).await {
    // ...
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](/docs/fr/auth/modele) | Built-in, trait `RuniqueUser` |
| [Middlewares & CurrentUser](/docs/fr/auth/middleware) | Protection des routes |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
