# Sessions

Runique uses `CleaningMemoryStore` as the default session store — a wrapper around an in-memory `HashMap` that adds automatic cleanup, watermark protection, and protection for valuable sessions.

Anonymous sessions live in memory and are lost on restart. **Authenticated** sessions are also saved in the database (`eihwaz_sessions`), automatically as soon as a database is configured: they survive a restart, and the database is the authority (see [Store & watermarks](/docs/en/session/store)).

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
