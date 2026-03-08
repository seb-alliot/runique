```markdown
# Sessions

Runique uses `CleaningMemoryStore` as the default session store — a wrapper around an in-memory `HashMap` that adds automatic cleanup, watermark protection, and protection for valuable sessions.

Data is lost when the server restarts. For persistence, use an external store (Redis, database).

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Store & Watermarks](https://github.com/seb-alliot/runique/blob/main/docs/en/session/store/store.md) | `CleaningMemoryStore`, low/high watermarks, memory estimation |
| [Protection](https://github.com/seb-alliot/runique/blob/main/docs/en/session/protection/protection.md) | Automatic protection (`user_id`), manual (`session_active`), shopping cart use cases |
| [Usage & Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/session/usage/usage.md) | Session access in handlers, `.env`, builder |

---

## Next Steps

← [**Authentication**](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md) | [**Environment Variables**](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md) →
```
