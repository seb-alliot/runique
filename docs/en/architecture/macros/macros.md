# Rust Macros

Runique provides a set of macros to simplify development.

---

## Context Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `context!` | Create a Tera context | `context!("title" => "Page")` |
| `context_update!` | Add to a Request's context | `context_update!(request => { "key" => value })` |

---

## Flash Message Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `success!` | Success message (session) | `success!(request.notices => "OK!")` |
| `error!` | Error message (session) | `error!(request.notices => "Error")` |
| `info!` | Info message (session) | `info!(request.notices => "Info")` |
| `warning!` | Warning (session) | `warning!(request.notices => "Warning")` |
| `flash_now!` | Immediate message (no session) | `flash_now!(error => "Errors")` |

---

## Routing Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `urlpatterns!` | Define named routes | `urlpatterns!("/" => view!{...}, name = "index")` |
| `view!` | Handler for all HTTP methods | `view!{ GET => handler, POST => handler2 }` |
| `impl_objects!` | Django-like manager for SeaORM | `impl_objects!(Entity)` |

---

## Error Macros

| Macro | Description |
| ----- | ----------- |
| `impl_from_error!` | Generates `From<Error>` for `AppError` |

---

## See also

| Section | Description |
| --- | --- |
| [Key concepts](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/concepts/concepts.md) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Tera tags & filters](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/tera/tera.md) | Django-like tags, filters, functions |
| [Middleware stack](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/middleware/middleware.md) | Slot order, dependency injection |
| [Request lifecycle](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/lifecycle/lifecycle.md) | Lifecycle, best practices |

## Back to summary

- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/02-architecture.md)
