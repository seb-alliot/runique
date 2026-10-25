# ORM & Database

Runique uses **SeaORM** with a Django-like manager provided through the `impl_objects!` macro.

## Table of Contents

| Module | Description |
| --- | --- |
| [Manager & helpers](/docs/en/orm/manager) | `impl_objects!`, `all()`, `filter()`, `get()`, `get_or_404()`, `FormEntity` |
| [CRUD Queries](/docs/en/orm/queries) | SELECT, INSERT, UPDATE, DELETE, COUNT, `search!` |
| [Advanced](/docs/en/orm/advanced) | Transactions, relations, full CRUD pattern |

> **Form integration**: `#[form(schema = …)]` ties the form to its model module's entity automatically (`schema = blog` → `blog::Entity`), which gives `Form::objects` and `search!(@Form => ...)`. See [Forms — entity link](/docs/en/model/forms).

## Next Steps

← [**Templates**](/docs/en/template) | [**Middleware & Security**](/docs/en/middleware) →
