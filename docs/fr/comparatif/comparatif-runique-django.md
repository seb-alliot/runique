# Comparatif Runique vs Django

## CLI

| Commande | Django | Runique |
|----------|--------|---------|
| Créer un projet | `django-admin startproject nom` | `runique new nom` |
| Créer une app | `python manage.py startapp nom` | — |
| Migrations (générer) | `python manage.py makemigrations` | `runique makemigrations` |
| Migrations (appliquer) | `python manage.py migrate` | `runique migration up` |
| Migrations (annuler) | `python manage.py migrate app 0001` | `runique migration down --files ...` |
| Statut migrations | — | `runique migration status` |
| Créer superuser | `python manage.py createsuperuser` | `runique create-superuser` |
| Démarrer | `python manage.py runserver` | `cargo run` — `runique start` pour (re)générer le panel admin |

---

## Routing

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Déclaration des routes | `urls.py` avec `path()` | `url.rs` avec macro `urlpatterns!{}` |
| Routes dynamiques | `path('users/<int:id>/', view)` | `"/users/{id}"` dans `urlpatterns!` |
| Namespaces | `app_name` + `include()` | `Router::new().nest("/prefix", ...)` |
| Reverse URL | `{% url "nom_vue" %}` natif | `{% link "nom_vue" %}` → fonction Tera custom |
| Paramètre de chemin typé | `kwargs['id']` (toujours str dans Django) | `request.get_path_as::<i32>("id")` |
| Paramètre de chemin brut | `kwargs['id']` | `request.get_path("id")` |
| Query param unique | `request.GET.get('key')` | `request.get_query("key")` |
| Query string complète | `request.GET` | `request.query::<MyStruct>()` (désérialise vers un struct `Deserialize`) |
| Headers HTTP | `request.META['HTTP_X_FOO']` | `request.headers.get("x-foo")` |

---

## Vues / Handlers

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Vue fonction | `def ma_vue(request)` | `async fn ma_vue(...)` |
| Vue classe | `class MaVue(View)` | — |
| Accès session | `request.session` | `request.session` via `context::template::Request` |
| Accès DB | `Model.objects.get(...)` | `Model::objects.get(...)` ou builders SeaORM |
| Rendu template | `render(request, "template.html", ctx)` | `request.render("template.html")` |
| Redirect | `redirect("nom_url")` | `Redirect::to("/url")` ou `reverse(&engine, "nom")` |
| Messages flash | `messages.success(request, "...")` | macros `success!`, `error!`, `info!`, `warning!` |
| Connexions secondaires | `DATABASES['mongo']` | `engine.extension::<mongodb::Client>()` (TypeMap multi-type) |

---

## Formulaires

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Définition | `class MonForm(ModelForm)` | `#[form]` struct ou `RuniqueForm` manuel |
| Validation | `form.is_valid()` | `form.is_valid().await` |
| Champs disponibles | CharField, EmailField, FileField, etc. | TextField, EmailField, PasswordField, HiddenField, ChoiceField, NumericField, BooleanField, FileField, DateField, TimeField, DateTimeField, DurationField, PhoneField |
| Rendu HTML | `{{ form.as_p }}` | `{% form.nom_form %}` (entier) ou `{% form.nom_form.champ %}` |
| CSRF intégré | automatique | automatique — injecté avant le premier champ |
| Sauvegarde | `form.save()` | `form.save(&db).await` (si `#[form]`) |
| Accès aux données | `form.cleaned_data['clé']` | `form.cleaned_string("clé")`, `form.cleaned_i32(...)`, etc. |
| Validation async | non | oui (accès DB dans `clean()`) |
| Validation croisée | `clean()` | `clean()` async |
| Fichiers | `FileField` | `FileField` multipart natif avec validation type/taille |
| Sanitisation HTML | non (à la main) | `sanitize_rich` / `sanitize_strict` appliquées aux champs `richtext` |

---

## Templates

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Moteur | Django Template Language | Tera (syntaxe Jinja2 / Django-like) |
| Héritage | `{% extends %}` / `{% block %}` | idem |
| Fichiers statiques | `{% load static %}` + `{% static "file" %}` | `{% static "file" %}` natif |
| Fichiers media | `{{ MEDIA_URL }}file` | `{% media "file" %}` natif (variables supportées) |
| URL reverse | `{% url "nom" %}` | `{% link "nom" %}` |
| CSRF | `{% csrf_token %}` | `{% csrf %}` |
| Messages | `{% for m in messages %}` | `{% messages %}` |
| Internationalisation | `{% trans "..." %}` | `{{ t("section.clé") }}` / `{{ tf("...", ["var"]) }}` |

---

## ORM / Base de données

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| ORM | Django ORM natif | SeaORM (Rust async) |
| Définition modèle | `class User(models.Model)` | macro `model!{}` (types v1 SQL ou v2 sémantiques) |
| Migrations auto | oui (détection changements) | `runique makemigrations` |
| QuerySet chaînable | `User.objects.filter(...).order_by(...)` | `User::objects.filter(...).order_by(...)` |
| `.get()` strict | lève `MultipleObjectsReturned` | `.one()` — retourne `Err` si plusieurs lignes |
| Tri aléatoire | `order_by('?')` | `.order_by_random()` |
| Tri par expression | — | `.order_by_expr(expr, order)` |
| Relations | ForeignKey, ManyToMany, OneToOne | Relations SeaORM standard |
| Transactions | `with transaction.atomic()` | `db.transaction(...)` |
| Multi-moteur SQL | oui | PostgreSQL, MySQL, SQLite |
| Connexions secondaires | `DATABASES` multi-entrées | `.with_custom_db::<T>()` × N types (TypeMap) |
| Extension table framework | — | `extend!{}` — `ALTER TABLE ADD COLUMN` sur tables `eihwaz_*` |

