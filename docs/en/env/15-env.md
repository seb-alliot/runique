# Environment Variables

All configuration keys available through `.env`. The listed values are the defaults applied if the variable is missing.

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Application & Server](https://github.com/seb-alliot/runique/blob/main/docs/en/env/application/application.md) | DEBUG, BASE_DIR, IP_SERVER, PORT, SECRET_KEY, DB, Redirects |
| [Assets & Media](https://github.com/seb-alliot/runique/blob/main/docs/en/env/assets/assets.md) | STATICFILES_DIRS, MEDIA_ROOT, TEMPLATES_DIR and associated URLs |
| [Security & Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/env/securite/securite.md) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

---

## Internationalisation (CLI)

| Variable | Default | Description |
| --- | --- | --- |
| `LANG` | system locale | CLI language (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priority: `.env` > system locale (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Next Steps

← [**Sessions**](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)
