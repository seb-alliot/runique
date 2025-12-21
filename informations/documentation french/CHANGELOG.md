# Changelog - Rusti Framework

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet adhère à [Semantic Versioning](https://semver.org/lang/fr/).

---

## [Non publié]

### À venir
- [ ] CLI pour scaffolding de projets (`rusti new mon-app`)
- [ ] Support WebSocket
- [ ] Middleware d'authentification intégré
- [ ] Support GraphQL
- [ ] Générateur de documentation API
- [ ] Benchmarks de performance

---

## [0.1.0] - 2025-01-XX

### 🎉 Version initiale

#### ✨ Ajouté

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
- Auto-injection du contexte (csrf_token, messages, debug)

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
- Pool de connexions configurable
- Masquage des mots de passe dans les logs

**Middleware**
- Middleware de gestion d'erreur avec pages debug détaillées
- Middleware CSRF avec génération de tokens sécurisés
- Middleware flash messages avec niveaux (success, error, info)
- Middleware par défaut (erreur + timeout)
- Support des middleware personnalisés

**Sécurité**
- Protection CSRF intégrée
- Tokens HMAC-SHA256
- Sessions sécurisées avec `tower-sessions`
- Validation constante des tokens
- Mode debug/production

**Fichiers statiques**
- Service automatique des fichiers statiques
- Service automatique des fichiers média
- Filtres Tera `{{ "file" | static }}`
- Configuration flexible des chemins

**Gestion d'erreur**
- Pages d'erreur élégantes (404, 500)
- Mode debug avec informations détaillées :
  - Stack trace complète
  - Informations de requête HTTP
  - Source du template
  - Liste des templates disponibles
  - Variables d'environnement
- Templates d'erreur personnalisables
- Fallback HTML en cas d'échec

**Extractors Axum personnalisés**
- `Template` - Extraction et rendu de templates
- `Message` - Gestion des flash messages
- Auto-injection du contexte dans les templates

**Documentation**
- README complet avec exemples
- Guide de démarrage pas à pas
- Documentation des templates et balises
- Guide de la base de données et ORM
- Guide de configuration
- Documentation de l'API
- Exemples de code complets

#### 🔧 Technique

**Architecture**
- Modularisation claire du code
- Séparation des responsabilités
- Pattern builder pour la configuration
- Trait extensions pour Tera

**Dépendances**
- `axum` - Framework HTTP
- `tokio` - Runtime async
- `tower` - Middleware
- `tower-http` - Services HTTP
- `tower-sessions` - Gestion des sessions
- `tera` - Moteur de templates
- `sea-orm` - ORM (optionnel)
- `serde` / `serde_json` - Sérialisation
- `tracing` - Logging
- `dotenvy` - Variables d'environnement
- `regex` - Preprocessing des templates
- `hmac` / `sha2` - CSRF tokens
- `chrono` - Gestion du temps

**Features Cargo**
- `orm` (défaut) - Support SeaORM
- `sqlite` - Driver SQLite
- `postgres` - Driver PostgreSQL
- `mysql` / `mariadb` - Driver MySQL/MariaDB
- `all-databases` - Tous les drivers

#### 📝 Exemples fournis

- `demo-app` - Application complète avec templates, DB, formulaires
- Tests unitaires et d'intégration
- Exemples dans la documentation

#### 🐛 Connu / Limitations

- Variables dans les balises personnalisées non supportées
- Un seul niveau de preprocessing des templates
- Rate limiting non intégré (utiliser `tower-governor`)
- Pas de support WebSocket natif
- Migrations manuelles (via `sea-orm-cli`)

---

## Comparaison des versions

### Django → Rusti

| Fonctionnalité | Django | Rusti v0.1.0 | Statut |
|----------------|--------|--------------|--------|
| **Routing** | ✅ `urls.py` | ✅ `urlpatterns!` | Complet |
| **Templates** | ✅ Jinja2-like | ✅ Tera + balises custom | Complet |
| **ORM** | ✅ Django ORM | ✅ SeaORM + API Django-like | Complet |
| **Formulaires** | ✅ Django Forms | ❌ Pas encore | À venir |
| **Admin** | ✅ Django Admin | ❌ Pas encore | À venir |
| **Auth** | ✅ Intégré | ❌ Manuel | À venir |
| **Migrations** | ✅ `manage.py migrate` | ⚠️ `sea-orm-cli` | Partiel |
| **CSRF** | ✅ Middleware | ✅ Middleware | Complet |
| **Sessions** | ✅ Intégré | ✅ Intégré | Complet |
| **Static files** | ✅ `collectstatic` | ✅ Service automatique | Complet |
| **i18n** | ✅ Complet | ❌ Pas encore | À venir |
| **Cache** | ✅ Multiple backends | ❌ Pas encore | À venir |

---

## Contributions

Merci à tous les contributeurs ! Vos contributions font la différence.

---

## Liens

- [Documentation](https://docs.rs/rusti)
- [Dépôt GitHub](https://github.com/votre-repo/rusti)
- [Issues](https://github.com/votre-repo/rusti/issues)
- [Changelog](https://github.com/votre-repo/rusti/blob/main/CHANGELOG.md)

---

**Légende**
- ✅ Fonctionnalité complète
- ⚠️ Fonctionnalité partielle
- ❌ Pas encore implémenté
- 🔧 En développement

---

[Non publié]: https://github.com/votre-repo/rusti/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/votre-repo/rusti/releases/tag/v0.1.0
