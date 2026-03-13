# Variables d'environnement

Toutes les clés configurables via `.env`. Les valeurs indiquées sont les défauts appliqués si la variable est absente.

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Application & Serveur](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/application/application.md) | DEBUG, BASE_DIR, IP_SERVER, PORT, SECRET_KEY, DB, Redirections |
| [Assets & médias](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/assets/assets.md) | STATICFILES_DIRS, MEDIA_ROOT, TEMPLATES_DIR et URLs associées |
| [Sécurité & sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/securite/securite.md) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

---

## Internationalisation (CLI)

| Variable | Défaut | Description |
| --- | --- | --- |
| `LANG` | locale système | Langue de la CLI (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priorité : `.env` > locale système (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Prochaines étapes

← [**Sessions**](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md)
