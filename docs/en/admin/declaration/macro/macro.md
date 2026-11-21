# `ModelAdmin` builder

Typed API to declare a model's list and form. Columns are SeaORM enum variants (`Column::Title`): a typo does not compile.

## Skeleton

```rust
fn articles() -> ModelAdmin<article::Entity> {
    use article::Column::*;
    ModelAdmin::new()
        .title("Articles")
        .list(|l| {
            l.display([Title, Status, Views])
                .sortable([Title, Status])
                .ordering([(Title, Order::Asc)])
                .search([Title, Summary])
                .search_help("Title or summary")
                .filter([Status])
                .per_page(15)
                .empty_value("—")
                .action(Status, "Published", "Publish")
                .action(Status, "Archived", "Archive")
        })
        .form(|f| {
            f.fields([Title, Summary, Body, Status, Views])
                .readonly([Views])
                .create_only([Title])
                .labels([(Summary, "Summary"), (Body, "Body")])
        })
}
```

Keep `use …::Column::*` **inside** the model function so two models with a `Title` column never clash.

## `ModelAdmin` methods

| Method | Role |
| --- | --- |
| `.title("…")` | Name in nav and pages (else `verbose_name_plural`, `verbose_name`, then the table name) |
| `.list(…)` | List view config |
| `.form(…)` | Create / edit config |
| `.with_form::<F>()` | Builds create / edit from a form of the project, see "Project form" below |
| `.templates(…)` | Overrides of this model's blocks or pages (HTML, CSS), see [Template override](/docs/en/admin/template-surcharge) |

## List — `ListConfig`

| Method | Effect |
| --- | --- |
| `.display([Col, …])` | Visible columns, in order |
| `.links([Col, …])` | Columns linking to the detail. **No effect for now** (see Limits): each row has its link to the detail |
| `.sortable([Col, …])` | Only columns accepted via `?sort_by=` |
| `.ordering([(Col, Order::Asc), …])` | Default SQL order |
| `.search([Col, …])` | Full-text columns. Omitted: text columns of `display`. `.search([])` disables |
| `.search_help("…")` | Search help text. **No effect for now** (see Limits) |
| `.filter([Col, …])` | Side filters (`?filter_status=…`) |
| `.per_page(n)` | Page size for this model |
| `.filter_size(n)` | Values per page of the filter groups for this model |
| `.empty_value("—")` | Text for empty values. **No effect for now** (see Limits) |
| `.action(Col, value, "Label")` | Bulk action: set `Col` to `value` on checked rows |

### Bulk actions

```rust
.list(|l| {
    l.display([Title, Status])
        .action(Status, "Published", "Publish")
        .action(Status, "Archived", "Archive")
})
```

Boot rules:

- action column must be **writable** on edit (in `fields`, not `readonly` / `create_only`)
- forbidden on secrets and the owner, and, on the accounts table only, on the rank flags and the activation (`is_superuser`, `is_dirigeant`, `is_admin`, `is_active`)

## Form — `FormConfig`

| Method | Effect |
| --- | --- |
| `.fields([Col, …])` | Shown fields, in order |
| `.readonly([Col, …])` | Visible but not editable |
| `.create_only([Col, …])` | Writable on create, frozen afterwards |
| `.labels([(Col, "Label"), …])` | Custom labels |

```rust
.form(|f| {
    f.fields([Slug, Title, Body, ViewCount])
        .create_only([Slug])
        .readonly([ViewCount])
        .labels([(ViewCount, "Views")])
})
```

Without `.form()`, every field of the model's form (`ModelMeta::Form`) is used. The gate still decides what the current account may read or write.

## Project form — `with_form`

By default, the create / edit pages are built from the form `model!{}` generates. `.with_form::<F>()` builds them from a `#[form(schema = …)]` form of the project: its `customize()` and its `clean` then apply in the admin too.

```rust
// src/forms.rs
#[form(schema = article, fields = [title, body, image])]
pub struct ArticleForm;

impl RuniqueForm for ArticleForm {
    fn customize(form: &mut Forms) {
        // The old image is deleted when a new one replaces it.
        form.field(&MediaField::image("image").upload_to("articles").replace());
    }
}

// src/admin.rs: the form's type is written in the return type
fn article() -> ModelAdmin<article::Entity, ArticleForm> {
    use article::Column::*;
    ModelAdmin::new()
        .with_form::<ArticleForm>()
        .list(|l| l.display([Title]))
}
```

A form of another model does not compile. The rest is checked at boot, and the application refuses to start if:

- a field of the form is neither a column nor a list field of the model;
- a field is named after a secret column, or after a rank flag on the accounts table;
- a field can't hold its column's type;
- a column needed to create a row (NOT NULL, no default) has no field.

`.form(…)` combines with `with_form`: `fields`, `readonly`, `create_only` and `labels` apply to the form given. Rights don't change: the gate still decides what the account reads and writes, and secret columns are never written by the admin. The form can only narrow.

## Extended accounts

To surface columns added by `extend!` on `eihwaz_users`:

```rust
fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Username, Email, IsActive, Website])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email, Bio, Website, Phone]))
}

// in site():
site.accounts_users(accounts())
```

Rank flags (`is_superuser`, `is_dirigeant`, `is_admin`) are not form fields. Boot refuses them. Rank is changed from the account page. `write::insert` refuses a `Set(true)` on these columns, `write::update` refuses any `Set`. `is_superuser` is set with `runique create-superuser`. Detail: [Permissions](/docs/en/admin/permission).

## Secrets

Secret columns (password, tokens…) are refused in display, search, filter, sortable, links, fields, and actions. The error shows up in boot `validate()`.

## Current limits

- `.links`, `.search_help` and `.empty_value` are accepted and checked at boot, but the list's display doesn't use them yet: the list shows each row as a card with its link to the detail, no help text under the search, and empty values as they are.

## Real example (demo-app)

```rust
fn blog() -> ModelAdmin<blog::Entity> {
    use blog::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Title, Email, Website, Status])
                .sortable([Title, Status])
                .ordering([(Title, Order::Asc)])
                .search([Title, Summary])
                .filter([Status])
                .empty_value("—")
                .action(Status, "Published", "Publish")
                .action(Status, "Archived", "Archive")
        })
        .form(|f| {
            f.fields([Title, Email, Website, Summary, Content, Status, ViewCount])
                .readonly([ViewCount])
        })
}
```

## See also

| Section | Description |
| --- | --- |
| [List view](/docs/en/admin/list) | Runtime list behaviour |
| [Boot validation](/docs/en/admin/declaration-daemon) | Automatic checks |
| [Declaration](/docs/en/admin/declaration) | AdminStaging and AdminSite |

## Back to summary

- [Admin summary](/docs/en/admin)
