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
    pk: id => i32,
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

| Syntax            | Column type          | Notes                     |
|-------------------|----------------------|---------------------------|
| `pk: id => i32`   | `INTEGER` (32-bit)   | Auto-increment by default |
| `pk: id => i64`   | `BIGINT` (64-bit)    | Auto-increment by default |
| `pk: id => uuid`  | `UUID`               | No auto-increment         |

```rust
// Integer PK (most common)
model! {
    Article,
    table: "articles",
    pk: id => i32,
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
    pk: id => i32,
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
    pk: id => i32,
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
    pk: id => i32,
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

## Complete example — blog application

```rust
use runique::prelude::*;

// ── Users ──────────────────────────────────────────────────────
model! {
    User,
    table: "users",
    pk: id => i32,
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
    pk: id => i32,
    fields: {
        name: String [required, max_len(100), unique],
        slug: String [required, max_len(100), unique],
    }
}

// ── Posts ──────────────────────────────────────────────────────
model! {
    Post,
    table: "posts",
    pk: id => i32,
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
    pk: id => i32,
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
    pk: id => i32,
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
    pk: id => i32,
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
