# Variables d'environnement

Toutes les clés configurables via `.env`. Les valeurs indiquées sont les défauts appliqués si la variable est absente.

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Application & Serveur](/docs/fr/env/application) | DEBUG, BASE_DIR, IP_SERVER, PORT, SECRET_KEY, DB, Redirections |
| [Assets & médias](/docs/fr/env/assets) | STATICFILES_DIRS, MEDIA_ROOT, TEMPLATES_DIR et URLs associées |
| [Sécurité & sessions](/docs/fr/env/securite) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

---

## Internationalisation (CLI)

| Variable | Défaut | Description |
| --- | --- | --- |
| `LANG` | locale système | Langue de la CLI (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priorité : `.env` > locale système (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Prochaines étapes

← [**Sessions**](/docs/fr/session)
