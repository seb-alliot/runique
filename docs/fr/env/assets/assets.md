# Assets & médias

## Fichiers statiques et médias

| Variable | Défaut | Description |
|----------|--------|-------------|
| `STATICFILES_DIRS` | `static` | Dossier des fichiers statiques |
| `STATIC_URL` | `/static` | Préfixe URL sous lequel ce dossier est servi |
| `MEDIA_ROOT` | `{BASE_DIR}/media`, sinon `{dossier courant}/media` | Dossier des fichiers envoyés par les utilisateurs. Créé au démarrage s'il n'existe pas ; le démarrage échoue s'il ne peut pas l'être |
| `MEDIA_URL` | `/media` | Préfixe URL sous lequel ce dossier est servi |
| `TEMPLATES_DIR` | `templates` | Dossier(s) des templates Tera, séparés par des virgules |
| `STATICFILES` | `default_storage` | Réservée : lue mais pas encore utilisée |
| `RUNIQUE_MAX_UPLOAD_MB` | non définie | Taille maximale d'un fichier uploadé (MB). Non définie : une requête est limitée à 2 Mo (valeur d'axum). Définie : la requête entière est limitée à cette valeur + 1 Mo pour les champs texte |
| `RUNIQUE_MAX_TEXT_FIELD_KB` | `1024` | Taille maximale d'un champ texte multipart (KB) |

---

### Chemins relatifs et préfixes

- Un chemin relatif (`static`, `templates`, `media`) est lu depuis le **dossier où l'application est lancée** : lancez `cargo run` depuis la racine du projet, et fixez `WorkingDirectory` (systemd) ou `BASE_DIR` en production.
- Changer `STATIC_URL` ou `MEDIA_URL` déplace les URL publiques : `{% static %}` et `{% media %}` suivent automatiquement. Un chemin écrit en dur dans un template (`/static/...`) ne suit pas.
- Les fichiers ne sont servis par Runique que si le builder l'active (`.statics()`, ou `.static_files(...)`). Derrière un Nginx qui sert lui-même `/static/` et `/media/`, les préfixes des deux côtés doivent correspondre.
- Un dossier de templates introuvable ne bloque pas le démarrage : la page échoue au rendu avec « template not found ».

### Ressources internes de Runique

| Variable | Défaut | Description |
|----------|--------|-------------|
| `STATIC_RUNIQUE_PATH` | `<sources de runique>/static` | Fichiers statiques internes (admin, pages d'erreur, page d'accueil par défaut) |
| `STATIC_RUNIQUE_URL` | `/runique/static` | Préfixe URL sous lequel ils sont servis |
| `MEDIA_RUNIQUE_PATH` | `<sources de runique>/media` | Médias internes |
| `MEDIA_RUNIQUE_URL` | `/runique/media` | Préfixe URL des médias internes |
| `TEMPLATES_RUNIQUE` | `<sources de runique>/templates` | Templates internes |
| `OG_IMAGE` | `/runique/static/favicon_runique.ico` | Image Open Graph par défaut |

`<sources de runique>` est le dossier de la crate au moment de la compilation (`CARGO_MANIFEST_DIR`).

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
| [Application & Serveur](/docs/fr/env/application) | RUNIQUE_ENV, DEBUG, IP_SERVER, PORT, DB |
| [Sécurité & sessions](/docs/fr/env/securite) | Middlewares, CSP, Sessions |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
