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

Pour les cas où vous avez déjà l'utilisateur (un `RuniqueUser`, par exemple le modèle `eihwaz_users` que vous venez d'authentifier) et souhaitez contrôler la persistance DB et la connexion exclusive. Elle renouvelle l'identifiant de session et le jeton CSRF.

```rust
login(
    &session,
    &user,   // &impl RuniqueUser
    None,    // Option<&RuniqueSessionStore> — persistance multi-appareils
    false,   // exclusive — invalider les autres sessions
).await?;
```

### Connexion exclusive

Pour n'autoriser qu'un seul appareil connecté à la fois, passer `exclusive: true` :

```rust
login(&session, &user, Some(&store), true).await?;
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

`logout()` vide **toute** la session (messages flash compris) et lui donne un nouvel identifiant : un message flash destiné à la page suivante s'ajoute **après** l'appel.

---

## Révocation des sessions

`logout()` ne ferme que la session courante. Pour invalider **toutes** les sessions d'un utilisateur (« se déconnecter partout », compromission de compte, changement de mot de passe), il faut passer par les stores.

Le reset de mot de passe intégré (`with_password_reset`) le fait déjà une fois le nouveau mot de passe enregistré. Dans votre propre handler (changement de mot de passe depuis le profil, compte compromis…), une seule méthode suffit :

```rust
engine.close_user_sessions(user_id).await;
```

Elle supprime d'abord les sessions du compte en base (`eihwaz_sessions`), puis en mémoire. L'ordre compte : une session retirée de la mémoire en premier pourrait être relue depuis la base par une requête arrivant entre les deux. Une erreur de base est tracée, pas renvoyée.

> **Limite :** un store branché avec `with_session_store()` (Redis…) ne peut pas être parcouru par utilisateur ; ses sessions ne sont pas fermées.

Pour ne révoquer que les **autres** appareils en gardant la session courante, utilisez `invalidate_other_sessions(user_id, &cookie_id)` côté DB — c'est exactement ce que fait la [connexion exclusive](#connexion).

---

## Vérifications

```rust
// L'utilisateur est-il connecté ?
if is_authenticated(&session).await {
    // ...
}

// Récupérer l'ID en session (retourne Pk = i32/i64/Uuid selon la feature active)
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
