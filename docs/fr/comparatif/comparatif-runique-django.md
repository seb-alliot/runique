# Comparatif Runique vs Django

## CLI

| Commande | Django | Runique |
|----------|--------|---------|
| Créer un projet | `django-admin startproject nom` | `runique new nom` |
| Créer une app | `python manage.py startapp nom` | — |
| Migrations (générer) | `python manage.py makemigrations` | `runique makemigrations` |
| Migrations (appliquer) | `python manage.py migrate` | `runique migration up` (wrapper `sea-orm-cli migrate up`) |
| Migrations (annuler) | `python manage.py migrate app 0001` | `runique migration down --files ...` (wrapper `sea-orm-cli migrate down`) |
| Statut migrations | — | `runique migration status` (wrapper `sea-orm-cli migrate status`) |
| Créer superuser | `python manage.py createsuperuser` | `runique create-superuser` |
| Démarrer les services | `python manage.py runserver` | `cargo run` — `runique start` uniquement pour initialiser/renouveler la vue admin |

---

## Routing

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Déclaration des routes | `urls.py` avec `path()` | `url.rs` avec macro `urlpatterns!{}` |
| Routes dynamiques | `path('users/<int:id>/', view)` | `"/users/{id}"` dans `urlpatterns!` |
| Namespaces | `app_name` + `include()` | `Router::new().nest("/prefix", ...)` |
| Reverse URL | `{% url "nom_vue" %}` natif | `{% link "nom_vue" %}` → Tera function custom |
| Récupérer un paramètre de chemin | `kwargs['id']` via `request.resolver_match` | `Path(id): Path<i32>` en paramètre de vue (extractor Axum) — `request.path_param("id")` [prévu](../../ROADMAP.md#4c-requestpath_param-et-requestquery_param) |
| Récupérer un query param | `request.GET.get('key')` | `Query(params): Query<HashMap<...>>` en paramètre de vue — `request.query_param("key")` [prévu](../../ROADMAP.md#4c-requestpath_param-et-requestquery_param) |

---

## Vues / Handlers

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Vue fonction | `def ma_vue(request)` | `async fn ma_vue(...)` |
| Vue classe | `class MaVue(View)` | — |
| Accès session | `request.session` | `request.session` via `context::template::Request` (ou `Session` extractor axum directement) |
| Accès DB | `Model.objects.get(...)` | sea-orm query builders |
| Rendu template | `render(request, "template.html", ctx)` | `request.render("template.html")` — contexte déjà dans `request.context` |
| Redirect | `redirect("nom_url")` | `Redirect::to("/url")` ou `reverse(&engine, "nom")` / `reverse_with_parameters(...)` (prelude) |
| Messages flash | `messages.success(request, "...")` | `success!(message => "...")` — macros `success!`, `error!`, `info!`, `warning!` (prelude) |

---

## Formulaires

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Définition | `class MonForm(forms.Form)` / `class MonForm(ModelForm)` | `#[derive(RuniqueForm)] struct MonForm` (équivalent `ModelForm`) |
| Validation | `form.is_valid()` | `form.is_valid().await` |
| Champs disponibles | CharField, EmailField, etc. | TextField, EmailField, PasswordField, HiddenField, etc. (liste fixe, pas de widget custom) |
| Rendu HTML | `{{ form.as_p }}` | `{% form.nom_form %}` (formulaire entier) ou `{% form.nom_form.champ %}` (champ individuel) |
| CSRF intégré | automatique | automatique — injecté par le filtre Tera `form_filter` avant le premier champ |
| Sauvegarde | `form.save()` | `form.save(&db).await` |
| Validation async | non | oui (accès DB possible) |
| Formulaires fichier | `FileField` | Multipart partiel |

---

## Templates

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Moteur | Django Template Language | Tera (syntaxe Jinja2) |
| Héritage | `{% extends %}` / `{% block %}` | idem Tera |
| Fichiers statiques | `{% load static %}` `{% static "file" %}` | `{% static "file" %}` natif |
| Fichiers media | `{{ MEDIA_URL }}file` | `{% media "file" %}` natif |
| URL reverse | `{% url "nom" %}` | `{% link "nom" %}` |
| CSRF | `{% csrf_token %}` | `{% csrf %}` |
| Messages | `{% for m in messages %}` | `{% messages %}` |
| Internationalisation | `{% trans "..." %}` | `{{ t("section.clé") }}` |

---

## ORM / Base de données

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| ORM | Django ORM natif | sea-orm |
| Définition modèle | `class User(models.Model)` | entité sea-orm (struct Rust) dans `src/entities/` (dossier imposé, lu par le parser) |
| Migrations auto | oui (détection changements) | `runique makemigrations` (détection changements depuis entités) |
| QuerySet chaînable | `User.objects.filter(...).order_by(...)` | sea-orm Select builder |
| Relations | ForeignKey, ManyToMany, OneToOne | Relations sea-orm |
| Transactions | `with transaction.atomic()` | `db.transaction(...)` sea-orm |
| Multi-DB | oui | PostgreSQL, MySQL, SQLite |
| NoSQL | via packages tiers | via crates tierces (ex. `mongodb`) |
| Re-export | — | `runique::sea_orm` + `sea_query` |

---

## Authentification

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Login / Logout | `authenticate()` + `login()` | `login()`, `login_staff()`, `logout()` — `LoginGuard` = middleware anti-brute force |
| Vérif authentification | `request.user.is_authenticated` | `is_authenticated(&session).await` |
| Utilisateur courant | `request.user` | `CurrentUser` (injecté via `load_user_middleware`) |
| Protection route | `@login_required` | middleware `login_required` |
| Redirection si connecté | manuel | middleware `redirect_if_authenticated` |
| Sessions | natif | tower-sessions |
| Protection brute force | `django-axes` (tiers) | `LoginGuard` natif (tentatives + lockout) |
| Hashage mot de passe | PBKDF2 / argon2 | argon2, bcrypt, scrypt, custom (détection automatique à la vérification) |
| Activation compte email | natif (`auth`) | **manquant** |
| Reset password | natif | **manquant** (prévu via `lettre`) |
| Déconnexion forcée toutes sessions | oui | **manquant** |

---

## Sécurité

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| CSRF | natif | natif (constant-time via `subtle`) |
| CSP | `django-csp` (tiers) | natif (`use_nonce: true` par défaut) |
| HSTS | `SECURE_HSTS_SECONDS` | natif (`max-age=31536000; includeSubDomains`) |
| SameSite cookies | configurable | `Strict` par défaut |
| HttpOnly cookies | par défaut | toujours `true` |
| Validation hôtes | `ALLOWED_HOSTS` | `.with_allowed_hosts(...)` dans le builder |
| Rate limiting | `django-ratelimit` (tiers) | `RateLimiter` natif |
| Sanitisation inputs | — | middleware sanitize natif |
| Secret key générée | manuel | `runique new` génère 32 bytes hex automatiquement |

---

## Vue Admin

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Activation | `admin.site.register(Model)` | macro `admin!{}` + `runique start` |
| List / Create / Edit / Detail / Delete | natif | natif |
| Pagination liste | natif | **manquant** |
| `list_display` | natif | **manquant** |
| Recherche / filtres | natif | **manquant** |
| Templates personnalisables | oui | oui (hiérarchie Tera) |
| Permissions par ressource | natif | stockées, non injectées dans le contexte Tera |
| Création compte admin | `createsuperuser` | `runique create-superuser` |
| Compte admin depuis l'app | non | non (identique) |

---

## Email

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Envoi email | `send_mail()` natif | **manquant** — `lettre` à brancher |
| Templates email | natif | **manquant** |
| Backend SMTP/console | configurable | — |

---

## Internationalisation

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Langues supportées | illimitées (fichiers `.po`) | 9 (`en`, `fr`, `it`, `es`, `de`, `pt`, `ja`, `zh`, `ru`) |
| Détection auto langue | `LocaleMiddleware` | `LANG` / `LC_ALL` env |
| Fallback | oui | oui (`Lang::En`) |
| Traductions framework | `.po`/`.mo` | fichiers JSON (14 sections, compilés dans le binaire) |
| `t("clé")` | `_("...")` | `t("section.clé")` → `Cow<'static, str>` |

---

## Performance & Déploiement

| Aspect | Django | Runique |
|--------|--------|---------|
| Runtime | CPython (GIL) | Tokio async Rust |
| Serveur prod | Gunicorn + Nginx | binaire compilé (Axum/Hyper) |
| Empreinte mémoire | ~50–100 MB | ~5–15 MB |
| Compilation | — | `cargo build --release` |
| Docker | oui | oui |
| Déploiement | fly.io, Heroku, Azure, etc. | idem (binaire statique = plus simple) |

---

## Ce qui manque encore à Runique (résumé)

- Flux auth complet (activation email, reset password)
- Intégration email native
- Upload fichier robuste (validation MIME, resize)
- Pagination admin + `list_display` + filtres
- Permissions runtime dans l'admin
- Équivalent `django-simple-history`
- NoSQL natif (hors scope, brancher `mongodb`)
- `request.path_param()` / `request.query_param()` — actuellement via extractors Axum bruts (voir [roadmap](../../ROADMAP.md#4c-requestpath_param-et-requestquery_param))
