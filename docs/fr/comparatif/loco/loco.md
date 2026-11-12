# Comparatif Runique et Loco

Vérifié le 5 novembre 2026 dans [la documentation](https://loco.rs/docs/) et [le dépôt](https://github.com/loco-rs/loco) de Loco 1.3.1. « — » : rien de tel trouvé dans sa documentation ni dans son code.

## CLI

| Commande | Loco | Runique |
|----------|------|---------|
| Installer | `cargo install loco` | `cargo install runique --features sqlite --locked` |
| Créer un projet | `loco new` (assistant : type d'application, base, worker) | `runique new nom` |
| Migrations (générer) | `cargo loco generate model` / `generate migration` | `runique makemigrations` |
| Migrations (appliquer) | `cargo loco db migrate` | `runique migration up` |
| Migrations (annuler) | `cargo loco db down` | `runique migration down -n 1` |
| Statut migrations | `cargo loco db status` | `runique migration status` |
| Régénérer les entités | `cargo loco db entities` | — (`model!{}` les génère à la compilation) |
| Générateurs de code | modèle, contrôleur, scaffold, worker, mailer, tâche, planificateur, déploiement | — |
| Lister les routes | `cargo loco routes` | — |
| Lister les middlewares | `cargo loco middleware` | — |
| Diagnostic | `cargo loco doctor` | — |
| Créer un superutilisateur | — | `runique create-superuser` |
| Démarrer | `cargo loco start` | `cargo run` |
| Rechargement du code | `cargo loco watch` (avec `cargo-watch`) | — |

## Routing

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Déclaration des routes | `Routes::new().prefix("notes").add("/", get(list))` | `url.rs` avec macro `urlpatterns!{}` |
| Routes dynamiques | `.add("/{id}", get(get_one))` | `"/users/{id}"` dans `urlpatterns!` |
| Reverse URL | — | `{% link "nom_vue" %}` |
| Paramètre de chemin | extracteur Axum `Path` | `Path` Axum ou `request.get_path_as::<i32>("id")` |
| Query string | `QueryValidate<T>` ou extracteur Axum | `request.query::<MyStruct>()?` |
| Préfixe d'application | `AppRoutes` | `Router::new().nest("/prefix", ...)` |

## Vues / Handlers

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Handler | `async fn hello() -> Result<Response>` | `async fn ma_vue(mut request: Request) -> AppResult<Response>` |
| Accès DB | `State(ctx): State<AppContext>`, puis `ctx.db` | `request.engine.db` |
| Rendu | `format::render().view(&v, "home/hello.html", data)` | `request.render("template.html")` |
| Réponse JSON | `format::json(res)` | `Json` d'Axum |
| Redirection | `format::redirect("/url")` | `Redirect::to("/url")` |
| Messages flash | — | macros `success!`, `error!`, `info!`, `warning!` |
| Connexions secondaires | `MultiDbInitializer`, connexions nommées | `.with_custom_db(client)` |

## Formulaires

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Définition | struct `#[derive(Deserialize, Validate)]` (crate `validator`) | `#[form(schema = blog, fields = [...])]` ou `RuniqueForm` manuel |
| Lecture et validation | extracteurs `FormValidate<T>`, `JsonValidate<T>`, `QueryValidate<T>` | `ValidationForm::try_form(form, &request).await` |
| Rendu HTML | — | `{% form.nom_form %}` ou `{% form.nom_form.champ %}` |
| CSRF intégré | — | automatique, injecté avant le premier champ |
| Sauvegarde | code du modèle (SeaORM) | `validated.save(&db).await`, le formulaire porte son entité (`FormEntity`) |
| Validation croisée | `ValidatorTrait` | `clean()` async, avec accès à la base |
| Fichiers | `Multipart` d'Axum, stockage configurable | `FileField` et `MediaField` : type, taille, signature du contenu pour les images, l'audio et la vidéo |
| Stockage | disque, mémoire, S3, GCP, Azure | dossier `MEDIA_ROOT` |

## Templates

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Moteur | Tera, moteur remplaçable (`ViewRenderer`) | Tera |
| Emplacement | `assets/views/` | `templates/` |
| Fichiers statiques | middleware `static` (`assets/static`) | `{% static "file" %}`, avec SRI en production |
| Fichiers media | — | `{% media "file" %}` |
| URL reverse | — | `{% link "nom" %}` |
| CSRF | — | `{% csrf %}` |
| Messages | — | `{% messages %}` |
| Binaire autonome | feature `embedded_assets` | — |

## ORM / Base de données

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| ORM | SeaORM | SeaORM |
| Moteurs | SQLite, PostgreSQL (features par défaut) | SQLite, PostgreSQL, MySQL/MariaDB, un moteur par compilation |
| Définition modèle | entités générées depuis les migrations | macro `model!{}` |
| Migrations | DSL `loco_rs::schema`, écrites ou générées | `runique makemigrations` compare au dernier instantané |
| Requêtes | `query::condition()` et SeaORM | `search!` et SeaORM |
| Pagination | `query::paginate` | `.paginate(...)` de SeaORM |
| Transactions | SeaORM | `request.db_atomic()` et SeaORM |
| Multi-tenant | feature `multi-tenancy` | — |
| Extension table framework | — | `extend!{}` sur les tables `eihwaz_*` |

## Authentification

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Login / Logout | routes `/api/auth/*` générées | `authenticate_user()` + `login()`, `logout()` |
| JWT | `auth::JWT`, `auth::JWTWithUser<T>` | — |
| Clé d'API | `find_by_api_key` | — |
| Sessions | cookie signé ou chiffré, stores base et Redis (feature `auth_session`) | tower-sessions (mémoire + repli en base) |
| OAuth2 / OIDC | feature `auth_oauth` | — |
| Protection brute force | — | `LoginGuard` |
| Hashage mot de passe | Argon2id (`hash::hash_password`) | Argon2 par défaut |
| Reset password | routes `forgot` et `reset` générées | jetons hashés en base, à usage unique, avec expiration |
| Droits | — | rangs et droits des admins par compte |

## Sécurité

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| CSRF | — | toujours actif |
| CSP et en-têtes | `secure_headers` à activer, présets `github` et `owasp` | actifs par défaut, nonce par requête |
| Rate limiting | couche `tower` choisie par le projet | `RateLimiter`, déclaré par route |
| CORS | middleware `cors` | `.with_cors(...)` |
| Hôtes autorisés | — | `.with_allowed_hosts(...)` |
| Open Redirect | — | middleware toujours actif |
| IP réelle derrière un proxy | middleware `remote_ip` | `.with_trusted_proxies(...)` |

## Panel Admin

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Activation | — | builder typé `ModelAdmin` + `AdminSite.register(...)` |
| CRUD, recherche, filtres | — | natif |
| Permissions | — | rangs, droits des admins par compte (table et colonne) |
| Historique | — | natif |

## Tâches de fond

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Workers | trait `Worker`, files Redis, PostgreSQL, SQLite ou threads | — |
| Planificateur | `cargo loco scheduler`, expressions cron | — |
| Tâches ponctuelles | `cargo loco task` | — |
| Cache | mémoire, Redis, base de données | — |

## Email

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Envoi | mailers envoyés par un worker | `Email::new().send()` |
| Templates | `subject.t`, `html.t`, `text.t` (Tera) | templates Tera via `.template(&tera, "emails/mon.html", ctx.into())` |
| Backend SMTP | fichiers `config/*.yaml` | `.env` |
| Backend dev (console) | — | `EMAIL_BACKEND=console` |

## Observabilité / Logging

| Fonctionnalité | Loco | Runique |
|----------------|------|---------|
| Config logging | clés `logger` en YAML | arbre `RuniqueLog` par domaine |
| Base | `tracing` | `tracing` |
| Formats | `compact`, `pretty`, `json` | stdout, fichier JSON ou texte, `LogSink` |
| Priorité | `RUST_LOG` | `RUST_LOG` |

## Tests

| Fonctionnalité | Loco | Runique |
|----------|------|---------|
| Commande | `cargo test` | `runique test` |
| Requêtes HTTP | `request::<App, _, _>()`, `TestServer` | `reqwest` ou `tower::ServiceExt::oneshot` |
| Isolation base | `request_with_create_db`, `#[serial]` | `runique_test`, transaction par test |

## Déploiement

| Aspect | Loco | Runique |
|--------|------|---------|
| Configuration | `config/*.yaml`, `LOCO_ENV` | `.env`, `RUNIQUE_ENV` |
| Livraison | binaire `<package>-cli` | binaire |
| Générateurs | Docker, nginx, AWS Lambda | — |
| TLS automatique | — | feature `acme` |
| Licence | Apache-2.0 | MIT |
