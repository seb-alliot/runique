# Assets & médias

## Fichiers statiques et médias

| Variable | Défaut | Description |
|----------|--------|-------------|
| `STATICFILES_DIRS` | `static` | Dossier des fichiers statiques |
| `STATIC_URL` | `/static` | Préfixe URL pour les fichiers statiques |
| `MEDIA_ROOT` | `media` | Dossier des fichiers médias uploadés |
| `MEDIA_URL` | `/media` | Préfixe URL pour les médias |
| `STATIC_RUNIQUE_PATH` | — | Chemin vers les assets internes de Runique |
| `STATIC_RUNIQUE_URL` | `/runique/static` | Préfixe URL pour les assets de Runique |
| `MEDIA_RUNIQUE_PATH` | — | Chemin vers les médias internes de Runique |
| `MEDIA_RUNIQUE_URL` | `/runique/media` | Préfixe URL pour les médias de Runique |
| `TEMPLATES_DIR` | — | Dossier des templates Tera |
| `TEMPLATES_RUNIQUE` | — | Dossier des templates internes de Runique |
| `STATICFILES` | `default_storage` | Backend de stockage |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/application/application.md) | DEBUG, IP_SERVER, PORT, DB, Redirections |
| [Sécurité & sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/securite/securite.md) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Retour au sommaire

- [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md)
