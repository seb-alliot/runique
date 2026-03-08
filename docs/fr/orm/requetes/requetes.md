# Requêtes CRUD

## SELECT — Récupérer

```rust
// Tous les enregistrements
let users: Vec<users::Model> = users::Entity::objects
    .all()
    .all(&*db)
    .await?;

// Avec limite et offset
let users = users::Entity::objects
    .all()
    .limit(10)
    .offset(0)
    .all(&*db)
    .await?;

// Avec tri
use sea_orm::Order;
let users = users::Entity::objects
    .all()
    .order_by_asc(users::Column::Name)
    .all(&*db)
    .await?;
```

---

## COUNT — Compter

```rust
let count = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .count(&*db)
    .await?;
```

---

## WHERE — Filtrage

```rust
use sea_orm::ColumnTrait;

// Égalité
let user = users::Entity::objects
    .filter(users::Column::Email.eq("test@example.com"))
    .first(&*db)
    .await?;

// Comparaisons
let users = users::Entity::objects
    .filter(users::Column::Age.gt(18))
    .all(&*db)
    .await?;

// Multiples conditions
let users = users::Entity::objects
    .filter(users::Column::Active.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&*db)
    .await?;

// OU
let users = users::Entity::objects
    .filter(
        users::Column::Email.eq("a@test.com")
            .or(users::Column::Email.eq("b@test.com"))
    )
    .all(&*db)
    .await?;
```

---

## INSERT — Créer

```rust
use sea_orm::Set;

let new_user = users::ActiveModel {
    email: Set("john@example.com".to_string()),
    username: Set("john".to_string()),
    password: Set(hash_password("password123")),
    ..Default::default()
};

let user = new_user.insert(&*db).await?;
```

---

## UPDATE — Modifier

```rust
use sea_orm::{Set, Unchanged};

let mut user = users::Entity::find_by_id(1)
    .one(&*db)
    .await?
    .ok_or("User not found")?;

let mut user = user.into_active_model();
user.email = Set("newemail@example.com".to_string());

let updated = user.update(&*db).await?;
```

---

## DELETE — Supprimer

```rust
// Supprimer un seul
let result = users::Entity::delete_by_id(1)
    .exec(&*db)
    .await?;

// Supprimer multiples
let result = users::Entity::delete_many()
    .filter(users::Column::Active.eq(false))
    .exec(&*db)
    .await?;
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Manager & helpers](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/manager/manager.md) | `impl_objects!`, helpers |
| [Avancé](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/avance/avance.md) | Transactions, relations |

## Retour au sommaire

- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md)
