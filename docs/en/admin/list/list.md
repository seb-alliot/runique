# List view — Pagination, sorting, search and filters

The admin list works **at SQL level**: pagination, sort, search and filters. Nothing is loaded into memory just to page.

Config is declared on `ModelAdmin` via `.list(|l| …)`.

## Contents

- [Displayed columns](#displayed-columns)
- [Detail links](#detail-links)
- [Sorting](#sorting)
- [Search](#search)
- [Filters](#filters)
- [Bulk actions](#bulk-actions)
- [Pagination](#pagination)
- [URL parameters](#url-parameters)

## Displayed columns

```rust
.list(|l| l.display([Title, Status, Views]))
```

Without `display`, every non-secret column appears (accounts: username, email, is_active).

Array order is header order.

## Detail links

```rust
.list(|l| l.display([Title, Status]).links([Title]))
```

Without `links`, the first `display` column becomes the link.

## Sorting

Default SQL order:

```rust
.list(|l| {
    l.display([Title, CreatedAt])
        .ordering([(CreatedAt, Order::Desc), (Title, Order::Asc)])
})
```

Clickable sort from the UI / URL — **only** declared columns:

```rust
.sortable([Title, Status])
```

Any other `?sort_by=` is ignored.

## Search

```rust
.list(|l| {
    l.search([Title, Summary])
        .search_help("Title or summary")
})
```

Behaviour:

- minimum **3** characters (shorter terms are ignored)
- max length bounded by staging `.search_max_len`
- without `.search(...)`: text columns from `display` are used

## Filters

```rust
.list(|l| l.filter([Status, Lang]))
```

Each filter offers **distinct** values in the allowed scope (max 100). URL prefix is `filter_`:

```text
?filter_status=Published&filter_lang=en
```

## Bulk actions

```rust
.list(|l| {
    l.display([Title, Status])
        .action(Status, "Published", "Publish")
        .action(Status, "Archived", "Archive")
})
```

The UI sends the action **index** (`a0`, `a1`…), never the raw value. The value is fixed in code.

Boot constraints:

- column writable on edit
- no secret, no owner, no account rank flags

## Pagination

| Level | Method | Default |
| --- | --- | --- |
| Site | `AdminStaging::page_size(n)` | 10 |
| Model | `ListConfig::per_page(n)` | inherits site |

```rust
admin.page_size(15)

.list(|l| l.per_page(30))
```

## URL parameters

| Param | Role |
| --- | --- |
| `page` | Page number (1-based) |
| `sort_by` | Column name (must be in `sortable`) |
| `sort_dir` | `asc` or `desc` |
| `search` | Full-text term |
| `filter_<col>` | Filter value |

Example:

```text
/admin/blog/?page=2&sort_by=title&sort_dir=asc&search=runique&filter_status=Published
```

## Empty value

```rust
.list(|l| l.empty_value("—"))
```

## See also

| Section | Description |
| --- | --- |
| [ModelAdmin builder](/docs/en/admin/declaration-macro) | `.list` / `.form` syntax |
| [Permissions](/docs/en/admin/permission) | Columns cut by rights |
| [Templates](/docs/en/admin/template) | List HTML override |

## Back to summary

- [Admin summary](/docs/en/admin)
