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
| [Mise en place](setup/setup.md) | Câbler l'admin dans un projet existant, créer un superuser |
| [CLI](declaration/cli.md) | Commande `runique start`, workflow général |
| [Permissions](permission/permissions.md) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](template/templates.md) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](evolution/futur_feature.md) | Axes d'évolution et état bêta |
