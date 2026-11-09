# Admin filter system: how it works
> Runique · Axum + SeaORM + Tera: from the `.filter([...])` declaration to the sidebar, with the real code and the Rust techniques used at each step.

## Objectives

- Understand the path of a filter: declaration, startup check, SQL queries, URL, template
- See how types (`Column`) prevent typos
- Build composed SeaORM conditions (`Condition::all`, `Condition::any`)
- Paginate distinct values without counting them
- Handle URL parameters without trusting the visitor
- Build links that add or remove a filter with an iterator chain

## Table of contents

1. Overview
2. Step 1: Declaring a filter
3. Step 2: Startup check
4. Step 3: The offered values
5. Step 4: Applying the chosen filters
6. Step 5: The sidebar links
7. Step 6: The template
8. Step 7: Collapsing groups in the browser
9. Security
10. Summary of design choices
11. Exercises
12. Key points

## 1. Overview

The sidebar of the admin list shows, for each filterable column, its **distinct values**. Clicking a value filters the list. Everything is done in SQL: no table is loaded into memory.

```
src/admin.rs           ModelAdmin::new().list(|l| l.filter([Lang, SectionId]))
      │
startup                check: no secret column
      │
GET request            /admin/doc_page?filter_lang=fr&filter_lang=en&page=1
      │
list handler           1. distinct values of each filterable column (sidebar)
                       2. conditions of the chosen filters + list + total
      │
Tera template          collapsible groups, links that add or remove a value
```

## 2. Step 1: Declaring a filter

```rust
// src/admin.rs
fn doc_page() -> ModelAdmin<doc_page::Entity> {
    use doc_page::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, Lang, SectionId])
            .filter([Lang, SectionId])
            .filter_size(10)              // values per group before the pagination arrows
    })
}
```

```rust
// runique/src/admin/builder/list.rs (excerpt)
pub fn filter(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
    self.filter = cols.into_iter().collect();
    self
}
```

What the signature shows:

- `E::Column`: the type of the columns **of the administered entity**. Passing `users::Column::Email` to a `ModelAdmin<doc_page::Entity>` does not compile, and neither does a misspelled column (`Lnag`);
- `impl IntoIterator<Item = ...>`: you can pass an array, a `Vec`, or any iterator (course `generics`);
- `mut self -> Self`: a by-value builder (course `orm`, section 2.7).

> **Tip**: filter on columns with **few values**: booleans, enums, short codes, foreign keys. Filtering on a title or a date to the second produces as many values as there are rows.

## 3. Step 2: Startup check

The compiler guarantees that a column exists. What it cannot know is that a column is **secret**. Each entity declares its secret columns (`E::SECRET`, for example the password), and the builder refuses them in all lists:

```rust
// runique/src/admin/builder/admin_builder.rs (excerpt)
for secret in E::SECRET {
    let name = secret.as_str();
    for list in self
        .list
        .lists_using(name)
        .into_iter()
        .chain(self.form.lists_using(name))
    {
        errors.push(format!(
            "{list}: `{name}` is a secret column, never shown, searched, filtered, sorted or written by the admin"
        ));
    }
}
```

A `.filter([Password])` therefore stops **startup** with a message that names the faulty list. The choice: an error at startup rather than at the first request, so that no dangerous configuration reaches production.

Rust technique: `.into_iter().chain(...)` goes through two sources (lists of the list view, lists of the form) as one, without building an intermediate collection.

## 4. Step 3: The offered values

### 4.1 The query

```rust
// runique/src/admin/builder/list.rs (excerpt)
/// The values a filter offers for `col`: the distinct ones of the rows in
/// `scope`, as text (how `resolve_filters` compares them), sorted, NULL
/// left out, `page` of `size` of them. One more than `size` is read: its
/// presence says a next page exists, without counting the values.
pub(crate) fn filter_values(
    backend: DbBackend,
    scope: Condition,
    col: E::Column,
    page: u64,
    size: u64,
) -> Select<E> {
    let cast = Alias::new(text_cast_type_of(backend));
    // The databases take a signed 64-bit offset: a huge page from the URL
    // stays in range instead of failing the query.
    let offset = std::cmp::Ord::min(
        page.saturating_sub(1).saturating_mul(size),
        i64::MAX.unsigned_abs(),
    );
    E::find()
        .filter(scope)
        .filter(col.is_not_null())
        .select_only()
        .column_as(Expr::col((E::default(), col)).cast_as(cast), FILTER_VALUE)
        .distinct()
        .order_by_asc(Expr::col(Alias::new(FILTER_VALUE)))
        .offset(offset)
        .limit(size.saturating_add(1))
}
```

