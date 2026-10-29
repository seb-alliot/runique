# Évolutions prévues

Pistes pour les prochaines versions de l'admin Runique (builder `ModelAdmin`).

## Inlines admin

Éditer des modèles liés dans le formulaire parent, à la manière des inlines Django :

```rust
ModelAdmin::new()
    .form(|f| f.fields([Title, Status]))
    // futur :
    // .inline(lignes, |i| i.fk(CommandeId).fields([Produit, Qte]))
```

L'inline afficherait un tableau éditable (ajout, suppression, modification) dans la page create/edit du parent.

## `view!{}` — CRUD public

Macro de routage public analogue aux vues génériques Django, avec templates surchargeables :

```rust
view! {
    Article {
        model: entities::article::Entity,
        prefix: "/articles",
        templates: {
            list:   "articles/list.html",
            detail: "articles/detail.html",
            create: "articles/create.html",
        },
        page_size: 10,
        login_required: true,
    }
}
```

Routes visées : liste paginée, détail, create, edit, delete — templates framework overridables block par block, comme l'admin.

## Filtres métier custom

Aujourd'hui le scope liste suit le gate (droits + ownership). Un crochet déclaratif pour un filtre métier (« commandes de ma région ») reste ouvert.

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin |
| [Builder ModelAdmin](/docs/fr/admin/declaration-macro) | API actuelle |
| [Permissions](/docs/fr/admin/permission) | Gate runtime |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
