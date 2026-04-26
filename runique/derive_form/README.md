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
    {
        field: type [option1, option2, ...],
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
    {
        title:      text [required, max_length: 255],
        content:    richtext [required],
        slug:       text [required, unique],
        views:      int [required, default: 0],
        published:  bool [required, default: false],
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
    { title: text [required] }
}

// UUID PK
model! {
    Session,
    table: "sessions",
    pk: token => uuid,
    { user_id: int [required] }
}
```

---

## Field types

Fields default to **nullable** unless `required` is specified.

| Type          | SQL type                    | Notes                                    |
|---------------|-----------------------------|------------------------------------------|
| `text`        | `VARCHAR(255)`              | Short text field                         |
| `textarea`    | `TEXT`                      | Long text, multi-line                    |
| `richtext`    | `TEXT`                      | Rich text (HTML editor)         |
| `email`       | `VARCHAR(255)`              | Email — validated format                 |
| `password`    | `VARCHAR(255)`              | Password — hashed on save                |
| `url`         | `VARCHAR(255)`              | URL — validated format                   |
| `slug`        | `VARCHAR(255)`              | Slug — auto-generated from title         |
| `color`       | `VARCHAR(255)`              | Hex color                                |
| `ip`          | `VARCHAR(255)`              | IP address                               |
| `int`         | `INTEGER`                   |                                          |
| `bigint`      | `BIGINT`                    |                                          |
| `float`       | `DOUBLE`                    |                                          |
| `decimal`     | `DECIMAL`                   |                                          |
| `percent`     | `DOUBLE`                    | Stored as float                          |
| `bool`        | `BOOLEAN`                   |                                          |
| `date`        | `DATE`                      |                                          |
| `time`        | `TIME`                      |                                          |
| `datetime`    | `DATETIME`                  |                                          |
| `timestamp`   | `TIMESTAMP`                 |                                          |
| `timestamp_tz`| `TIMESTAMPTZ`               | With timezone (Postgres)                 |
| `uuid`        | `UUID`                      |                                          |
| `json`        | `JSON`                      |                                          |
| `image`       | `VARCHAR(255)`              | Stores file path — upload handled by app |
| `document`    | `VARCHAR(255)`              | Stores file path                         |
| `file`        | `VARCHAR(255)`              | Stores file path                         |
| `choice`      | `VARCHAR` or native enum    | Requires `enum(EnumName)` — see Enums    |
| `radio`       | `VARCHAR` or native enum    | Same as `choice`, rendered as radio      |

```rust
model! {
    Profile,
    table: "profiles",
    pk: id => Pk,
    {
        username:   text     [required, max_length: 50, unique],
        bio:        textarea,
        score:      float    [required, default: 0.0],
        is_active:  bool     [required, default: true],
        birth_date: date,
        avatar:     image,
        token:      uuid     [required, unique],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
```

---

## Field options

| Option                  | Description                                                        |
|-------------------------|--------------------------------------------------------------------|
| `required`              | Column is `NOT NULL` + form validation                             |
| `unique`                | UNIQUE constraint                                                  |
| `index`                 | Create a database index                                            |
| `default: value`        | Default value: `0`, `true`, `"draft"`, etc.                        |
| `max_length: n`         | Max string length (validation + `VARCHAR(n)`)                      |
| `min_length: n`         | Min string length (validation)                                     |
| `max: n` / `min: n`     | Max / min integer value (validation)                               |
| `max_f: n` / `min_f: n` | Max / min float value (validation)                                 |
| `auto_now`              | Set to `NOW()` on INSERT                                           |
| `auto_now_update`       | Set to `NOW()` on UPDATE                                           |
| `rows: n`               | Number of rows for `textarea` / `richtext` in admin                |
| `step: n`               | Step for numeric fields in forms                                   |
| `max_size: n`         | Max upload size (e.g., 2 GB, 500 KB) for file fields              |
| `upload_to: "path"`     | Upload directory for file fields (required for files)             |
| `enum(EnumName)`        | Enum reference for `choice` / `radio` fields                       |
| `skip`                  | Column generated in SQL but excluded from forms                    |
| `no_hash`               | Prevent auto-hashing for `password` fields                         |

```rust
model! {
    Product,
    table: "products",
    pk: id => Pk,
    {
        name:        text    [required, max_length: 200],
        description: textarea,
        price:       float   [required, min_f: 0.0],
        stock:       int     [required, default: 0, min: 0],
        sku:         text    [required, max_length: 50, unique],
        is_active:   bool    [required, default: true],
        banner:      image   [upload_to: "products/", max_size: 2 MB],
        created_at:  datetime [auto_now],
        updated_at:  datetime [auto_now_update],
    }
}
```

---

## Foreign keys

Use the `fk(table.column, action)` option to declare a foreign key.

**Actions:** `cascade`, `set_null`, `restrict`, `set_default`

```rust
model! {
    Comment,
    table: "comments",
    pk: id => Pk,
    {
        post_id:    int  [required],
        author_id:  int,
        content:    textarea [required],
        created_at: datetime [auto_now],
    },
    relations: {
        belongs_to: Post via post_id [cascade],
        belongs_to: User via author_id [set_null],
    }
}
```

---

## Enums

The `model!` macro supports declaring enums directly alongside the model via the optional
`enums:` block. Variants are rendered as a `<select>` in admin forms.

### Engine detection

Enum storage depends on the database engine detected at **compile time**.
Runique reads `DB_ENGINE` (or the prefix of `DATABASE_URL`) from the `.env` at the
crate root. Make sure at least one of these is set:

```env
DB_ENGINE=postgres    # or mysql / sqlite
# or
DATABASE_URL=postgresql://user:pass@localhost/db
```

If neither is found, compilation fails with an explicit error.

### Auto detection (default)

When no backing type is specified, the enum adapts automatically:

| Engine   | Storage             | SeaORM type               |
|----------|---------------------|---------------------------|
| Postgres | Native `ENUM` type  | `db_type = "Enum"`        |
| MySQL    | `VARCHAR`           | `db_type = "String"`      |
| SQLite   | `VARCHAR`           | `db_type = "String"`      |

### Enum syntax

```text
enums: {
    EnumName: [Variant, Variant = "db_value", Variant = ("db_value", "Label"), ...],
}
```

Each variant can optionally specify:

- `= "db_value"` — value stored in the database (defaults to variant name)
- `= ("db_value", "Display label")` — db value + label shown in admin

### Example

```rust
use runique::prelude::*;

model! {
    Task,
    table: "tasks",
    pk: id => Pk,
    enums: {
        Status: [
            Draft:    "Brouillon",
            Active:   "Publié",
            Archived: "Archivé",
        ],
        Priority: [Low, Medium, High],
    },
    {
        title:    text     [required],
        status:   choice   [enum(Status), required, default: "draft"],
        priority: choice   [enum(Priority), required],
    }
}
```

---

## Relations

The optional `relations:` block declares SeaORM relations on the model.

### Relation types

| Declaration                                     | Description                              |
|-------------------------------------------------|------------------------------------------|
| `belongs_to: Model via field`                   | This model holds the FK column           |
| `has_many: Model`                               | One-to-many (inverse of belongs_to)      |
| `has_many: Model as alias`                      | One-to-many with a custom relation name  |
| `has_one: Model`                                | One-to-one (inverse of belongs_to)       |
| `has_one: Model as alias`                       | One-to-one with custom relation name     |
| `many_to_many: Model through JoinTable via fk`  | Many-to-many through a join table        |

### Relations example

```rust
use runique::prelude::*;

model! {
    Post,
    table: "posts",
    pk: id => Pk,
    {
        title:     text [required],
        author_id: int  [required, fk(users.id, cascade)],
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
    {
        body:    textarea [required],
        post_id: int      [required, fk(posts.id, cascade)],
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
    {
        title: text [required],
    },
    relations: {
        many_to_many: Tag through ArticleTag via article_id,
    }
}

model! {
    ArticleTag,
    table: "article_tags",
    pk: id => Pk,
    {
        article_id: int [required, fk(articles.id, cascade)],
        tag_id:     int [required, fk(tags.id, cascade)],
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

// ── Categories ─────────────────────────────────────────────────
model! {
    Category,
    table: "categories",
    pk: id => Pk,
    {
        name: text [required, max_length: 100, unique],
        slug: slug [required, unique],
    }
}

// ── Posts ──────────────────────────────────────────────────────
model! {
    Post,
    table: "posts",
    pk: id => Pk,
    enums: {
        PostStatus: [
            Draft     = ("draft",     "Draft"),
            Published = ("published", "Published"),
            Archived  = ("archived",  "Archived"),
        ],
    },
    {
        title:        text     [required, max_length: 255],
        slug:         slug     [required, unique],
        content:      richtext [required],
        excerpt:      textarea,
        status:       choice   [enum(PostStatus), required, default: "draft"],
        author_id:    int      [required, fk(users.id, cascade)],
        category_id:  int      [fk(categories.id, set_null)],
        views:        bigint   [required, default: 0],
        created_at:   datetime [auto_now],
        updated_at:   datetime [auto_now_update],
    },
    relations: {
        has_many: Comment,
    }
}

// ── Comments ───────────────────────────────────────────────────
model! {
    Comment,
    table: "comments",
    pk: id => Pk,
    {
        post_id:     int      [required, fk(posts.id, cascade)],
        author_id:   int      [fk(users.id, set_null)],
        content:     textarea [required],
        is_approved: bool     [required, default: false],
        created_at:  datetime [auto_now],
    },
    relations: {
        belongs_to: Post via post_id,
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
    {
        title:        text [required],
        is_published: bool [required, default: false],
        views:        bigint [required, default: 0],
        author_id:    int   [required, fk(users.id, cascade)],
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

    // Count
    let total = Entity::objects.count(db).await.unwrap();

    // Get by ID — returns Err if not found
    let post = Entity::objects.get(db, 1).await.unwrap();

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

The `schema` parameter takes the **module** containing the entity
(not the `schema()` function directly).

```rust
use runique::prelude::*;
use crate::entities::post;

model! {
    Post,
    table: "posts",
    pk: id => Pk,
    {
        title:        text  [required, max_length: 255],
        content:      richtext [required],
        excerpt:      textarea,
        is_published: bool  [required, default: false],
        created_at:   datetime [auto_now],
        updated_at:   datetime [auto_now_update],
    }
}

// Expose title, content, excerpt, is_published — auto_now fields excluded automatically
#[form(schema = post, fields = [title, content, excerpt, is_published])]
pub struct PostForm;
```

### `#[form]` parameters

| Parameter  | Required | Description                                       |
|------------|----------|---------------------------------------------------|
| `schema`   | yes      | Module path of the entity generated by `model!`   |
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
