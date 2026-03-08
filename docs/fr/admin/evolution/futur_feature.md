# Évolutions prévues

Fonctionnalités envisagées pour les versions futures de l'admin Runique.

---

## Permissions granulaires par opération CRUD

Actuellement les permissions s'appliquent uniformément à toutes les opérations.
L'objectif est de permettre :

```rust
admin! {
    users: users::Model => UserForm {
        title: "Utilisateurs",
        permissions: {
            list:   ["staff", "admin"],
            create: ["admin"],
            edit:   ["admin"],
            delete: ["admin"],
        }
    }
}
```

---

## Filtres et recherche dans la liste

Ajout de filtres déclaratifs sur la vue `list` :

```rust
admin! {
    users: users::Model => UserForm {
        title: "Utilisateurs",
        filters: ["username", "is_active"],
        search: ["username", "email"],
    }
}
```

---

## Relations et champs calculés

Support des relations SeaORM dans les vues detail/edit (affichage des entités liées).

---

## Amélioration du retour d'erreur du daemon

Meilleur feedback lors de la génération : erreurs de compilation Rust exposées directement dans le terminal avec contexte.


## Revenir au sommaire

- [Sommaire Admin](../11-Admin.md)