The SQL produced looks like:

```sql
SELECT DISTINCT CAST("lang" AS TEXT) AS filter_value
FROM "doc_page"
WHERE <allowed scope> AND "lang" IS NOT NULL
ORDER BY filter_value ASC
LIMIT 11 OFFSET 0
```

### 4.2 The choices, one by one

| Element | Why |
|---|---|
| `.filter(scope)` | only the rows the account may see offer their values: a value does not leak through the sidebar |
| `is_not_null()` | `NULL` is not a clickable value |
| `cast_as(text)` | booleans, integers, enums and texts become text: a single code path for all types |
| `text_cast_type_of(backend)` | the name of the text type differs by engine (Postgres, MySQL, SQLite) |
| `distinct()` | one value per row of the bar, even if a thousand rows carry it |
| `limit(size + 1)` | read **one value more** than the page: if it exists, there is a next page, without `COUNT` |
| `saturating_sub`, `saturating_mul`, `min(.., i64::MAX)` | an oversized page in the URL (a twenty-digit page number) causes neither an integer overflow nor a database error |

### 4.3 The "one more" technique

```rust
let mut values: Vec<String> = /* result of filter_values, size + 1 at most */ vec![];
let size = 10;
let next_page = values.len() as u64 > size;
values.truncate(size as usize);       // show only the requested page
```

Counting all the distinct values would cost one more query and a full scan. Reading `size + 1` rows is enough to know whether to show the "next" arrow.

### 4.4 Saturating arithmetic

```rust
let page: u64 = 0;                         // a page 0 coming from the URL
let offset = page.saturating_sub(1)        // 0 instead of an overflow (0 - 1)
    .saturating_mul(10);                   // stays within the bounds of a u64
```

Any numeric value coming from a URL can be 0, huge or absurd: `saturating_*` keeps it within bounds without panicking (course `variables-et-fonctions`, section 5).

## 5. Step 4: Applying the chosen filters

### 5.1 The URL

```
?filter_lang=fr&filter_lang=en&filter_section_id=3
```

A value is a `filter_<column>=<value>` parameter. Several values of the same column are repeated.

### 5.2 The condition

```rust
// runique/src/admin/builder/list.rs (excerpt)
pub(crate) fn resolve_filters(
    &self,
    backend: DbBackend,
    filters: &[(String, String)],
) -> Option<Condition> {
    let cast = Alias::new(text_cast_type_of(backend));
    let mut cond = Condition::all();                         // AND between columns
    for col in &self.filter {                                // only the DECLARED columns
        let one_of = filters
            .iter()
            .filter(|(name, _)| name == col.as_str())
            .fold(Condition::any(), |any, (_, value)| {      // OR between the values of one column
                any.add(
                    Expr::col((E::default(), *col))
                        .cast_as(cast.clone())
                        .eq(value.as_str()),
                )
            });
        if !one_of.is_empty() {
            cond = cond.add(one_of);
        }
    }
    (!cond.is_empty()).then_some(cond)
}
```

For the URL of section 5.1:

```sql
WHERE (CAST(lang AS TEXT) = 'fr' OR CAST(lang AS TEXT) = 'en')
  AND (CAST(section_id AS TEXT) = '3')
```

### 5.3 The Rust techniques

| Technique | Line | Effect |
|---|---|---|
| loop over `self.filter` | `for col in &self.filter` | a `filter_password=...` or `filter_unknown=...` parameter is **ignored**: no column is taken from the URL |
| `fold` | `.fold(Condition::any(), ...)` | builds a condition from a list, without a mutable variable |
| `Condition::all()` / `Condition::any()` | | SeaORM's AND / OR, nestable |
| `cast_as` on both sides | column as text, value as text | the offered values (section 4) and the compared values have the same form |
| `bool::then_some` | `(!cond.is_empty()).then_some(cond)` | `None` when no filter is active: no useless `WHERE` |

The value coming from the URL (`value.as_str()`) becomes a **bound parameter** of the query: never SQL. A `filter_lang=' OR 1=1 --` is a string compared to `lang`, which matches nothing.

### 5.4 The same filter written differently

