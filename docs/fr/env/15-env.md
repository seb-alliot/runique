# Variables d'environnement

Toutes les clés configurables via `.env`. Les valeurs indiquées sont les défauts appliqués si la variable est absente.

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Application & Serveur](/docs/fr/env/application) | RUNIQUE_ENV, DEBUG, BASE_DIR, IP_SERVER, PORT, SECRET_KEY, DB, envoi de courriel |
| [Assets & médias](/docs/fr/env/assets) | STATICFILES_DIRS, MEDIA_ROOT, TEMPLATES_DIR et URLs associées |
| [Sécurité & sessions](/docs/fr/env/securite) | Middlewares, CSP, Sessions |

---

## Internationalisation (CLI)

| Variable | Défaut | Description |
| --- | --- | --- |
| `LANG` | `en` | Langue de l'application et de la commande `runique` : une des neuf de Runique (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`) ou une langue du dossier `locales/` du projet. Application : `.with_lang()` du builder > `.env` > `en`. Commande : `.env` > locale système > `en`. Voir [i18n](/docs/fr/configuration/i18n) |

---

## Prochaines étapes

← [**Sessions**](/docs/fr/session)
