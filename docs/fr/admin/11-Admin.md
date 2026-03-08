# Admin Runique

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
| [Mise en place](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/setup/setup.md) | Câbler l'admin dans un projet existant, créer un superuser |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/declaration/cli.md) | Commande `runique start`, workflow général |
| [Daemon & génération](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/declaration/daemon/generation.md) | Fichiers générés, comportement du watcher |
| [Macro `admin!`](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/declaration/macro/macro.md) | Syntaxe complète, champs obligatoires et optionnels |
| [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/permission/permissions.md) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/template/templates.md) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/evolution/evolution.md) | Axes d'évolution et état bêta |

## Revenir au menu

- [Readme Francais](https://github.com/seb-alliot/runique/blob/main/README.fr.md)
