# Admin Runique

L'administration Runique génère un CRUD complet à partir d'une macro déclarative (`admin!`).
Le code produit est du Rust ordinaire — lisible, auditable et modifiable si nécessaire.
L'approche est volontairement transparente : pas de magie cachée, pas de runtime inconnu.

## Exemple minimal

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin", "staff"],
        list_display: [
            ["username", "Nom d'utilisateur"],
            ["email", "Email"],
            ["is_active", "Actif"],
        ],
        list_filter: [
            ["is_active", "Actif"],
        ]
    }
}
```

## Table des matières

| Fichier | Contenu |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin dans un projet existant, créer un superuser |
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Daemon & génération](/docs/fr/admin/declaration) | Fichiers générés, comportement du watcher |
| [Macro `admin!`](/docs/fr/admin/declaration) | Syntaxe complète, champs obligatoires et optionnels |
| [Création d'utilisateur](/docs/fr/admin/user-creation) | Flux complet création → email → activation, modèle custom |
| [Vue liste](/docs/fr/admin/liste) | Pagination, tri, recherche, `list_display`, `list_filter` |
| [Permissions](/docs/fr/admin/permission) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution et état bêta |

## Revenir au menu

- [Readme Francais](https://github.com/seb-alliot/runique/blob/main/README.fr.md)
