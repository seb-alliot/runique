# Environment Variables

All configuration keys available through `.env`. The listed values are the defaults applied if the variable is missing.

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Application & Server](/docs/en/env/application) | RUNIQUE_ENV, DEBUG, BASE_DIR, IP_SERVER, PORT, SECRET_KEY, DB, email sending |
| [Assets & Media](/docs/en/env/assets) | STATICFILES_DIRS, MEDIA_ROOT, TEMPLATES_DIR and associated URLs |
| [Security & Sessions](/docs/en/env/security) | Middlewares, CSP, Sessions |

---

## Internationalisation (CLI)

| Variable | Default | Description |
| --- | --- | --- |
| `LANG` | `en` | Language of the application and of the `runique` command: one of Runique's nine (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`) or a language of the project's `locales/` folder. Application: the builder's `.with_lang()` > `.env` > `en`. Command: `.env` > system locale > `en`. See [i18n](/docs/en/configuration/i18n) |

---

## Next Steps

← [**Sessions**](/docs/en/session)
