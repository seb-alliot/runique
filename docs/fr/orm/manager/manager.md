# Manager & helpers SeaORM

## SeaORM + Objects Manager

Runique utilise **SeaORM** avec un manager Django-like via la macro `impl_objects!`.

La macro est **générée automatiquement** par le daemon dans chaque fichier d'entité (`runique makemigrations`). Aucune déclaration manuelle nécessaire.

> **Note** : `impl_objects!` est du sucre syntaxique pur. Elle ne modifie pas les requêtes générées — le SQL produit est identique à du SeaORM natif.

```rust
use demo_app::entities::users;

// Récupérer tous les users
let all_users = users::Entity::objects
    .all()
    .all(&*db)
    .await?;

// Avec filtre
let active_users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .all(&*db)
    .await?;

// Un seul résultat
let user = users::Entity::objects
    .filter(users::Column::Id.eq(user_id))
    .first(&*db)
    .await?;
```

---

## Sans macro (SeaORM natif)

```rust
// Tous les enregistrements
let all_users = users::Entity::find().all(&*db).await?;

// Avec filtre
let active_users = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .all(&*db)
    .await?;

// Un seul résultat par ID
let user = users::Entity::find_by_id(user_id)
    .one(&*db)
    .await?;
```

---

## Helpers disponibles

- `all()` : point d'entrée pour chaîner `filter`, `order_by_asc/desc`, `limit`, `offset`.
- `filter(...)` / `exclude(...)` : ajoutent des conditions.
- `first(db)` / `count(db)` : exécutent la requête.
- `get(db, id)` / `get_optional(db, id)` : accès direct par clé primaire.
- `get_or_404(db, ctx, msg)` : retourne une réponse 404/500 avec rendu Tera si manquant.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Requêtes CRUD](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/requetes/requetes.md) | SELECT, COUNT, WHERE, INSERT, UPDATE, DELETE |
| [Avancé](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/avance/avance.md) | Transactions, relations |

## Retour au sommaire

- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md)
