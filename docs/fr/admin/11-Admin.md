# Admin Runique

** CLi**
La commande `runique start` lance un daemon qui surveille `src/admin.rs` en continu via un watcher basé sur `notify`.


L'administration Runique génère un CRUD complet à partir d'une macro déclarative (`admin!`).
Le code produit est du Rust ordinaire — lisible, auditable et modifiable si nécessaire.
L'approche est volontairement transparente : pas de magie cachée, pas de runtime inconnu.

## Exemple minimal

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin", "staff"]
    }
}
```

## Table des matières

| Fichier | Contenu |
| --- | --- |
| [declaration — Vue d'ensemble](declaration/cli/cli.md) | Commande `runique start`, workflow général |
| [Macro `admin!`](declaration/macro/macro.md) | Syntaxe complète, champs, erreurs compile-time |
| [Daemon & génération](declaration/daemon/generation.md) | Watcher, structure `src/admins/`, écrasement |
| [Permissions](permission/permissions.md) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](template/04-templates.md) | Hiérarchie de templates, blocks, surcharge du visuel |
| [CSRF](template/csrf/csrf.md) | Token CSRF, `csrf.js`, `{% csrf %}`, checklist login custom |
| [Évolutions](evolution/futur_feature.md) | Axes d'évolution et état bêta |
