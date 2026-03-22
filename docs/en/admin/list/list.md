# List view — Pagination, sorting, search and filters

The admin list view handles all its operations **at the SQL level**: pagination, column sorting, full-text search and value filters. Nothing is loaded into memory.

## Table of Contents

- [Displayed columns — list_display](#displayed-columns--list_display)
- [Sidebar filters — list_filter](#sidebar-filters--list_filter)
- [Column sorting](#column-sorting)
- [Search](#search)
- [Pagination](#pagination)
- [URL parameters](#url-parameters)

## Displayed columns — list_display

By default, all entity columns are shown. Declaring `list_display` restricts the display to an ordered selection:

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin"],
        list_display: [
            ["username", "Username"],
            ["email", "Email"],
            ["is_active", "Active"],
        ]
    }
}
```

Each entry is a `["column_name", "Label"]` pair. Columns declared in `list_display` also serve as the **sorting whitelist**: only these columns (and `id`) accept a `sort_by` parameter.

Available in the Tera context via `visible_columns` (list of names) and `column_labels` (corresponding labels).

## Sidebar filters — list_filter

Declaring `list_filter` enables a sidebar showing the distinct values of each field:

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Users",
        permissions: ["admin"],
        list_filter: [
            ["is_active", "Active"],                    // default: 10 values per page
            ["is_superuser", "Superuser"],              // default: 10 values per page
            ["username", "Username", 5],                // explicit limit: 5 values per page
        ]
    }
}
```

- For each column, the daemon generates a SQL query that loads distinct values with `LIMIT` / `OFFSET`.
- Sidebar pagination (`‹ 1/N ›`) is **fully server-side** via the `fp_{column}` URL parameter — no JavaScript required.
- The per-page limit is configured **per column** via the optional 3rd element (default: `10`).
- Values are displayed as-is (`CAST(column AS TEXT)`). **Do not use `list_filter` on foreign key (FK) or `id` columns** — the raw value (`35`, `128`…) is not human-readable. Use the search bar instead.
- Good candidates: booleans, enumerations, short codes (`lang`, `status`, `block_type`).
- Clicking a value applies a `WHERE column = value` filter to the SQL query.
- Multiple filters on different columns are combined with `AND`.

## Column sorting

Each column header is clickable. The resulting URL:

```text
/admin/users/list?sort_by=email&sort_dir=asc&page=1
```

- `sort_by`: column name (must be in `list_display` or be `id`)
- `sort_dir`: `asc` or `desc`
- Changing the sort automatically resets to **page 1**

The current direction is shown by ▲ / ▼ in the header. Clicking the same column a second time reverses the direction.

## Search

The search bar filters records across all visible text columns (`ILIKE '%term%'`). It preserves the active sort parameters.

```text
/admin/users/list?search=alice&sort_by=email&sort_dir=asc
```

The entry counter reflects the filtered result.

## Pagination

Pagination is computed at the SQL level (`LIMIT` / `OFFSET`). Page size is configured at the application level:

```rust
.with_admin(|a| {
    a.site_title("Administration")
     .auth(RuniqueAdminAuth::new())
     .page_size(15)   // ← entries per page
})
```

Active filters and search are preserved in pagination links.

## URL parameters

| Parameter | Value | Description |
| --- | --- | --- |
| `page` | integer ≥ 1 | Current page (default: 1) |
| `sort_by` | column name | Sort column |
| `sort_dir` | `asc` \| `desc` | Sort direction (default: `asc`) |
| `search` | string | Search term |
| `filter_{column}` | value | Active filter on a column |
| `fp_{column}` | integer ≥ 0 | Current page of a sidebar filter group (0-indexed) |

All parameters can be combined. Priority order: filters → search → sort → pagination.

## See also

- [Macro `admin!`](/docs/en/admin/declaration) — `list_display`, `list_filter` syntax
- [Template context](/docs/en/admin/template/clef/context) — Tera keys for the list view
- [Template override](/docs/en/admin/template/surcharge) — customise the list rendering

## Back to summary

- [Admin summary](/docs/en/admin)
