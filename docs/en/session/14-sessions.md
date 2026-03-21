# Sessions

Runique uses `CleaningMemoryStore` as the default session store — a wrapper around an in-memory `HashMap` that adds automatic cleanup, watermark protection, and protection for valuable sessions.

Data is lost when the server restarts. For persistence, use an external store (Redis, database).

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Store & Watermarks](/docs/en/session/store) | `CleaningMemoryStore`, low/high watermarks, memory estimation |
| [Protection](/docs/en/session/protection) | Automatic protection (`user_id`), manual (`session_active`), shopping cart use cases |
| [Usage & Configuration](/docs/en/session/usage) | Session access in handlers, `.env`, builder |

---

## Next Steps

← [**Authentication**](/docs/en/auth) | [**Environment Variables**](/docs/en/env) →
