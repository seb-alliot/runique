# Changelog - Runique Framework

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.86/),
et ce projet adhère à [Semantic Versioning](https://semver.org/lang/fr/).

---

## [1.0.86] - 2025-01-15

### Version initiale

#### Ajouté

**Core Framework**
- Structure `RuniqueApp` avec builder pattern
- Configuration via `Settings` (défaut, .env, builder)
- Support des variables d'environnement
- Logging avec `tracing`

**Routing**
- Macro `urlpatterns!` inspirée de Django
- Support du reverse routing
- Nommage des routes avec `name = "..."`
- Fonction `reverse()` et `reverse_with_parameters()`

**Templates**
- Intégration de Tera
- Preprocessing des templates au démarrage
- Balises personnalisées Django-like :
  - `{% static "file" %}` - Fichiers statiques
  - `{% media "file" %}` - Fichiers média
  - `{% csrf %}` - Token CSRF
  - `{% messages %}` - Flash messages
  - `{% link "route" %}` - Reverse routing
- Support de l'héritage de templates
- Auto-injection du contexte (csrf_token, messages, debug, csp_nonce)

**Formulaires**
- Macro `#[runique_form]` pour formulaires personnalisés
- Macro `#[derive(DeriveModelForm)]` pour génération automatique depuis modèles
- Types de champs : CharField, TextField, EmailField, PasswordField, IntegerField, FloatField, BooleanField, DateField, DateTimeField, IPAddressField, URLField, SlugField, JSONField
- Validation avec `require()` et `optional()`
- Méthode `get_value<T>()` type-safe
- Hash Argon2id automatique pour PasswordField
- Sanitisation XSS automatique pour champs texte
- Génération automatique de `validate()`, `to_active_model()` et `save()` pour ModelForm

**ORM & Base de données**
- Intégration SeaORM
- API Django-like avec `impl_objects!` :
  - `Entity::objects.all()`
  - `Entity::objects.filter()`
  - `Entity::objects.exclude()`
  - `Entity::objects.get()`
  - `Entity::objects.count()`
- Support multi-bases :
  - SQLite (défaut)
  - PostgreSQL
  - MySQL / MariaDB
- Configuration automatique depuis `.env`
- Détection automatique du moteur de base de données
- Pool de connexions configurable (défaut: max=20, min=5)
- Masquage des mots de passe dans les logs

**Middleware**
- Middleware de gestion d'erreur avec pages debug détaillées
- Middleware CSRF avec génération de tokens HMAC-SHA256 sécurisés
- Middleware flash messages avec niveaux (success, error, info)
- Middleware sanitisation XSS optionnel
- Middleware CSP (Content Security Policy) avec 3 configurations prédéfinies
- Middleware ALLOWED_HOSTS avec support wildcards
- Middleware d'authentification (`login_required`, `redirect_if_authenticated`)
- Support des middleware personnalisés

**Sécurité**
- Protection CSRF intégrée avec tokens HMAC-SHA256
- Validation ALLOWED_HOSTS avec wildcards (`.example.com`)
- CSP avec génération de nonces cryptographiques
- Headers de sécurité complets (X-Content-Type-Options, X-Frame-Options, etc.)
- Sanitisation XSS automatique optionnelle
- Sessions sécurisées avec `tower-sessions`
- Validation constante des tokens
- Hash Argon2id pour mots de passe
- Mode debug/production

**Fichiers statiques**
- Service automatique des fichiers statiques
- Service automatique des fichiers média
- Filtres Tera {% static "css/main.css %} et {% media "media_name.format %}
- Configuration flexible des chemins

**Gestion d'erreur**
- Pages d'erreur élégantes (404, 500)
- Mode debug avec informations détaillées :
  - Stack trace complète
  - Informations de requête HTTP
  - Source du template avec numéro de ligne
  - Liste des templates disponibles
  - Variables d'environnement
  - Version de Rust
- Templates d'erreur personnalisables
- Fallback HTML en cas d'échec

**Extractors Axum personnalisés**
- `Template` - Extraction et rendu de templates
- `Message` - Gestion des flash messages
- `ExtractForm<T>` - Extraction et validation de formulaires
- Auto-injection du contexte dans les templates

**Macros**
- `urlpatterns!` - Définition de routes Django-like
- `context!` - Création de contextes Tera avec deux syntaxes
- `impl_objects!` - Active l'API Django-like pour les entités
- `#[runique_form]` - Génère Deref/DerefMut et serde(flatten) pour formulaires
- `#[derive(DeriveModelForm)]` - Génère formulaire complet depuis modèle
- `reverse!` et `reverse_with_parameters!` - Reverse routing

**Documentation**
- README complet avec exemples
- Guide de démarrage pas à pas (GETTING_STARTED.md)
- Documentation des templates et balises (TEMPLATES.md)
- Guide de la base de données et ORM (DATABASE.md)
- Guide de configuration (CONFIGURATION.md)
- Guide des formulaires (FORMULAIRE.md)
- Guide CSP (CSP.md)
- Guide de contribution (CONTRIBUTING.md)
- Documentation complète (~89 pages)

#### Technique

**Architecture**
- Modularisation claire du code
- Séparation des responsabilités
- Pattern builder pour la configuration
- Trait extensions pour Tera
- Macro procédurale pour génération de code

**Dépendances**
- `axum` 0.7 - Framework HTTP
- `tokio` 1.43 - Runtime async
- `tower` - Middleware
- `tower-http` - Services HTTP
- `tower-sessions` - Gestion des sessions
- `tera` 1.20 - Moteur de templates
- `sea-orm` 1.1 - ORM (optionnel)
- `serde` / `serde_json` - Sérialisation
- `tracing` - Logging
- `dotenvy` - Variables d'environnement
- `regex` - Preprocessing des templates
- `hmac` / `sha2` - CSRF tokens
- `argon2` - Hash mots de passe
- `chrono` - Gestion du temps

**Features Cargo**
- `default = ["orm"]` - Active ORM par défaut
- `orm` - Support SeaORM
- `sqlite` - Driver SQLite
- `postgres` - Driver PostgreSQL
- `mysql` / `mariadb` - Driver MySQL/MariaDB
- `all-databases` - Tous les drivers

#### Exemples fournis

- Application démo complète avec templates, DB, formulaires
- Tests unitaires (75% coverage)
- Tests d'intégration
- Exemples dans la documentation

#### Connu / Limitations

**Fonctionnalités manquantes :**
- Rate limiting : Le flag existe (`Settings.rate_limiting`) mais l'implémentation n'est pas faite
- CLI pour migrations : Utiliser `sea-orm-cli` directement
- WebSocket natif : Utiliser les layers Axum/Tower
- Admin panel : Non implémenté
- Hot reload : Utiliser `cargo-watch`

**Inconsistances documentation/code résolues :**
- CSP.md : Correction de la structure `CspConfig` (champs plats, pas enum)
- DATABASE.md : Clarification des paramètres de pool hardcodés
- FORMULAIRE.md : Validation que `DeriveModelForm` génère bien `validate()`, `to_active_model()` et `save()`

**Limitations techniques :**
- Variables dynamiques non supportées dans balises personnalisées templates
- Un seul niveau de preprocessing des templates
- SQLite limité à 1 connexion (limitation native)

#### Statistiques du projet

- Lignes de code : ~15,000 LOC
- Documentation : ~89 pages (~150 pages imprimées)
- Tests : 75% coverage
- Nombre de tests : 50+ tests unitaires et intégration
- Modules : 20+ modules

#### Sécurité

**Score Mozilla Observatory : A+ (115/100)**

Headers de sécurité implémentés :
- Content-Security-Policy
- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block
- Referrer-Policy: strict-origin-when-cross-origin
- Permissions-Policy
- Strict-Transport-Security (recommandé en production)

---

## À venir (v1.1.0)

### Planifié

**Fonctionnalités**
- Implémentation réelle du rate limiting
- CLI Runique pour scaffolding et migrations
- Support WebSocket intégré
- Admin panel généré automatiquement
- Hot reload en développement
- Système de cache (Redis, Memcached)
- Support i18n/l10n complet

**Améliorations**
- Pool de connexions entièrement configurable via Settings
- Variables dynamiques dans balises templates
- Multi-niveau preprocessing templates
- Génération automatique de documentation API
- Benchmarks de performance
- Plus de types de champs pour formulaires

**Documentation**
- Guide de déploiement avancé
- Tutoriels vidéo
- Exemples d'applications réelles
- Patterns et best practices

---

## Comparaison des versions

### Django → Runique

| Fonctionnalité | Django | Runique v1.0.86 | Statut |
|----------------|--------|--------------|--------|
| **Routing** | `urls.py` | `urlpatterns!` | Complet |
| **Templates** | Jinja2-like | Tera + balises custom | Complet |
| **ORM** | Django ORM | SeaORM + API Django-like | Complet |
| **Formulaires** | Django Forms | `#[runique_form]` + `DeriveModelForm` | Complet |
| **Admin** | Django Admin | Pas encore | À venir |
| **Auth** | Intégré | Middleware de base | Partiel |
| **Migrations** | `manage.py migrate` | `sea-orm-cli` | Partiel |
| **CSRF** | Middleware | Middleware | Complet |
| **Sessions** | Intégré | Intégré | Complet |
| **Static files** | `collectstatic` | Service automatique | Complet |
| **i18n** | Complet | Pas encore | À venir |
| **Cache** | Multiple backends | Pas encore | À venir |
| **Rate limiting** | Django-ratelimit | Flag seulement | À venir |

---

## Contributions

Merci à tous les contributeurs ! Vos contributions font la différence.

Développeur principal : Itsuki

---

## Liens

- [Documentation](informations)
- [Dépôt GitHub](https://github.com/seb-alliot/runique.git)
- [Issues](https://github.com/seb-alliot/runique/tree/issues)
- [Discussions](https://github.com/seb-alliot/runique/tree/discussions)
- [Changelog](informations/documentation_french/CHANGELOG.md)

---

**Légende**
- Fonctionnalité complète
- Fonctionnalité partielle
- Pas encore implémenté
- En développement

---

[1.0.86]: # Changelog - Runique Framework

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.86/),
and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [1.0.86] - 2025-01-15

### Initial Release

#### Added

**Core Framework**
- `RuniqueApp` structure with builder pattern
- Configuration via `Settings` (default, .env, builder)
- Environment variable support
- Logging with `tracing`

**Routing**
- Django-inspired `urlpatterns!` macro
- Reverse routing support
- Route naming with `name = "..."`
- `reverse()` and `reverse_with_parameters()` functions

**Templates**
- Tera integration
- Template preprocessing at startup
- Django-like custom tags:
  - `{% static "file" %}` - Static files
  - `{% media "file" %}` - Media files
  - `{% csrf %}` - CSRF token
  - `{% messages %}` - Flash messages
  - `{% link "route" %}` - Reverse routing
- Template inheritance support
- Auto-injection of context (csrf_token, messages, debug, csp_nonce)

**Forms**
- `#[runique_form]` macro for custom forms
- `#[derive(DeriveModelForm)]` macro for auto-generation from models
- Field types: CharField, TextField, EmailField, PasswordField, IntegerField, FloatField, BooleanField, DateField, DateTimeField, IPAddressField, URLField, SlugField, JSONField
- Validation with `require()` and `optional()`
- Type-safe `get_value<T>()` method
- Automatic Argon2id hashing for PasswordField
- Automatic XSS sanitization for text fields
- Automatic generation of `validate()`, `to_active_model()` and `save()` for ModelForm

**ORM & Database**
- SeaORM integration
- Django-like API with `impl_objects!`:
  - `Entity::objects.all()`
  - `Entity::objects.filter()`
  - `Entity::objects.exclude()`
  - `Entity::objects.get()`
  - `Entity::objects.count()`
- Multi-database support:
  - SQLite (default)
  - PostgreSQL
  - MySQL / MariaDB
- Automatic configuration from `.env`
- Automatic database engine detection
- Configurable connection pool (default: max=20, min=5)
- Password masking in logs

**Middleware**
- Error handling middleware with detailed debug pages
- CSRF middleware with secure HMAC-SHA256 token generation
- Flash message middleware with levels (success, error, info)
- Optional XSS sanitization middleware
- CSP (Content Security Policy) middleware with 3 predefined configurations
- ALLOWED_HOSTS middleware with wildcard support
- Authentication middleware (`login_required`, `redirect_if_authenticated`)
- Custom middleware support

**Security**
- Built-in CSRF protection with HMAC-SHA256 tokens
- ALLOWED_HOSTS validation with wildcards (`.example.com`)
- CSP with cryptographic nonce generation
- Complete security headers (X-Content-Type-Options, X-Frame-Options, etc.)
- Optional automatic XSS sanitization
- Secure sessions with `tower-sessions`
- Constant token validation
- Argon2id hashing for passwords
- Debug/production modes

**Static Files**
- Automatic static file serving
- Automatic media file serving
- Tera filters `{{ "file" | static }}` and `{{ "file" | media }}`
- Flexible path configuration

**Error Handling**
- Elegant error pages (404, 500)
- Debug mode with detailed information:
  - Complete stack trace
  - HTTP request information
  - Template source with line number
  - List of available templates
  - Environment variables
  - Rust version
- Customizable error templates
- HTML fallback on failure

**Custom Axum Extractors**
- `Template` - Template extraction and rendering
- `Message` - Flash message management
- `ExtractForm<T>` - Form extraction and validation
- Auto-injection of context into templates

**Macros**
- `urlpatterns!` - Django-like route definition
- `context!` - Tera context creation with two syntaxes
- `impl_objects!` - Enable Django-like API for entities
- `#[runique_form]` - Generate Deref/DerefMut and serde(flatten) for forms
- `#[derive(DeriveModelForm)]` - Generate complete form from model
- `reverse!` and `reverse_with_parameters!` - Reverse routing

**Documentation**
- Complete README with examples
- Step-by-step getting started guide (GETTING_STARTED.md)
- Template and tag documentation (TEMPLATES.md)
- Database and ORM guide (DATABASE.md)
- Configuration guide (CONFIGURATION.md)
- Forms guide (FORMULAIRE.md)
- CSP guide (CSP.md)
- Contributing guide (CONTRIBUTING.md)
- Complete documentation (~89 pages)

#### Technical

**Architecture**
- Clear code modularization
- Separation of concerns
- Builder pattern for configuration
- Trait extensions for Tera
- Procedural macro for code generation

**Dependencies**
- `axum` 0.7 - HTTP framework
- `tokio` 1.43 - Async runtime
- `tower` - Middleware
- `tower-http` - HTTP services
- `tower-sessions` - Session management
- `tera` 1.20 - Template engine
- `sea-orm` 1.1 - ORM (optional)
- `serde` / `serde_json` - Serialization
- `tracing` - Logging
- `dotenvy` - Environment variables
- `regex` - Template preprocessing
- `hmac` / `sha2` - CSRF tokens
- `argon2` - Password hashing
- `chrono` - Time management

**Cargo Features**
- `default = ["orm"]` - Enable ORM by default
- `orm` - SeaORM support
- `sqlite` - SQLite driver
- `postgres` - PostgreSQL driver
- `mysql` / `mariadb` - MySQL/MariaDB driver
- `all-databases` - All drivers

#### Examples Provided

- Complete demo application with templates, DB, forms
- Unit tests (75% coverage)
- Integration tests
- Examples in documentation

#### Known Issues / Limitations

**Missing Features:**
- Rate limiting: Flag exists (`Settings.rate_limiting`) but implementation not done
- CLI for migrations: Use `sea-orm-cli` directly
- Native WebSocket: Use Axum/Tower layers
- Admin panel: Not implemented
- Hot reload: Use `cargo-watch`

**Resolved Documentation/Code Inconsistencies:**
- CSP.md: Corrected `CspConfig` structure (flat fields, not enum)
- DATABASE.md: Clarified hardcoded pool parameters
- FORMULAIRE.md: Validated that `DeriveModelForm` generates `validate()`, `to_active_model()` and `save()`

**Technical Limitations:**
- Dynamic variables not supported in custom template tags
- Single level of template preprocessing
- SQLite limited to 1 connection (native limitation)

#### Project Statistics

- Lines of code: ~15,000 LOC
- Documentation: ~89 pages (~150 printed pages)
- Tests: 75% coverage
- Number of tests: 50+ unit and integration tests
- Modules: 20+ modules

#### Security

**Mozilla Observatory Score: A+ (115/100)**

Implemented security headers:
- Content-Security-Policy
- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block
- Referrer-Policy: strict-origin-when-cross-origin
- Permissions-Policy
- Strict-Transport-Security (recommended in production)

---

## Coming Soon (v1.1.0)

### Planned

**Features**
- Actual rate limiting implementation
- Runique CLI for scaffolding and migrations
- Integrated WebSocket support
- Auto-generated admin panel
- Hot reload in development
- Cache system (Redis, Memcached)
- Complete i18n/l10n support

**Improvements**
- Fully configurable connection pool via Settings
- Dynamic variables in template tags
- Multi-level template preprocessing
- Automatic API documentation generation
- Performance benchmarks
- More form field types

**Documentation**
- Advanced deployment guide
- Video tutorials
- Real application examples
- Patterns and best practices

---

## Version Comparison

### Django → Runique

| Feature | Django | Runique v1.0.86 | Status |
|---------|--------|--------------|--------|
| **Routing** | `urls.py` | `urlpatterns!` | Complete |
| **Templates** | Jinja2-like | Tera + custom tags | Complete |
| **ORM** | Django ORM | SeaORM + Django-like API | Complete |
| **Forms** | Django Forms | `#[runique_form]` + `DeriveModelForm` | Complete |
| **Admin** | Django Admin | Not yet | Coming |
| **Auth** | Built-in | Basic middleware | Partial |
| **Migrations** | `manage.py migrate` | `sea-orm-cli` | Partial |
| **CSRF** | Middleware | Middleware | Complete |
| **Sessions** | Built-in | Built-in | Complete |
| **Static files** | `collectstatic` | Automatic serving | Complete |
| **i18n** | Complete | Not yet | Coming |
| **Cache** | Multiple backends | Not yet | Coming |
| **Rate limiting** | Django-ratelimit | Flag only | Coming |

---

## Contributors

Thanks to all contributors! Your contributions make the difference.

Lead Developer: Itsuki

---

## Links

- [Documentation](informations)
- [GitHub Repository](https://github.com/seb-alliot/runique)
- [Issues](https://github.com/seb-alliot/runique/tree/issues)
- [Discussions](https://github.com/seb-alliot/runique/tree/discussions)
- [Changelog](hinformations/documentation_french/CHANGELOG.md)

---

**Legend**
- Complete feature
- Partial feature
- Not yet implemented
- In development

---

[1.0.86]: https://github.com/seb-alliot/runique/releases

*Documentation created with ❤️ by Claude for Itsuki*
