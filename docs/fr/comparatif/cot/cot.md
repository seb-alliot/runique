# Comparatif Runique et Cot

Vérifié le 5 novembre 2026 dans [le guide](https://cot.rs/guide/latest/) et [le dépôt](https://github.com/cot-rs/cot) de Cot 0.7. « — » : rien de tel trouvé dans sa documentation ni dans son code.

## CLI

| Commande | Cot | Runique |
|----------|-----|---------|
| Installer | `cargo install --locked cot-cli` | `cargo install runique --features sqlite --locked` |
| Créer un projet | `cot new nom` | `runique new nom` |
| Migrations (générer) | `cot migration make` | `runique makemigrations` |
| Migration vide | `cot migration new <nom>` | — |
| Migrations (appliquer) | automatique au démarrage | `runique migration up` |
| Migrations (annuler) | `cargo run migration rollback <nom>` | `runique migration down -n 1` |
| Statut migrations | `cot migration list` | `runique migration status` |
| Créer un superutilisateur | code de `App::init` dans l'exemple du guide | `runique create-superuser` |
| Collecter les statiques | `cargo run -- collect-static public/` | — |
| Démarrer | `cargo run` | `cargo run` |
| Rechargement du code | `bacon` (`bacon.toml` généré) | — |

## Routing

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Déclaration des routes | `Router::with_urls([Route::with_handler_and_name("/", index, "index")])` | `url.rs` avec macro `urlpatterns!{}` |
| Routes dynamiques | `"/hello/{name}"` | `"/users/{id}"` dans `urlpatterns!` |
| Reverse URL | `cot::reverse!(urls, "nom")` | `{% link "nom_vue" %}` |
| Paramètre de chemin | extracteur `Path(name): Path<String>` | `Path` Axum ou `request.get_path_as::<i32>("id")` |

## Vues / Handlers

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Handler | `async fn index() -> cot::Result<Html>` | `async fn ma_vue(mut request: Request) -> AppResult<Response>` |
| Accès DB | extracteur `db: Database` | `request.engine.db` |
| Rendu template | `IndexTemplate { .. }.render()?` | `request.render("template.html")` |
| Redirection | `reverse_redirect!(request, "nom")` | `Redirect::to("/url")` |
| Messages flash | — | macros `success!`, `error!`, `info!`, `warning!` |
| Pages d'erreur | `error_handler()`, page de debug avec `debug = true` | templates `404.html`, `500.html`, page de debug avec `DEBUG=true` |
| API | feature `openapi`, `swagger-ui` | — |

## Formulaires

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Définition | `#[derive(Form)]` | `#[form(schema = blog, fields = [...])]` ou `RuniqueForm` manuel |
| Lecture et validation | `ContactForm::from_request(&mut request).await?` | `ValidationForm::try_form(form, &request).await` |
| Validation intégrée | `#[form(opts(max_length = 100))]` | contraintes du modèle ou du champ |
| Validation personnalisée | `context.add_error(FormErrorTarget::Field("title"), ...)` | `clean()` async, avec accès à la base |
| Rendu HTML | `{{ form }}` ou `form.fields()` | `{% form.nom_form %}` ou `{% form.nom_form.champ %}` |
| CSRF intégré | — | automatique, injecté avant le premier champ |
| Fichiers | `form/fields/files.rs` | `FileField` et `MediaField` : type, taille, signature du contenu pour les images, l'audio et la vidéo |

## Templates

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Moteur | Askama, templates liés à une struct | Tera |
| Héritage | `{% extends %}` / `{% block %}` | idem |
| Inclusion | `{% include %}` | idem |
| Fichiers statiques | `static_files!(...)`, extracteur `StaticFiles` | `{% static "file" %}`, avec SRI en production |
| Version des statiques | `rewrite = "query_param"` | `?v=` automatique |
| URL reverse | `cot::reverse!(urls, "nom")` | `{% link "nom" %}` |
| Internationalisation | — | `t()` et `tf()` en Rust, 9 langues pour les textes du framework |

## ORM / Base de données

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| ORM | ORM propre (`sea-query`, `sqlx`) | SeaORM |
| Moteurs | SQLite, PostgreSQL, MySQL, activés ensemble par défaut | SQLite, PostgreSQL, MySQL/MariaDB, un moteur par compilation |
| Définition modèle | `#[model]` sur une struct | macro `model!{}` |
| Clé étrangère | `ForeignKey<User>` | `belongs_to`, avec action de suppression obligatoire |
| Requêtes | `query!(Customer, $id == 5)`, `Customer::objects()` | `search!`, `Model::objects` |
| Insertion en lot | `bulk_insert` | SeaORM |
| Transactions | `db.begin()`, imbrication par savepoints | `request.db_atomic()` et SeaORM |
| Migration personnalisée | `#[migration_op]`, `forwards` et `backwards` | migration SeaORM écrite à la main |

## Authentification

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Login / Logout | extracteur `Auth` : `auth.login(...)`, `auth.logout()` | `authenticate_user()` + `login()`, `logout()` |
| Utilisateurs | `DatabaseUser`, `DatabaseUserApp` | table `eihwaz_users`, extensible par `extend!{}` |
| Sessions | stores mémoire, base, fichier, Redis | tower-sessions (mémoire + repli en base), store personnalisable |
| Cookie de session | `Secure`, `HttpOnly`, `SameSite=Strict` par défaut, configurables | `HttpOnly`, `SameSite=Strict`, `Secure` en production |
| Hashage mot de passe | Argon2 | Argon2 par défaut |
| Protection brute force | — | `LoginGuard` |
| Reset password | — | jetons hashés en base, à usage unique, avec expiration |

## Sécurité

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| CSRF | — | toujours actif |
| CSP et en-têtes de sécurité | — | actifs par défaut, nonce par requête |
| Rate limiting | — | `RateLimiter`, déclaré par route |
| CORS | — | `.with_cors(...)` |
| Hôtes autorisés | — | `.with_allowed_hosts(...)` |
| Open Redirect | — | middleware toujours actif |

## Panel Admin

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Activation | `#[derive(AdminModel)]` + `DefaultAdminModelManager` | builder typé `ModelAdmin` + `AdminSite.register(...)` |
| CRUD complet | natif | natif |
| Pagination | native | `.page_size(n)`, `.per_page(n)` |
| Recherche / filtres | — | `.search([...])`, `.filter([...])` |
| Actions de groupe | — | `.action(Col, value, "Libellé")` |
| Accès | utilisateur connecté | rangs, droits des admins par compte (table et colonne) |
| Historique | — | natif |

## Email et cache

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Envoi email | `EmailMessage::builder()`, feature `email` | `Email::new().send()` |
| Backend dev (console) | `type = "console"` | `EMAIL_BACKEND=console` |
| Configuration | `[email]` en TOML | `.env` |
| Cache | mémoire, Redis, fichier (`Cache`) | — |

## Observabilité / Logging

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Base | `tracing` | `tracing` |
| Configuration | — | arbre `RuniqueLog` par domaine, stdout, fichier, `LogSink` |

## Tests

| Fonctionnalité | Cot | Runique |
|----------------|-----|---------|
| Commande | `cargo test` | `runique test` |
| Unitaires | `TestRequestBuilder` | `tower::ServiceExt::oneshot` |
| Intégration | `Client::new(MyProject)` | `reqwest` |
| Base de test | `TestDatabase::new_sqlite()`, `new_postgres`, `new_mysql` | `runique_test`, transaction par test |
| Bout en bout | `#[cot::e2e_test]` | — |

## Déploiement

| Aspect | Cot | Runique |
|--------|-----|---------|
| Configuration | `config/dev.toml`, `config/prod.toml` | `.env`, `RUNIQUE_ENV` |
| TLS automatique | — | feature `acme` |
| Licence | MIT ou Apache-2.0 | MIT |
