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
| `RUNIQUE_MAX_UPLOAD_MB` | `100` | Taille maximale d'**un** fichier envoyé (MB), pour toute l'application. Voir *Taille des envois* ci-dessous |
| `RUNIQUE_MAX_BODY_MB` | `100`, jamais moins que `RUNIQUE_MAX_UPLOAD_MB` | Taille maximale d'**une requête** avec fichiers (MB) : tous ses fichiers cumulés, plus 1 Mo pour les champs texte. Voir *Taille des envois* ci-dessous |
| `RUNIQUE_MAX_TEXT_FIELD_KB` | `1024` | Taille maximale d'un champ texte multipart (KB) |

---

### Taille des envois

Un envoi de fichiers passe trois limites, de la plus fine à la plus large :

| Limite | Porte sur | Défaut | Se règle |
| --- | --- | --- | --- |
| `.max_size()` du champ | chaque fichier de ce champ | 10 Mo | dans le formulaire : `.max_size(FileSize::mb(50))` |
| `RUNIQUE_MAX_UPLOAD_MB` | chaque fichier, tous champs confondus | 100 Mo | dans le `.env` |
| `RUNIQUE_MAX_BODY_MB` | la requête entière : tous ses fichiers + 1 Mo de champs texte | 100 Mo (+ 1 Mo) | dans le `.env` |

Le `.max_size()` d'un champ ne peut pas dépasser `RUNIQUE_MAX_UPLOAD_MB` ni `RUNIQUE_MAX_BODY_MB` : un champ déclaré plus haut y est ramené (y compris par `form.max_size(...)` dans le handler), et l'indication affichée sous le champ montre cette vraie limite.

Un corps **sans fichier** (formulaire urlencoded, JSON) lu par Runique est limité à **2 Mo**, quelles que soient ces variables. Les fichiers sont écrits sur disque au fil de l'envoi ; ces corps-là sont lus en mémoire, et 100 Mo de mémoire par requête seraient offerts à n'importe quel visiteur.

#### Les modifier

```env
# Fichiers jusqu'à 200 Mo, requête jusqu'à 300 Mo (+ 1 Mo)
RUNIQUE_MAX_UPLOAD_MB=200
RUNIQUE_MAX_BODY_MB=300
```

Les deux valeurs sont lues au démarrage : **redémarrez l'application** après les avoir changées. Une valeur non numérique est ignorée et le défaut s'applique.

#### Selon ce que vous définissez

| `.env` | Un fichier jusqu'à | Une requête jusqu'à | Pour |
| --- | --- | --- | --- |
| rien | 100 Mo | 101 Mo | images, documents, courtes vidéos : le cas courant |
| `RUNIQUE_MAX_UPLOAD_MB=500` | 500 Mo | 501 Mo (suit le fichier) | un gros fichier par envoi (vidéo) |
| `RUNIQUE_MAX_BODY_MB=300` | 100 Mo | 301 Mo | plusieurs fichiers dans un même formulaire |
| `RUNIQUE_MAX_UPLOAD_MB=10` + `RUNIQUE_MAX_BODY_MB=30` | 10 Mo | 31 Mo | 3 PDF de 10 Mo, par exemple, et rien de plus gros |
| `RUNIQUE_MAX_UPLOAD_MB=200` + `RUNIQUE_MAX_BODY_MB=50` | 200 Mo | 201 Mo | la requête plus petite qu'un fichier est relevée au plafond par fichier |

Dans tous les cas, chaque champ reste à son `.max_size()` (10 Mo par défaut) : relever les variables ne relève pas les champs. Une vidéo de 200 Mo demande `.max_size(FileSize::mb(200))` sur le champ **et** `RUNIQUE_MAX_UPLOAD_MB=200`.

Un formulaire à plusieurs fichiers additionne leurs tailles : avec les défauts, deux fichiers de 60 Mo (chacun sous 100 Mo) font une requête de 120 Mo, refusée. Relevez `RUNIQUE_MAX_BODY_MB` au total attendu.

Baisser les valeurs sous les défauts réduit ce qu'un visiteur peut faire écrire sur le disque par requête ; un `.max_size()` plus haut devient alors inatteignable. Ces limites portent sur **une** requête, pas sur le volume total : limitez aussi le débit des routes d'envoi (`.rate_limit(...)` dans `urlpatterns!`).

#### Quand une limite est dépassée

| Dépassement | Ce qui se passe |
| --- | --- |
| un fichier au-delà du `.max_size()` de son champ | Le fichier est reçu, puis refusé par le champ : le formulaire est réaffiché avec l'erreur sous le champ, rien n'est enregistré |
| un fichier au-delà de `RUNIQUE_MAX_UPLOAD_MB` | La requête est refusée pendant l'envoi (413), le fichier partiel est supprimé ; le handler n'est pas appelé |
| une requête au-delà de `RUNIQUE_MAX_BODY_MB` + 1 Mo | Même chose (413) |
| un corps sans fichier au-delà de 2 Mo | La requête est refusée (413) |

Pour un refus pendant l'envoi (413), le serveur répond avant que le navigateur ait fini d'envoyer : la plupart des navigateurs n'affichent pas la réponse, mais une erreur de connexion. Le message (« fichier trop volumineux ») est lisible par un client HTTP (API, `curl`) et dans le journal. Pour prévenir le visiteur, donnez à `.max_size()` une valeur sous les limites globales : le refus vient alors du champ, avec son message dans la page.

Un extracteur axum (`Json`, `Bytes`, `Multipart`…) utilisé directement dans un handler suit `RUNIQUE_MAX_BODY_MB`, pas les 2 Mo de Runique. Pour le borner davantage, posez `axum::extract::DefaultBodyLimit::max(...)` sur sa route.

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
| `OG_IMAGE` | `/runique/static/runique_320.avif` | Image Open Graph par défaut |

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
