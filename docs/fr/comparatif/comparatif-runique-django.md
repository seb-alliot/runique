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
| Récupérer un paramètre de chemin | `kwargs['id']` | `form.cleaned_*` ou `request.path_param("id")` |
| Récupérer un query param | `request.GET.get('key')` | `form.cleaned_*` ou `request.from_url("key")` |

---

## Vues / Handlers

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Vue fonction | `def ma_vue(request)` | `async fn ma_vue(...)` |
| Vue classe | `class MaVue(View)` | — |
| Accès session | `request.session` | `request.session` via `context::template::Request` (ou `Session` extractor axum directement) |
| Accès DB | `Model.objects.get(...)` | `Model::objects.get(...)` (via `impl_objects!`) ou sea-orm query builders |
| Rendu template | `render(request, "template.html", ctx)` | `request.render("template.html")` — contexte déjà dans `request.context` |
| Redirect | `redirect("nom_url")` | `Redirect::to("/url")` ou `reverse(&engine, "nom")` / `reverse_with_parameters(...)` (prelude) |
| Messages flash | `messages.success(request, "...")` | `success!(message => "...")` — macros `success!`, `error!`, `info!`, `warning!` (prelude) |

---

## Formulaires

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Définition | `class MonForm(forms.Form)` / `class MonForm(ModelForm)` | `#[form]` struct (équivalent `ModelForm`) ou `RuniqueForm` manuel |
| Validation | `form.is_valid()` | `form.is_valid().await` |
| Champs disponibles | CharField, EmailField, etc. | TextField, EmailField, PasswordField, HiddenField, ChoiceField, NumericField, BooleanField, FileField, DateTimeField, DurationField |
| Rendu HTML | `{{ form.as_p }}` | `{% form.nom_form %}` (formulaire entier) ou `{% form.nom_form.champ %}` (champ individuel) |
| CSRF intégré | automatique | automatique — injecté avant le premier champ |
| Sauvegarde | `form.save()` | `form.save(&db).await` (si `#[form]`) |
| Accès aux données | `form.cleaned_data['clé']` | `form.cleaned_*("clé")` (ex: `string`, `i32`, `bool`, `uuid`, etc.) |
| Validation async | non | oui (accès DB direct dans `clean()`) |
| Formulaires fichier | `FileField` | Multipart natif avec validation dimensions/format |

---

## Templates

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Moteur | Django Template Language | Tera (syntaxe Jinja2 / Django-like) |
| Héritage | `{% extends %}` / `{% block %}` | idem Tera |
| Fichiers statiques | `{% load static %}` `{% static "file" %}` | `{% static "file" %}` natif |
| Fichiers media | `{{ MEDIA_URL }}file` | `{% media "file" %}` natif |
| URL reverse | `{% url "nom" %}` | `{% link "nom" %}` |
| CSRF | `{% csrf_token %}` | `{% csrf %}` |
| Messages | `{% for m in messages %}` | `{% messages %}` |
| Internationalisation | `{% trans "..." %}` | `{{ t("section.clé") }}` ou `{{ tf("...", ["var"]) }}` |

---

## ORM / Base de données

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| ORM | Django ORM natif | sea-orm (Rust async) |
| Définition modèle | `class User(models.Model)` | struct Rust annotée + `model!{}` macro |
| Migrations auto | oui (détection changements) | `runique makemigrations` |
| QuerySet chaînable | `User.objects.filter(...).order_by(...)` | `User::objects.filter(...).order_by(...)` (via `impl_objects!`) |
| Relations | ForeignKey, ManyToMany, OneToOne | Relations sea-orm standard |
| Transactions | `with transaction.atomic()` | `db.transaction(...)` |
| Multi-DB | oui | PostgreSQL, MySQL, SQLite |
| NoSQL | via packages tiers | via crates tierces (ex. `mongodb`) |
| Re-export | — | `runique::sea_orm` + `sea_query` intégrés |

---

## Authentification

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Login / Logout | `authenticate()` + `login()` | `auth_login(...)`, `logout()` |
| Vérif authentification | `request.user.is_authenticated` | `is_authenticated(&session).await` |
| Utilisateur courant | `request.user` | `CurrentUser` (injecté via middleware) |
| Protection route | `@login_required` | pattern `if !is_authenticated(&session).await { ... }` |
| Sessions | natif | tower-sessions (DB backend) |
| Protection brute force | `django-axes` (tiers) | `LoginGuard` natif (lockout auto) |
| Hashage mot de passe | PBKDF2 / argon2 | argon2 par défaut, multi-algos supportés |
| Activation compte email | natif (`auth`) | intégré au lien de réinitialisation/création password |
| Reset password | natif | `handle_forgot_password` + `handle_password_reset` natifs |
| Déconnexion forcée | oui | `RuniqueSessionStore::invalidate_all(user_id)` |

---

## Sécurité

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| CSRF | natif | natif (constant-time validation) |
| CSP | `django-csp` (tiers) | natif (`use_nonce: true` par défaut) |
| HSTS | `SECURE_HSTS_SECONDS` | natif |
| SameSite cookies | configurable | `Strict` par défaut |
| HttpOnly cookies | par défaut | toujours `true` |
| Rate limiting | `django-ratelimit` (tiers) | `RateLimiter` natif |
| Sanitisation inputs | — | middleware `sanitize` natif |
| Secret key | manuel | générée auto à l''install |

---

## Vue Admin

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Activation | `admin.site.register(Model)` | macro `admin!{}` |
| CRUD complet | natif | natif |
| Pagination liste | natif | `.pagination(n)` dans `DisplayConfig` |
| `list_display` | natif | `.columns_include()` / `.columns_exclude()` |
| Recherche / filtres | natif | `.list_filter()` + champ de recherche automatique |
| Templates custom | oui | oui (hiérarchie Tera) |
| Permissions | par ressource | RBAC dynamique (Groupes / Permissions) |

---

## Email

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Envoi email | `send_mail()` natif | `utils::Email::new().send()` natif |
| Templates email | natif | Tera templates supportés via `html(body)` |
| Backend SMTP | configurable | configuration via `.env` |

---

## Internationalisation

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Langues supportées | illimitées | 9 langues par défaut (JSON compilé) |
| Fallback | oui | oui (`Lang::En`) |
| `t("clé")` | `_("...")` | `t("section.clé")` |

---

## Performance & Déploiement

| Aspect | Django | Runique |
|--------|--------|---------|
| Runtime | CPython (interprété) | Tokio async Rust (compilé) |
| Empreinte mémoire | ~50–100 MB | ~5–15 MB |
| Compilation | — | binaire statique unique |

---

## Ce qu'il manque encore (comparé à Django)

Runique se rapproche de la complétude fonctionnelle de Django, mais quelques éléments restent en chantier :

- **File upload amélioré** : Le redimensionnement automatique d''image nativement côté serveur (resize/cropping).
- **Équivalent à `django-simple-history`** : Un système d''audit log intégré pour tracer l''historique de chaque modification en base de données.
- **NoSQL Natif** (Toujours hors périmètre principal, mais intégration MongoDB simplifiée prévue).
- `request.path_param()` / `request.query_param()` — actuellement via extractors Axum bruts (voir [roadmap](../../ROADMAP.md#4c-requestpath_param-et-requestquery_param))
