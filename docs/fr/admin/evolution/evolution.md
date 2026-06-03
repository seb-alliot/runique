# Évolutions prévues

Fonctionnalités envisagées pour les versions futures de l'admin Runique.

---

## Inlines admin

Permettre d'éditer des modèles liés directement dans le formulaire parent, à la manière des inlines Django :

```rust
admin! {
    commande {
        model: entities::commande::Entity,
        inline: [
            ["lignes", "crate::entities::commande_ligne", "commande_id"],
        ],
    }
}
```

L'inline afficherait un tableau éditable des entrées liées (ajout, suppression, modification) dans la page create/edit de la ressource parente.

---

## `view!{}` — router CRUD public avec templates surchargeables

Macro de routage qui fusionne déclaration de routes et templates par défaut overridables, analogue aux vues génériques Django (`ListView`, `DetailView`, `CreateView`…).

```rust
view! {
    Article {
        model: entities::article::Entity,
        prefix: "/articles",
        templates: {
            list:   "articles/list.html",    // surcharge optionnelle
            detail: "articles/detail.html",
            create: "articles/create.html",
        },
        page_size: 10,
        login_required: true,
    }
}
```

Routes générées : `GET /articles` (liste paginée), `GET /articles/<id>` (detail), `GET/POST /articles/new` (create), `GET/POST /articles/<id>/edit`, `POST /articles/<id>/delete`. Les templates par défaut sont fournis par le framework et surchargeables block par block, exactement comme l'admin.

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin dans un projet existant, créer un superuser |
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Permissions](/docs/fr/admin/permission) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge du visuel |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
