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

`BuiltinUserEntity::activate_account` active un compte en attente (`is_active` + `activated_at`) et renvoie le compte à jour, prêt pour `login`. Un compte déjà activé, ou désactivé depuis par le staff, n'est pas touché (`None`) : la réactivation reste une décision du staff.

```rust
if let Some(user) = BuiltinUserEntity::activate_account(&db, id).await? {
    login(&session, &user, None, false).await?;
}
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
