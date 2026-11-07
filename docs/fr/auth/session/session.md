# Helpers de session

## Import

```rust
use runique::prelude::*;
```

---

## Connexion

### `login` — ouvrir la session

Deux étapes : `authenticate_user` vérifie les identifiants (mot de passe, compte actif et activé), puis `login` inscrit le compte dans la session. `login` renouvelle l'identifiant de session et le jeton CSRF, et n'y écrit que l'id : le compte est relu en base à chaque requête (`request.user`).

```rust
match authenticate_user(&db, &username, &password).await {
    Some(user) => login(&session, &user, None, false).await?,
    None => { /* identifiants invalides : message générique */ }
}
```

Paramètres : le compte (`&impl RuniqueUser`), la persistance multi-appareils (`Option<&RuniqueSessionStore>`, le store par défaut sauvegarde déjà les sessions connectées en base) et la connexion exclusive.

`login` vérifie elle-même que le compte peut se connecter (`can_sign_in()` : actif et activé), quel que soit le chemin qui l'a chargé. Sinon, rien n'est écrit et elle renvoie `LoginError::CannotSignIn` :

```rust
match login(&session, &user, None, false).await {
    Ok(()) => { /* connecté */ }
    Err(LoginError::CannotSignIn) => { /* compte inactif ou pas encore activé */ }
    Err(LoginError::Session(e)) => { /* le store de session a échoué */ }
}
```

> **Un compte chargé autrement** (après une inscription, un OAuth, un lien magique) se connecte avec le même `login`. Il doit venir du serveur — compte tout juste créé, identité vérifiée — jamais d'un id reçu dans la requête.

### Activer puis connecter

`BuiltinUserEntity::activate_account` active un compte en attente (`is_active` + `activated_at`) et renvoie le compte à jour, prêt pour `login`. Un compte déjà activé, ou désactivé depuis par le staff, n'est pas touché (`None`) : la réactivation reste une décision du staff. Un id inconnu renvoie une erreur (`DbErr::RecordNotFound`).

```rust
if let Some(user) = BuiltinUserEntity::activate_account(&db, id).await? {
    login(&session, &user, None, false).await?;
}
```

### Connexion exclusive

Pour n'autoriser qu'un seul appareil connecté à la fois, le plus simple est le builder :

```rust
RuniqueApp::builder(config)
    .middleware(|m| m.with_exclusive_login(true))
```

Le store de sessions par défaut ferme alors les autres sessions du compte dès qu'il se connecte, quel que soit l'appel à `login` (même `login(&session, &user, None, false)`). Avec un store branché par `with_session_store()`, ce réglage n'a pas d'effet.

À la main, `exclusive: true` n'agit qu'avec un store en base passé à `login` ; avec `None`, il est ignoré sans erreur :

```rust
login(&session, &user, Some(&store), true).await?;
```

---

## Déconnexion

```rust
logout(&session, None).await?;

// Avec suppression de la session DB (multi-appareils)
logout(&session, Some(&store)).await?;
```

Pour une session connectée, `logout()` vide **toute** la session (messages flash compris) et lui donne un nouvel identifiant : un message flash destiné à la page suivante s'ajoute **après** l'appel. Sur une session anonyme, il ne fait rien : la session, son jeton CSRF et ses messages restent.

Si la suppression échoue (base indisponible), `logout()` renvoie l'erreur et la session reste **entière**, cookie compris : l'utilisateur peut réessayer. Ne montrez pas « déconnecté » dans ce cas. De même, `login` sous un autre compte ferme d'abord la session précédente ; s'il n'y parvient pas, la nouvelle connexion est refusée (`LoginError::Session`).

---

## Révocation des sessions

`logout()` ne ferme que la session courante. Pour invalider **toutes** les sessions d'un utilisateur (« se déconnecter partout », compromission de compte, changement de mot de passe), il faut passer par les stores.

Le reset de mot de passe intégré (`with_password_reset`) le fait déjà une fois le nouveau mot de passe enregistré. Dans votre propre handler (changement de mot de passe depuis le profil, compte compromis…), une seule méthode suffit :

```rust
request.engine.close_user_sessions(user_id).await?;
```

Elle supprime les sessions du compte en base (`eihwaz_sessions`), puis leurs copies en mémoire. La base fait autorité : chaque requête d'une session connectée revérifie sa ligne, donc une session supprimée en base est refusée partout, y compris sur une autre instance. Si la base échoue, rien d'autre n'est touché et l'erreur est renvoyée : les sessions sont encore actives, ne présentez pas l'opération comme réussie.

> **Store personnalisé :** un store branché avec `with_session_store()` (Redis…) ne peut pas être parcouru par utilisateur. Ses sessions se ferment quand même : `close_user_sessions` et un reset réussi augmentent `session_version` sur le compte, et une session ouverte avec une version plus ancienne prend fin à sa requête suivante.

Pour ne révoquer que les **autres** appareils en gardant la session courante, utilisez `invalidate_other_sessions(user_id, &cookie_id)` côté DB — c'est exactement ce que fait la [connexion exclusive](#connexion).

---

## Vérifications

```rust
// Un compte est-il inscrit dans la session ? (lit la session seule : ne vérifie
// pas que le compte existe encore ni qu'il est actif — préférez `request.user`)
if is_authenticated(&session).await {
    // ...
}

// Le compte connecté, relu en base à chaque requête (comme `request.user` chez Django)
if let Some(user) = &request.user {
    let user_id = user.id; // Pk = i32/i64/Uuid selon la feature active
    let username = &user.username;
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
