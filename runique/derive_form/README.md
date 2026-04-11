# derive_form

Procedural macros for the [Runique](https://github.com/seb-alliot/runique) web framework.

Exposes two macros:

- `model!(...)` — DSL to declare a SeaORM model and generate its schema
- `#[form(...)]` — attribute macro to generate a form struct from a model schema

---

## `model!(...)`

Declares a database model and generates the corresponding SeaORM entity, `ActiveModel`,
relations, and a `schema()` function used by `#[form(...)]`.

### Syntax

```text
model! {
    ModelName,
    table: "table_name",
    pk: field_name => pk_type,
    fields: {
        field: Type [option1, option2, ...],
        ...
    }
}
```

### Minimal example

```rust
use runique::prelude::*;

model! {
    Post,
    table: "posts",
    pk: id => Pk,
    fields: {
        title:   String [required, max_len(255)],
        content: String [required],
        slug:    String [required, unique],
        views:   i32    [required, default(0)],
        published: bool [required, default(false)],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
```

---

## Primary key types

| Syntax           | Column type                                        | Notes             |
|------------------|----------------------------------------------------|-------------------|
| `pk: id => Pk`   | `INTEGER` (default) or `BIGINT` (feature `big-pk`) | Auto-increment    |
| `pk: id => i32`  | `INTEGER` (32-bit)                                 | Auto-increment    |
| `pk: id => i64`  | `BIGINT` (64-bit)                                  | Auto-increment    |
| `pk: id => uuid` | `UUID`                                             | No auto-increment |

```rust
// Integer PK (most common)
model! {
    Article,
    table: "articles",
    pk: id => Pk,
    fields: { title: String [required] }
}

// UUID PK
model! {
    Session,
    table: "sessions",
    pk: token => uuid,
    fields: { user_id: i32 [required] }
}
```

---

## Field types

| Type              | SQL type            | Example usage                              |
|-------------------|---------------------|--------------------------------------------|
| `String`          | `VARCHAR(255)`      | `name: String [required, max_len(100)]`    |
| `text`            | `TEXT`              | `bio: text [nullable]`                     |
| `i8`              | `TINYINT`           | `score: i8 [required]`                     |
| `i16`             | `SMALLINT`          | `rank: i16 [required]`                     |
| `i32`             | `INTEGER`           | `count: i32 [required, default(0)]`        |
| `i64`             | `BIGINT`            | `views: i64 [required, default(0)]`        |
| `f32`             | `FLOAT`             | `rating: f32 [nullable]`                   |
| `f64`             | `DOUBLE`            | `price: f64 [required]`                    |
| `bool`            | `BOOLEAN`           | `is_active: bool [required, default(true)]`|
| `date`            | `DATE`              | `birth_date: date [nullable]`              |
| `time`            | `TIME`              | `start_time: time [nullable]`              |
| `datetime`        | `DATETIME`          | `created_at: datetime [auto_now]`          |
| `timestamp`       | `TIMESTAMP`         | `expires_at: timestamp [nullable]`         |
| `uuid`            | `UUID`              | `token: uuid [required, unique]`           |
| `json`            | `JSON`              | `metadata: json [nullable]`                |
| `blob`            | `BLOB`              | `data: blob [nullable]`                    |

```rust
model! {
    Profile,
    table: "profiles",
    pk: id => Pk,
    fields: {
        username:   String   [required, max_len(50), unique],
        bio:        text     [nullable],
        age:        i16      [nullable, min(0), max(150)],
        score:      f64      [required, default(0.0)],
        is_active:  bool     [required, default(true)],
        birth_date: date     [nullable],
        avatar_url: String   [nullable],
        metadata:   json     [nullable],
        token:      uuid     [required, unique],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
```

---

## Field options

| Option                   | Description                                    |
|--------------------------|------------------------------------------------|
| `required`               | Column is `NOT NULL`                           |
| `nullable`               | Column accepts `NULL`                          |
| `unique`                 | UNIQUE constraint                              |
| `index`                  | Create a database index                        |
| `default(value)`         | Default value — `default(0)`, `default(true)`, `default("draft")` |
| `max_len(n)`             | Max string length (validation + `VARCHAR(n)`)  |
| `min_len(n)`             | Min string length (validation)                 |
| `max(n)` / `max_f(n)`    | Max integer / float value (validation)         |
| `min(n)` / `min_f(n)`    | Min integer / float value (validation)         |
| `auto_now`               | Set to `NOW()` on INSERT (timestamps)          |
| `auto_now_update`        | Set to `NOW()` on UPDATE (timestamps)          |
| `fk(table.col, action)`  | Foreign key — see section below                |
| `label("...")`           | Display label for generated forms              |
| `help("...")`            | Help text for generated forms                  |

```rust
model! {
    Product,
    table: "products",
    pk: id => Pk,
    fields: {
        name:        String [required, max_len(200), label("Product name")],
        description: text   [nullable, help("Detailed product description")],
        price:       f64    [required, min_f(0.0)],
        stock:       i32    [required, default(0), min(0)],
        sku:         String [required, max_len(50), unique],
        is_active:   bool   [required, default(true)],
        created_at:  datetime [auto_now],
        updated_at:  datetime [auto_now_update],
    }
}
```

---

## Foreign keys

Use the `fk(table.column, action)` option to declare a foreign key.

**Actions:** `cascade`, `set_null`, `restrict`, `set_default`, `no_action`

```rust
model! {
    Comment,
    table: "comments",
    pk: id => Pk,
    fields: {
        // FK → posts.id, delete comment when post is deleted
        post_id:    i32    [required, fk(posts.id, cascade)],
        // FK → users.id, set null when user is deleted
        author_id:  i32    [nullable, fk(users.id, set_null)],
        content:    text   [required],
        created_at: datetime [auto_now],
    }
}
```

---

## Enums

The `model!` macro supports declaring enums directly alongside the model via the optional
`enums:` block. Variants are available as a Rust enum and integrate with SeaORM and the
admin form (rendered as a `<select>`).

### Backing types

| Keyword  | Storage          | SeaORM type                             | Notes                             |
|----------|------------------|-----------------------------------------|-----------------------------------|
| `String` | Text column      | `DeriveActiveEnum(String)`              | Default when no keyword specified |
| `i32`    | Integer column   | `DeriveActiveEnum(i32)`                 |                                   |
| `i64`    | Bigint column    | `DeriveActiveEnum(i64)`                 |                                   |
| `pg`     | Native enum type | `DeriveActiveEnum` + `db_type = "Enum"` | PostgreSQL only                   |

### Enum syntax

```text
enums: {
    EnumName BackingType [Variant, Variant = "db_value", Variant = ("db_value", "Label"), ...]
}
```

Each variant can optionally specify:

- `= "db_value"` — the value stored in the database (defaults to the variant name)
- `= ("db_value", "Display label")` — db value + label shown in the admin form

### Example — String-backed enum (default)

```rust
use runique::prelude::*;

model! {
    Post,
    table: "posts",
    pk: id => Pk,
    enums: {
        Status [Draft, Published, Archived]
    },
    fields: {
        title:  String [required],
        status: Status [required, default("Draft")],
    }
}
```

Stored as a `VARCHAR` column. The generated `Status` enum implements `DeriveActiveEnum`,
`Display`, `FromStr`, `Default`, `Serialize`, and `Deserialize`.

### Example — PostgreSQL native ENUM

For PostgreSQL `CREATE TYPE` enums, use the `pg` keyword. The column type becomes
`db_type = "Enum"` with `enum_name` set to the snake_case enum name.

```rust
use runique::prelude::*;

model! {
    Task,
    table: "tasks",
    pk: id => Pk,
    enums: {
        Priority pg [Low, Medium, High]
        Visibility pg [
            Public  = ("public",  "Public"),
            Team    = ("team",    "Team only"),
            Private = ("private", "Private"),
        ]
    },
    fields: {
        title:      String     [required],
        priority:   Priority   [required, default("Low")],
        visibility: Visibility [required, default("Public")],
    }
}
```

The corresponding PostgreSQL migration must create the type before the table:

```sql
CREATE TYPE priority AS ENUM ('Low', 'Medium', 'High');
CREATE TYPE visibility AS ENUM ('public', 'team', 'private');
```

SeaORM generates the correct `ColumnType::Enum { name, variants }` for these fields.

### Variant labels in the admin

When a field uses an enum type, the admin form renders a `<select>` with one `<option>`
per variant. The displayed text is the **label** if provided, otherwise the db value.

```rust
enums: {
    Role pg [
        Admin = ("admin", "Administrator"),
        Editor = ("editor", "Editor"),
        Viewer = ("viewer", "Read-only"),
    ]
}
```

---

## Relations

The optional `relations:` block declares SeaORM relations on the model.
It generates the `Relation` enum, `Related` implementations, and `RelationTrait` used by SeaORM queries.

### Relation types

| Declaration                                     | Description                                      |
|-------------------------------------------------|--------------------------------------------------|
| `belongs_to: Model via field`                   | This model holds the FK column                   |
| `has_many: Model`                               | One-to-many (inverse of belongs_to)              |
| `has_many: Model as alias`                      | One-to-many with a custom relation name          |
| `has_one: Model`                                | One-to-one (inverse of belongs_to)               |
| `has_one: Model as alias`                       | One-to-one with a custom relation name           |
| `many_to_many: Model through JoinTable via fk`  | Many-to-many through a join table                |

### Example

```rust
use runique::prelude::*;

model! {
    User,
    table: "users",
    pk: id => Pk,
    fields: {
        username: String [required, max_len(150)],
    }
}

model! {
    Post,
    table: "posts",
    pk: id => Pk,
    fields: {
        title:     String [required],
        author_id: Pk     [required, fk(users.id, cascade)],
    },
    relations: {
        belongs_to: User via author_id,
        has_many:   Comment,
    }
}

model! {
    Comment,
    table: "comments",
    pk: id => Pk,
    fields: {
        body:    text [required],
        post_id: Pk   [required, fk(posts.id, cascade)],
    },
    relations: {
        belongs_to: Post via post_id,
    }
}
```

### Many-to-many

```rust
model! {
    Article,
    table: "articles",
    pk: id => Pk,
    fields: {
        title: String [required],
    },
    relations: {
        many_to_many: Tag through ArticleTag via article_id,
    }
}

model! {
    ArticleTag,
    table: "article_tags",
    pk: id => Pk,
    fields: {
        article_id: Pk [required, fk(articles.id, cascade)],
        tag_id:     Pk [required, fk(tags.id, cascade)],
    },
    relations: {
        belongs_to: Article via article_id,
        belongs_to: Tag via tag_id,
    }
}
```

---

## Complete example — blog application

```rust
use runique::prelude::*;

// ── Users ──────────────────────────────────────────────────────
model! {
    User,
    table: "users",
    pk: id => Pk,
    fields: {
        username:   String   [required, max_len(150), unique],
        email:      String   [required, unique],
        password:   String   [required, max_len(128)],
        is_active:  bool     [required, default(true)],
        is_staff:   bool     [required, default(false)],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}

// ── Categories ─────────────────────────────────────────────────
model! {
    Category,
    table: "categories",
    pk: id => Pk,
    fields: {
        name: String [required, max_len(100), unique],
        slug: String [required, max_len(100), unique],
    }
}

// ── Posts ──────────────────────────────────────────────────────
model! {
    Post,
    table: "posts",
    pk: id => Pk,
    fields: {
        title:       String   [required, max_len(255)],
        slug:        String   [required, unique, max_len(255)],
        content:     text     [required],
        excerpt:     String   [nullable, max_len(500)],
        author_id:   i32      [required, fk(users.id, cascade)],
        category_id: i32      [nullable, fk(categories.id, set_null)],
        is_published: bool    [required, default(false)],
        views:       i64      [required, default(0)],
        created_at:  datetime [auto_now],
        updated_at:  datetime [auto_now_update],
    }
}

// ── Comments ───────────────────────────────────────────────────
model! {
    Comment,
    table: "comments",
    pk: id => Pk,
    fields: {
        post_id:    i32  [required, fk(posts.id, cascade)],
        author_id:  i32  [nullable, fk(users.id, set_null)],
        content:    text [required],
        is_approved: bool [required, default(false)],
        created_at: datetime [auto_now],
    }
}
```

---

## `impl_objects!` — ORM manager (Django-style)

Activate the `objects` manager on a model to get Django-style queries:

```rust
use runique::prelude::*;

model! {
    Post,
    table: "posts",
    pk: id => Pk,
    fields: {
        title:        String [required],
        is_published: bool   [required, default(false)],
        views:        i64    [required, default(0)],
        author_id:    i32    [required, fk(users.id, cascade)],
        created_at:   datetime [auto_now],
    }
}

impl_objects!(Entity);
```

```rust
// In a handler:
async fn posts_handler(ctx: Request) -> Response {
    let db = ctx.db();

    // All posts
    let all = Entity::objects.all().all(db).await.unwrap();

    // Filter: published posts, sorted by views descending
    let published = Entity::objects
        .filter(Column::IsPublished.eq(true))
        .order_by_desc(Column::Views)
        .limit(10)
        .all(db)
        .await
        .unwrap();

    // Exclude drafts
    let visible = Entity::objects
        .exclude(Column::IsPublished.eq(false))
        .all(db)
        .await
        .unwrap();

    // Count
    let total = Entity::objects.count(db).await.unwrap();

    // Get by ID — returns Err if not found
    let post = Entity::objects.get(db, 1).await.unwrap();

    // Get by ID — returns None if not found
    let maybe = Entity::objects.get_optional(db, 99).await.unwrap();

    // Get or auto-404
    let post_or_404 = Entity::objects
        .get_or_404(db, 1, &ctx, "Post not found")
        .await
        .unwrap();

    ctx.render("posts/list.html", context! { posts: published, total })
}
```

---

## `#[form(...)]`

Generates a form struct from the schema produced by `model!`.

```rust
use runique::prelude::*;

model! {
    Post,
    table: "posts",
    pk: id => Pk,
    fields: {
        title:        String [required, max_len(255), label("Title")],
        content:      text   [required, label("Content")],
        excerpt:      String [nullable, max_len(500)],
        is_published: bool   [required, default(false)],
        created_at:   datetime [auto_now],
        updated_at:   datetime [auto_now_update],
    }
}

// Only expose title, content, excerpt, is_published — auto_now fields excluded
#[form(schema = post_schema, fields = [title, content, excerpt, is_published])]
pub struct PostForm;
```

### `#[form]` parameters

| Parameter  | Required | Description                                       |
|------------|----------|---------------------------------------------------|
| `schema`   | yes      | Path to the schema function generated by `model!` |
| `fields`   | no       | Whitelist — only include these fields             |
| `exclude`  | no       | Blacklist — exclude these fields                  |

`fields` and `exclude` are mutually exclusive. The primary key is always excluded.

### Using the form in a handler

```rust
pub async fn create_post(
    mut req: Request,
    Prisme(mut form): Prisme<PostForm>,
) -> impl IntoResponse {
    if form.is_valid().await {
        form.save(&req.engine.db).await.ok();
        return Redirect::to("/posts").into_response();
    }

    ctx.render("posts/new.html", context! { form })
}
```

---

## License

MIT — part of the [Runique](https://github.com/seb-alliot/runique) project.
