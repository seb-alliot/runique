# ORM & Database

Runique uses **SeaORM** with a Django-like manager provided through the `impl_objects!` macro.

## Table of Contents

| Module | Description |
| --- | --- |
| [Manager & helpers](/docs/en/orm/manager) | `impl_objects!`, `all()`, `filter()`, `get()`, `get_or_404()`, `FormEntity` |
| [CRUD Queries](/docs/en/orm/queries) | SELECT, INSERT, UPDATE, DELETE, COUNT, `search!` |
| [Advanced](/docs/en/orm/advanced) | Transactions, relations, full CRUD pattern |

> **Form integration**: via `#[form(model = Entity)]`, a form exposes `Form::objects` and supports `search!(@Form => ...)`. See [Forms — model attribute](/docs/en/model/forms/forms).

## Next Steps

← [**Templates**](/docs/en/template) | [**Middleware & Security**](/docs/en/middleware) →
