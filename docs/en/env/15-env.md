# Environment Variables

All configuration keys available through `.env`. The listed values are the defaults applied if the variable is missing.

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Application & Server](/docs/en/env/application) | DEBUG, BASE_DIR, IP_SERVER, PORT, SECRET_KEY, DB, Redirects |
| [Assets & Media](/docs/en/env/assets) | STATICFILES_DIRS, MEDIA_ROOT, TEMPLATES_DIR and associated URLs |
| [Security & Sessions](/docs/en/env/securite) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

---

## Internationalisation (CLI)

| Variable | Default | Description |
| --- | --- | --- |
| `LANG` | system locale | CLI language (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priority: `.env` > system locale (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Next Steps

← [**Sessions**](/docs/en/session)
