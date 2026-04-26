# Assets & médias

## Fichiers statiques et médias

| Variable | Défaut | Description |
|----------|--------|-------------|
| `STATICFILES_DIRS` | `static` | Dossier des fichiers statiques |
| `STATIC_URL` | `/static` | Préfixe URL pour les fichiers statiques |
| `MEDIA_ROOT` | `media` | Dossier des fichiers médias uploadés |
| `MEDIA_URL` | `/media` | Préfixe URL pour les médias |
| `TEMPLATES_DIR` | `templates` | Dossier des templates Tera (liste séparée par virgules possible) |
| `STATICFILES` | `default_storage` | Backend de stockage |
| `RUNIQUE_MAX_UPLOAD_MB` | `100` | Taille maximale globale d'un upload fichier (MB) |
| `RUNIQUE_MAX_TEXT_FIELD_KB` | `1024` | Taille maximale d'un champ texte multipart (KB) |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](/docs/fr/env/application) | DEBUG, IP_SERVER, PORT, DB, Redirections |
| [Sécurité & sessions](/docs/fr/env/securite) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