---

## Authentification

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Login / Logout | `authenticate()` + `login()` | `auth_login(...)`, `logout()` |
| Vérif authentification | `request.user.is_authenticated` | `is_authenticated(&session).await` |
| Utilisateur courant | `request.user` | `CurrentUser` (injecté via middleware) |
| Protection route | `@login_required` | `if !is_authenticated(...).await { redirect }` |
| Sessions | natif | tower-sessions (MemoryStore + DB fallback) |
| Protection brute force | `django-axes` (tiers) | `LoginGuard` natif (lockout auto) |
| Hashage mot de passe | PBKDF2 / Argon2 | Argon2 par défaut |
| Reset password | natif | natif — template email personnalisable via `.email_template("...")` |
| Déconnexion forcée | oui | `RuniqueSessionStore::invalidate_all(user_id)` |

---

## Sécurité

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| CSRF | natif | natif (comparaison constant-time) |
| CSP | `django-csp` (tiers) | natif (`use_nonce: true` par défaut) |
| HSTS | `SECURE_HSTS_SECONDS` | natif |
| SameSite cookies | configurable | `Strict` par défaut |
| HttpOnly cookies | par défaut | toujours `true` |
| Rate limiting | `django-ratelimit` (tiers) | `RateLimiter` natif |
| Open Redirect | — | natif — toutes les réponses 3xx vérifiées (slot 25) |
| CORS | `django-cors-headers` (tiers) | natif via `.with_cors(...)` |
| Permissions-Policy | — | natif — preset sécurisé par défaut |
| Trusted Proxies / XFF | `SECURE_PROXY_SSL_HEADER` (partiel) | natif — validation chaîne XFF complète, preset RFC 1918 |
| Secret key | manuel | générée auto à `runique new` |

---

## Panel Admin

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Activation | `admin.site.register(Model)` | macro `admin!{}` |
| CRUD complet | natif | natif |
| Pagination liste | natif | `.page_size(n)` (liste + historique) |
| `list_display` | natif | `list_display: [["col", "Libellé"], ...]` |
| Résolution FK en liste | — | 3ème élément : `["fk_id", "Libellé", "table.colonne"]` |
| Recherche / filtres | natif | `list_filter` + recherche plein-texte SQL |
| Actions de groupe | `actions` | `group_action` — bool (2 éléments) ou enum (3 éléments, valeur exacte) |
| Création multiple | — | `bulk_create: champ` — split par virgule, insère N enregistrements |
| Édition en masse | — | bulk edit natif sur sélection multi-entrées |
| Relations M2M | `filter_horizontal` / `ManyRelatedField` | `m2m: [...]` — table de jonction, diff automatique |
| Routes admin custom | `get_urls()` | `.extra_routes(vec![...])` |
| Templates custom | oui | oui (hiérarchie Tera) |
| Permissions | par ressource | RBAC dynamique (Groupes / Droits scopés) |
| Historique modifications | `django-simple-history` (tiers) | historique natif (créé/modifié/supprimé) avec diff de champs |
| Configuration builtins | — | bloc `configure {}` dans `admin!{}` |

---

## Email

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Envoi email | `send_mail()` natif | `Email::new().send()` natif |
| Templates email | natif | templates Tera via `.template("emails/mon.html")` |
| Backend SMTP | configurable | configuration via `.env` |
| Backend dev (console) | `EMAIL_BACKEND = 'console'` | `EMAIL_BACKEND=console` dans `.env` |

---

## Internationalisation

| Fonctionnalité | Django | Runique |
|----------------|--------|---------|
| Langues supportées | illimitées | 9 langues par défaut (JSON compilé) |
| Fallback | oui | oui (`Lang::En`) |
| Traduction | `_("...")` | `t("section.clé")` / `tf("...", ["var"])` |

---

## Performance & Déploiement

| Aspect | Django | Runique |
|--------|--------|---------|
| Runtime | CPython (interprété) | Tokio async Rust (compilé) |
| Empreinte mémoire | ~50–100 MB | ~5–15 MB |
| Compilation | — | binaire statique unique |
| ACME / TLS auto | `certbot` (externe) | natif via feature `acme` |

---

## Ce qu'il manque encore (comparé à Django)

- **Redimensionnement automatique d'images** : resize/cropping côté serveur non natif.
- **Vues CRUD publiques génériques** : pas d'équivalent aux `DetailView`, `ListView`, `CreateView` de Django pour les vues publiques — prévu via `crud!{}` (en développement). Le panel admin couvre le CRUD backoffice via `admin!{}`.
- **Signals / hooks modèles** : `before_save`, `after_save`, `before_delete`, `after_delete` — infrastructure posée, générateur en cours de branchement.
- **Management commands** : pas d'équivalent à `manage.py custom_command` — les opérations one-shot passent par `src/bin/`.
- **Test client intégré** : pas de client HTTP de test natif — utiliser `reqwest` ou `axum::test`.
- **Fixtures** : pas de `loaddata`/`dumpdata` — les seeds sont des fonctions Rust.
- **Admin inline** : pas d'édition d'objets liés directement dans le formulaire parent.
- **i18n complète** : `t()`/`tf()` disponibles, mais pas de pluralisation ni de traduction des templates Tera.
- **Sitemap / RSS** : non natif.
- **Authentification tiers** : OAuth / OIDC structuré (Google, Microsoft, Apple, LDAP, SAML) mais flow non implémenté — stub uniquement. JWT et API key auth absents.
- **Observabilité sécurité** : CSP `report-uri`/`report-to` absent — les violations ne sont pas collectées. Audit log des connexions (réussies/échouées/lockouts) non tracé en DB.
