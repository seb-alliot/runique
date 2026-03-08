# Rôles et permissions

## Source de vérité : la table `users`

Le système de permissions s'appuie sur l'utilisateur authentifié, dont les données sont lues depuis la table `users`.

## Champs de contrôle d'accès

| Champ | Type | État | Rôle |
| --- | --- | --- | --- |
| `is_staff` | `bool` | ✅ Actif | Autorise l'accès à l'interface admin |
| `is_superuser` | `bool` | ✅ Actif | Accès total, bypass toutes les vérifications |
| `is_active` | `bool` | ⏳ En développement | Prévu pour bloquer les comptes désactivés |
| `roles` | `Option<Vec<String>>` | ⏳ En développement | Permissions par ressource (déclarées, non appliquées) |

---

## Ce qui est réellement appliqué aujourd'hui

### Entrée dans l'admin

Le middleware vérifie uniquement `is_staff` et `is_superuser` :

```
is_staff = true  OU  is_superuser = true  →  accès accordé
les deux = false                           →  redirection /admin/login
```

### is_superuser

Un utilisateur avec `is_superuser = true` bypass **toutes** les vérifications — entrée admin et permissions par ressource.

---

## Ce qui est déclaré mais pas encore appliqué

### is_active

Le champ existe dans le modèle `users` mais n'est pas encore vérifié par le middleware admin. Un compte avec `is_active = false` peut toujours se connecter si `is_staff` ou `is_superuser` est `true`.

### roles / permissions

La macro `admin!` permet de déclarer des rôles autorisés par ressource :

```rust
admin! {
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["editor", "admin"]
    }
}
```

La structure `ResourcePermissions` et la fonction `check_permission()` existent dans le code, mais **ne sont pas appelées** dans les handlers CRUD générés. Les permissions sont stockées sans être vérifiées à ce stade.

---

## Logique d'accès actuelle (état réel)

```
authentifié ?
  └─ non → redirection /admin/login
  └─ oui → is_staff OU is_superuser ?
               └─ aucun → redirection /admin/login
               └─ is_superuser → AUTORISÉ (accès total)
               └─ is_staff seulement → AUTORISÉ (sans vérification de rôle)
```

---

## Notes

- `is_active` et `roles` sont prévus dans la roadmap — voir [evolution/futur_feature.md](../evolution/futur_feature.md).
- La macro `admin!` définit uniquement les règles déclaratives ; la logique d'application est dans les middlewares.
- La granularité par opération CRUD (list/create/edit/delete) n'est pas supportée dans la version actuelle.

---

## Revenir au sommaire

- [Sommaire Admin](../11-Admin.md)
