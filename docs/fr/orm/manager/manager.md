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
    .all(&db)
    .await?;

// Avec filtre
let active_users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .all(&db)
    .await?;

// Un seul résultat
let user = users::Entity::objects
    .filter(users::Column::Id.eq(user_id))
    .first(&db)
    .await?;
```

---

## Sans macro (SeaORM natif)

```rust
// Tous les enregistrements
let all_users = users::Entity::find().all(&db).await?;

// Avec filtre
let active_users = users::Entity::find()
    .filter(users::Column::Active.eq(true))
    .all(&db)
    .await?;

// Un seul résultat par ID
let user = users::Entity::find_by_id(user_id)
    .one(&*db)
    .await?;
```

---

## Helpers disponibles

| Méthode | Description |
| --- | --- |
| `all()` | Point d'entrée — retourne un `RuniqueQueryBuilder` chainable |
| `filter(cond)` | Ajoute une condition WHERE (AND) |
| `exclude(cond)` | Ajoute une condition WHERE NOT |
| `asc(col)` / `desc(col)` | Tri ascendant / descendant par colonne |
| `order_by_random()` | Tri par `RANDOM()` — sans SQL brut |
| `order_by_expr(expr, order)` | Tri par une expression SeaORM arbitraire |
| `limit(n)` / `offset(n)` | Pagination |
| `first(db)` | Exécute et retourne le premier résultat (`Option<Model>`) |
| `one(db)` | Retourne le résultat si exactement 1 ligne, `Err` si plusieurs |
| `count(db)` | Compte les lignes correspondantes |
| `get(db, id)` / `get_optional(db, id)` | Accès direct par clé primaire |
| `get_or_404(db, ctx, msg)` | Retourne 404/500 avec rendu Tera si manquant |

### `order_by_random()`

```rust
let suggestion = MonForm::objects
    .all()
    .order_by_random()
    .limit(1)
    .first(&db)
    .await?;
```

### `order_by_expr(expr, order)`

```rust
use sea_orm::Order;

// Tri par longueur de titre
let results = MonForm::objects
    .all()
    .order_by_expr(sea_query::Expr::cust("LENGTH(titre)"), Order::Desc)
    .all(&db)
    .await?;
```

### `one()` — exactement un résultat

Analogue au `.get()` de Django : retourne `Err` si plusieurs lignes correspondent — sans scanner la table entière (fetche au plus 2 lignes).

```rust
// Retourne Ok(None) si 0 résultats, Ok(Some(model)) si 1, Err si plusieurs
let unique = MonForm::objects
    .all()
    .filter(mon_entity::Column::Email.eq("user@example.com"))
    .one(&db)
    .await?;

match unique {
    Ok(Some(model)) => { /* trouvé, unique */ }
    Ok(None) => { /* inexistant */ }
    Err(e) => { /* plusieurs lignes — données incohérentes */ }
}
```

---

## Macro `search!` — DSL de filtrage

Pour les cas courants, la macro `search!` offre une syntaxe plus concise que le chaînage SeaORM :

```rust
use runique::search;

// Équivalent à objects.filter(Col::Active.eq(true))
let actifs = search!(users::Entity => Active eq true)
    .all(&db).await?;

// Multi-conditions (AND)
let results = search!(users::Entity =>
    Active eq true,
    Age gte 18,
    Status in ["active", "verified"],
)
.limit(20)
.all(&db)
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
    .first(&db)
    .await?;

let all = UserForm::objects
    .all()
    .all(&db)
    .await?;
```

La syntaxe `@Form` dans `search!` délègue automatiquement à l'entité liée :

```rust
// Équivalent à search!(users::Entity => Active eq true)
let actifs = search!(@UserForm => Active eq true)
    .all(&db).await?;

// Toutes les syntaxes search! sont supportées
let results = search!(@UserForm =>
    Active eq true,
    Age gte 18,
    Status in ["active", "verified"],
)
.limit(20)
.all(&db)
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
