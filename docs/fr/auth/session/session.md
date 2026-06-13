# Helpers de session

## Import

```rust
use runique::prelude::*;
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

> **Note :** Si vous utilisez votre propre modèle utilisateur (Custom Model) en remplacement de la table par défaut, vous **devez** utiliser `login()`. En effet, `auth_login()` tente systématiquement d'interroger la table interne `eihwaz_users`.

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

## Révocation des sessions

`logout()` ne ferme que la session courante. Pour invalider **toutes** les sessions d'un utilisateur (« se déconnecter partout », compromission de compte, changement de mot de passe), appelez explicitement les méthodes des stores.

> **Important :** le reset de mot de passe intégré (`with_password_reset`) **n'invalide pas** les sessions actives — une session déjà ouverte (y compris volée) reste valide jusqu'à expiration. Si vous voulez révoquer les sessions au changement de mot de passe, faites-le dans votre propre handler après la mise à jour. C'est un choix délibéré : le framework fournit les primitives, vous décidez de la politique.

```rust
// Sessions persistées en DB (with_db_fallback)
if let Some(store) = engine
    .session_db_store
    .read()
    .ok()
    .and_then(|g| g.as_ref().cloned())
{
    store.invalidate_all(user_id).await?;
}

// Sessions en mémoire (store par défaut)
if let Some(mem) = engine
    .session_store
    .read()
    .ok()
    .and_then(|g| g.as_ref().cloned())
{
    mem.invalidate_user_sessions(user_id).await;
}
```

Pour ne révoquer que les **autres** appareils en gardant la session courante, utilisez `invalidate_other_sessions(user_id, &cookie_id)` côté DB — c'est exactement ce que fait la [connexion exclusive](#connexion).

---

## Vérifications

```rust
// L'utilisateur est-il connecté ?
if is_authenticated(&session).await {
    // ...
}

// Récupérer l'ID en session (retourne Pk = i32 ou i64)
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
