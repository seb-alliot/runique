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

## Déploiement Docker

`STATIC_RUNIQUE_PATH` pointe vers les fichiers statiques internes du panel admin (CSS, JS). Sa valeur par défaut est résolue à la compilation via `CARGO_MANIFEST_DIR` et est invalide dans un conteneur Docker au runtime.

À déclarer explicitement dans le `.env` :

```env
STATIC_RUNIQUE_PATH=/app/runique/static
```

Puis copier le dossier correspondant depuis les sources de Runique dans l'image lors du build Docker.

Après un bump de version de la dépendance `runique`, toujours rebuilder l'image :

```bash
docker compose build
docker compose up -d
```

`docker compose up -d` seul réutilise l'ancienne image et les anciens assets.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](/docs/fr/env/application) | DEBUG, IP_SERVER, PORT, DB, Redirections |
| [Sécurité & sessions](/docs/fr/env/securite) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
