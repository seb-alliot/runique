# Changelog - Rusti Framework

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet adhère à [Semantic Versioning](https://semver.org/lang/fr/).

---

## [1.0.0] - 2025-01-15

### Version initiale

#### Ajouté

**Core Framework**
- Structure `RustiApp` avec builder pattern
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
- Macro `#[rusti_form]` pour formulaires personnalisés
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
- Filtres Tera `{{ "file" | static }}` et `{{ "file" | media }}`
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
- `#[rusti_form]` - Génère Deref/DerefMut et serde(flatten) pour formulaires
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
- CLI Rusti pour scaffolding et migrations
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

### Django → Rusti

| Fonctionnalité | Django | Rusti v1.0.0 | Statut |
|----------------|--------|--------------|--------|
| **Routing** | `urls.py` | `urlpatterns!` | Complet |
| **Templates** | Jinja2-like | Tera + balises custom | Complet |
| **ORM** | Django ORM | SeaORM + API Django-like | Complet |
| **Formulaires** | Django Forms | `#[rusti_form]` + `DeriveModelForm` | Complet |
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

- [Documentation](https://docs.rs/rusti)
- [Dépôt GitHub](https://github.com/votre-repo/rusti)
- [Issues](https://github.com/votre-repo/rusti/issues)
- [Discussions](https://github.com/votre-repo/rusti/discussions)
- [Changelog](https://github.com/votre-repo/rusti/blob/main/CHANGELOG.md)

---

**Légende**
- Fonctionnalité complète
- Fonctionnalité partielle
- Pas encore implémenté
- En développement

---

[1.0.0]: https://github.com/votre-repo/rusti/releases/tag/v1.0.0