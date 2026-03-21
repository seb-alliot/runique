# Helpers de session

## Import

```rust
use runique::middleware::auth::{
    login, login_staff, logout,
    is_authenticated, get_user_id, get_username,
};
```

---

## Connexion

```rust
// Utilisateur classique — is_staff et is_superuser valent false par défaut
login(&session, user.id, &user.username).await?;

// Utilisateur staff/admin — avec droits et rôles personnalisés
login_staff(
    &session,
    user.id,
    &user.username,
    user.is_staff,
    user.is_superuser,
    user.roles(),
).await?;
```

### Connexion exclusive

Pour n'autoriser qu'un seul appareil connecté à la fois, activer via le builder :

```rust
RuniqueApp::builder(config)
    .middleware(|m| m.with_exclusive_login(true))
```

`login` et `login_staff` invalident alors automatiquement toutes les sessions existantes
de l'utilisateur à chaque nouvelle connexion. Aucun changement dans les handlers.

> Désactivé par défaut (`false`). Sans effet si un store externe est utilisé.

---

## Déconnexion

```rust
logout(&session).await?;
```

---

## Vérifications

```rust
// L'utilisateur est-il connecté ?
if is_authenticated(&session).await {
    // ...
}

// Récupérer l'ID en session
if let Some(user_id) = get_user_id(&session).await {
    // ...
}

// Récupérer le username en session
if let Some(username) = get_username(&session).await {
    // ...
}
```

---

## Variables d'environnement

Ces variables contrôlent les redirections automatiques des middlewares :

| Variable | Défaut | Description |
| --- | --- | --- |
| `REDIRECT_ANONYMOUS` | `/` | Cible de redirection pour les utilisateurs non connectés |
| `USER_CONNECTED_URL` | `/` | Cible de redirection pour les utilisateurs déjà connectés |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/modele/modele.md) | Built-in, trait `RuniqueUser` |
| [Middlewares & CurrentUser](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/middleware/middleware.md) | Protection des routes |

## Retour au sommaire

- [Authentification](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md)
