# Rôles et permissions

## Source de vérité : la table `users`

Le système de permissions s'appuie sur l'utilisateur authentifié, dont les données sont lues depuis la table `users`.

## Champs de contrôle d'accès

| Champ | Type | État | Rôle |
| --- | --- | --- | --- |
| `is_staff` | `bool` | ✅ Actif | Autorise l'accès à l'interface admin |
| `is_superuser` | `bool` | ✅ Actif | Accès total, bypass toutes les vérifications |
| `is_active` | `bool` | ⏳ En développement | Prévu pour bloquer les comptes désactivés |
| `roles` | `Option<Vec<String>>` | ✅ Actif | Rôles utilisateur — accessibles dans les templates via `current_user.roles` |

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

### roles

Le champ `roles` est disponible dans tous les templates admin via `current_user.roles`.

#### Saisie dans l'interface admin

Les rôles se saisissent en texte libre, séparés par des virgules :

```
editor
editor, moderator
admin, editor
```

#### Utilisation dans les templates

```html
{% if current_user and "editor" in current_user.roles %}
    <a href="...">Modifier</a>
{% endif %}
```

`is_superuser = true` bypass toujours les conditions de rôle — un superuser voit tout.

#### Permissions par ressource (déclaratives)

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

- `is_active` et `roles` sont prévus dans la roadmap — voir [Évolutions](/docs/fr/admin/evolution).
- La macro `admin!` définit uniquement les règles déclaratives ; la logique d'application est dans les middlewares.
- La granularité par opération CRUD (list/create/edit/delete) n'est pas supportée dans la version actuelle.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin dans un projet existant, créer un superuser |
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution et état bêta |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