```rust
use sea_orm::Condition;

// With a loop and a mutable variable
fn one_column_v1(values: &[&str]) -> Condition {
    let mut any = Condition::any();
    for v in values {
        any = any.add(doc_page::Column::Lang.eq(*v));
    }
    any
}

// With fold (Runique's form)
fn one_column_v2(values: &[&str]) -> Condition {
    values.iter().fold(Condition::any(), |any, v| any.add(doc_page::Column::Lang.eq(*v)))
}

// With IN: a single expression
fn one_column_v3(values: &[&str]) -> Condition {
    Condition::all().add(doc_page::Column::Lang.is_in(values.iter().copied()))
}
```

`IN` would be shorter, but Runique compares as **text** to handle all column types the same way: the `cast_as` and `OR` form keeps a single code path for a boolean, an integer or an enum.

## 6. Step 5: The sidebar links

Each displayed value is a link that **adds** the filter if it is inactive, or **removes** it if it is active, keeping all the other parameters.

```rust
// runique/src/admin/router/context.rs (excerpt)
pub struct FilterOption {
    pub value: String,
    /// Whether the list is filtered on this value.
    pub active: bool,
    /// `?…` of the list with this value turned on or off.
    pub qs: String,
}

impl FilterOption {
    pub(crate) fn new(params: &[(String, String)], column: &str, value: String) -> Self {
        let name = format!("filter_{column}");
        let is_it = |k: &str, v: &str| k == name && v == value;
        let active = params.iter().any(|(k, v)| is_it(k, v));
        let qs = encoded_query(
            params
                .iter()
                .filter(|(k, v)| k != "page" && !is_it(k, v))      // all parameters except the page and this value
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .chain((!active).then_some((name.as_str(), value.as_str())))   // + this value if it was inactive
                .chain(std::iter::once(("page", "1"))),             // + back to the first page
        );
        Self { value, active, qs }
    }
}
```

The iterator chain, line by line:

1. `filter`: keeps all current parameters, except `page` and the value itself;
2. `chain(Option)`: an `Option` is iterable (zero or one element): the value is added **only** if it was not active, hence the toggle;
3. `chain(once(..))`: we go back to page 1, since the number of results changes;
4. `encoded_query` encodes everything for the URL (an `&` or an `=` in a value does not break the link).

| Technique | Use here |
|---|---|
| capturing closure (`is_it`) | reuse the same test in `any` and in `filter` |
| `Option` as an iterator | add an element conditionally in a chain |
| `std::iter::once` | add a fixed element |
| `bool::then_some` | turn a condition into an `Option` |

A neighbouring function, `filter_clear_qs`, builds the "clear all" link of a group: same parameters, without any `filter_<column>`.

## 7. Step 6: The template

```html
{# runique/templates/admin/blocks/list/filters.html (excerpt) #}
{% for filter_entry in resource.display.list_filter %}
  {% set col = filter_entry[0] %}
  {% set label = filter_entry[1] %}
  {% set values = filter_values[col] %}
  {% if values and values | length > 0 %}
  <div class="admin-filter__group" data-filter-col="{{ col }}">
    <button class="admin-filter__group-title" type="button" aria-expanded="true">{{ label }}</button>
    <div class="admin-filter__group-body">
      {% if filter_clear[col] %}
      <a href="{{ filter_clear[col] }}" hx-get="{{ filter_clear[col] }}"
        hx-target="#list-content" hx-swap="outerHTML" hx-push-url="true"
        class="admin-filter__option admin-filter__option--clear">{{ admin_list_filters_clear }}</a>
      {% endif %}
      {# Each link turns its value on or off, the others kept. #}
      {% for option in values %}
        <a href="{{ option.qs }}" hx-get="{{ option.qs }}"
          hx-target="#list-content" hx-swap="outerHTML" hx-push-url="true"
          class="admin-filter__option {% if option.active %}admin-filter__option--active{% endif %}">
          {{ option.value | replace(from="_", to=" ") }}
        </a>
      {% endfor %}
      {% set pager = filter_pagers[col] %}
      {% if pager.prev_qs or pager.next_qs %}
        {% include "admin/blocks/list/pager.html" %}
      {% endif %}
    </div>
  </div>
  {% endif %}
{% endfor %}
```

