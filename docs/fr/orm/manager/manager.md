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

## Macro `search!` — DSL de filtrage

Pour les cas courants, la macro `search!` offre une syntaxe plus concise que le chaînage SeaORM :

```rust
use runique::search;

// Equivalent à objects.filter(Col::Active.eq(true))
let actifs = search!(users::Entity => Active = true)
    .all(&*db).await?;

// Multi-conditions (AND)
let results = search!(users::Entity =>
    Active = true,
    Age >= 18,
    Status = ("active" | "verified"),
)
.limit(20)
.all(&*db)
.await?;
```

Voir [Requêtes CRUD — Macro search!](/docs/fr/orm/requetes#macro-search--dsl-de-filtrage) pour la référence complète.

---

## Accès via formulaire — `FormEntity`

Lorsqu'un form est annoté avec `#[form(model = Entity)]`, Runique génère automatiquement :

- L'implémentation du trait `FormEntity` (lien form ↔ entité)
- Une constante `pub const objects` sur le form, identique à `Entity::objects`

```rust
#[form(schema = user_schema, model = users::Entity)]
pub struct UserForm;

// Accès objects via le form — équivalent à users::Entity::objects
let user = UserForm::objects
    .filter(users::Column::Id.eq(id))
    .first(&*db)
    .await?;

let all = UserForm::objects
    .all()
    .all(&*db)
    .await?;
```

La syntaxe `@Form` dans `search!` délègue automatiquement à l'entité liée :

```rust
// Équivalent à search!(users::Entity => Active = true)
let actifs = search!(@UserForm => Active = true)
    .all(&*db).await?;

// Toutes les syntaxes search! sont supportées
let results = search!(@UserForm =>
    Active = true,
    Age >= 18,
    Status = ("active" | "verified"),
)
.limit(20)
.all(&*db)
.await?;
```

> Le trait `FormEntity` est défini dans `runique::forms`. Il est implémenté automatiquement — aucune déclaration manuelle requise.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Requêtes CRUD](/docs/fr/orm/requetes) | SELECT, COUNT, WHERE, INSERT, UPDATE, DELETE |
| [Avancé](/docs/fr/orm/avance) | Transactions, relations |
| [Formulaires](/docs/fr/model/formulaires/formulaires) | `#[form(model = ...)]`, `FormEntity`, `Form::objects` |

## Retour au sommaire

- [ORM](/docs/fr/orm)