- `list_filter`: the declared columns, with their label, in declaration order;
- `filter_values[col]`: the page of values (`value`, `active`, `qs`) from section 6; `qs` already starts with `?`;
- `filter_clear[col]`: the "clear all" link of the group (`filter_clear_qs`);
- `filter_pagers[col]`: the arrows from section 4.3, present only if there is a page before or after;
- `hx-get`, `hx-target`, `hx-push-url`: htmx replaces only the list (`#list-content`) on click, and updates the URL; without JavaScript, the `href` reloads the whole page with the same result;
- `active_filters` and `filter_qs`: the state of the active filters, reused by the pagination and sort links so the selection is not lost;
- `{{ option.value }}` is escaped by Tera (autoescape active on `.html`): a value containing HTML is displayed as text, without being executed.

## 8. Step 7: Collapsing groups in the browser

Each group can be collapsed. Groups are **collapsed by default**; the choice is kept in the browser's `sessionStorage`, per resource and per column:

```js
// runique/static/js/admin/admin-list.js (excerpt)
const KEY = 'runique_fg_' + RESOURCE + '_' + col;
const stored = sessionStorage.getItem(KEY);
// Replié par défaut sauf préférence explicite (sessionStorage)
```

```
key:  runique_fg_doc_page_lang
      '1'               → open
      other or absent   → collapsed
```

`sessionStorage` lives as long as the tab: the state survives navigation between the pages of the list, without storing anything on the server or in a cookie. The key contains the resource: two lists have independent groups.

## 9. Security

| Risk | Protection |
|---|---|
| filtering on an unplanned column (`filter_password=...`) | only the columns of `.filter([...])` are read (loop over the declaration, section 5.2) |
| declaring a secret column | refused at startup (section 3) |
| SQL injection through a value | value passed as a bound parameter, never as SQL text |
| leaking values outside the account's rights | the offered values are computed within the allowed scope (`scope`) |
| oversized page in the URL | saturating arithmetic, bounded offset |
| HTML in a value | automatic escaping by Tera |
| loading a big table into memory | `DISTINCT`, `LIMIT` and `OFFSET` on the database side |

## 10. Summary of design choices

| Choice | Rejected alternative | Why |
|---|---|---|
| Typed columns (`Column::…`) | free strings | compile-time error |
| Startup check | check at request time | a dangerous configuration never reaches production |
| Distinct values in SQL | load the whole table | holds up on big tables |
| Read `size + 1` values | `COUNT(DISTINCT ...)` | one query fewer |
| Comparison as text (`CAST`) | one code path per column type | a single path for all types |
| Whitelist `.filter([...])` | accept any `filter_*` | no column name is read from the URL |
| Allowed scope in the values query | values from the whole table | no value leaks outside the rights |
| `sessionStorage` per resource and column | global state, cookie | independent groups, nothing on the server |

## 11. Exercises

### Exercise 1: read the condition

What condition does `?filter_status=published&filter_status=draft&filter_author=7&filter_password=x` produce with `.filter([Status, Author])`?

Answer: `(status = 'published' OR status = 'draft') AND (author = '7')`, as a text comparison. `filter_password` is ignored: the column is not declared.

### Exercise 2: next page

Write a function that receives the values read (at most `size + 1`) and returns the page to display and whether a next page exists.

```rust
// Solution:
fn split_page(mut values: Vec<String>, size: usize) -> (Vec<String>, bool) {
    let next = values.len() > size;
    values.truncate(size);
    (values, next)
}

assert_eq!(split_page(vec!["a".into(), "b".into(), "c".into()], 2), (vec!["a".into(), "b".into()], true));
assert_eq!(split_page(vec!["a".into()], 2), (vec!["a".into()], false));
```

### Exercise 3: toggle a parameter

Write `toggle(params, name, value) -> Vec<(String, String)>` that adds `(name, value)` if it is absent, removes it if it is present, and keeps the rest.

```rust
// Solution:
fn toggle(params: &[(String, String)], name: &str, value: &str) -> Vec<(String, String)> {
    let present = params.iter().any(|(k, v)| k == name && v == value);
    params
        .iter()
        .filter(|(k, v)| !(k == name && v == value))
        .cloned()
        .chain((!present).then(|| (name.to_string(), value.to_string())))
        .collect()
}
```

## 12. Key points

- Filterable columns are typed: a typo does not compile.
- Secret columns are refused at startup, not at request time.
- The offered values are distinct, computed in SQL, within the allowed scope, and paginated by reading one extra value.
- Only declared columns are read from the URL; values are bound parameters.
- OR between the values of a column, AND between columns.
- Links are built with an iterator chain: `filter`, `chain(Option)`, `once`.
