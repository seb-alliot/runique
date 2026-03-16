# Roadmap Runique

## 1. Sûreté API

**Status :** 🟡 En cours

### 1.a. CSRF secure-by-default

- **Principe directeur :** respect forcé du contrat d'utilisation `methode http -> prisme -> handler` pour stabiliser la sécurité CSRF.
- **Règle 1 (mutations) :** toutes les routes `POST`/`PUT`/`PATCH`/`DELETE` passent obligatoirement par Prisme.
- **Règle 2 (lecture body) :** Prisme reste l'unique lecteur du body (pas de relecture middleware, pas de buffering global).
- **Règle 3 (source token) :** `form-data` / `x-www-form-urlencoded` => token CSRF dans les champs ; `json/ajax` => token CSRF dans le header.
- **Règle 4 (GET) :** `GET` reste classique (normalisation query/headers), sans vérification CSRF.
- **Application progressive :** mode compat (warning) puis mode strict (refus des routes mutantes hors contrat).
- **Effet attendu :** réduction des failles liées au non-respect du contrat, simplification des handlers, stabilité sécurité renforcée.

### 1.b. Middleware CSP

- 🟢 `'unsafe-inline'` retiré de `script_src` et `style_src` par défaut
- 🟢 `use_nonce: true` par défaut — nonce vide filtré
- 🟢 HSTS ajouté : `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- 🟢 Builder CSP — sous-builder `CspConfig` avec directives, toggles et presets (livré 1.1.47)

### 1.c. Robustesse runtime

- 🟢 `utils/middleware/csrf.rs` — `mask_csrf_token()` → `Result` (fix appliqué, plus de crash DDoS)
- 🟢 Comparaisons CSRF constant-time via `subtle::ct_eq` (`csrf_gate.rs`, `middleware/security/csrf.rs`, `forms/fields/hidden.rs`)
- 🟢 Cookies session : `HttpOnly: true`, `SameSite: Strict`
- 🟢 `allowed_hosts.rs` : bypass DEBUG supprimé → interrupteur `RUNIQUE_ENABLE_HOST_VALIDATION`
- 🟢 `cli_admin.rs` : validation du chemin provider avant `Command::new` (anti-RCE)
- 🟢 `SECRET_KEY` aléatoire générée à `runique new`
- 🟢 `utils/trad/switch_lang.rs` — `RwLock` → `AtomicU8` (livré 1.1.46)
- 🔴 `utils/middleware/csrf.rs:57,74` — `SystemTime::UNIX_EPOCH.unwrap()` (risque quasi nul, à surveiller)
- 🔴 Réduire `panic!/unwrap/expect` sur les chemins runtime
- 🔴 Propager des erreurs typées (`Result`) sur les points critiques (middleware, daemon, CLI, i18n)

### 1.d. Nouveaux outils sécurité

- 🟢 `RateLimiter` — rate limiting par IP, configurable par handler (`middleware/rate_limit.rs`)
- 🟢 `LoginGuard` — protection brute-force par username (`middleware/auth/login_guard.rs`)
- 🔴 Tracing sécurité structuré (voir 4.a)

### 1.e. Sécurité / permissions admin

- 🔴 Vérification runtime des rôles (requête DB par requête admin)
- 🔴 Contrat `is_staff` / `is_superuser` / rôles custom à clarifier
- 🔴 Tests d'autorisation par opération CRUD

---

## 2. Bugs muets (silent failures)

**Status :** 🔴 À inventorier

Bugs qui ne crashent pas mais produisent un comportement incorrect sans avertissement.
→ À identifier au fil des audits et revues de code. Aucun répertorié à ce jour.

---

## 3. Stabilité & Fallback

**Status :** 🟡 En cours

### 3.a. Tests et couverture

- **Tests exhaustifs** : 🟡 82.83% fonctions (objectif 85% minimum).
- **Audit sécurité** : 🟢 Fait — corrections appliquées (branche i18n, 2026-03-13/14)

### 3.b. Validation au boot (fail-fast)

- Valider toute la config critique avant le démarrage serveur (security, middleware, db, password, admin).
- Refuser le boot en production si incohérence ou valeur manquante.
- Autoriser des fallbacks contrôlés en dev/test avec warning explicite.

### 3.c. Gouvernance globale de la config API

- Définir un ordre de priorité unique pour toute l'API :
    1. Overrides explicites de démarrage
    2. Configuration applicative (`RuniqueConfig`)
    3. Variables d'environnement
    4. Valeurs par défaut framework
- Documenter un point d'entrée clair de l'initialisation globale (pas de logique implicite cachée).
- Éviter les doubles sources de vérité entre config runtime et valeurs par défaut internes.
- Ajouter des tests d'intégration sur la résolution de config (priorités + erreurs de validation).

> **Status contrat développeur :** 🟠 En stand-by — choix de `is_valid()` en cours de réflexion.

---

## 4. Features

**Status :** 🟡 En cours

### 4.a. I18n et Tracing

**I18n :** 🟢 Fini — intégration runtime (config/session/request), `set_lang()` depuis l'env ou la requête.

**Tracing d'erreur :**

- Optionnel
- `debug = false` : tracing off
- `debug = true` : tracing on (console + page debug)

**Tracing sécurité :** 🔴 À faire

- Événements structurés sur les points critiques (à brancher sur le tracing existant) :
  - CSRF rejeté → `tracing::warn!`
  - Host bloqué → `tracing::warn!`
  - Login échoué / compte verrouillé (`LoginGuard`) → `tracing::warn!`
  - Rate limit dépassé (`RateLimiter`) → `tracing::warn!`
- Contexte riche : IP, username (si disponible), route, timestamp

### 4.b. Formulaires — `#[derive(DeriveModelForm)]`

**Status :** 🟡 À évaluer — potentiellement supprimé

- **Étape 1 — Check viabilité :** inventorier les usages réels (code + docs + exemples) et confirmer que le couple `model!(...)` + `#[form(...)]` couvre tous les cas actuels.
- **Étape 2 — Mesure des pertes occasionnées :** lister précisément ce qui serait perdu (ergonomie, rétrocompatibilité, snippets existants, onboarding) et estimer l'impact migration.
- **Étape 3 — Plan de transition :** préparer une migration douce (dépréciation documentée, alias temporaire éventuel, guide de remplacement).
- **Étape 4 — Validation technique :** vérifier compile/tests/docs après remplacement des usages critiques.
- **Étape 5 — Décision finale :** `GO` suppression ou `NO-GO` maintien selon coût réel vs bénéfice architecture.

### 4.c. Configuration du pool Database

**Status :** 🔴 À faire

- Permettre la configuration du pool via `.env`
- Découpage `.env` envisagé :
  - `.env` — config basique dev + redirections
  - `.env.conf` — pool, lang, timezone
  - `.env.security` — CSP (interrupteur + directives), rate limite

### 4.d. Système de slots atomique (extensibilité builder)

- Permettre à des plugins/crates externes de revendiquer un slot middleware stable à l'init du module.
- Fondation : `AtomicU16::fetch_add` — sans lock, sans risque de poison.
- À implémenter si/quand un écosystème de plugins est envisagé.

---

## 5. Vue Admin

**Status :** 🟡 En cours (beta)

### Court terme

- 🔴 **list_display** : colonnes affichées dans la liste, configurables par ressource dans `admin!{}`
  - `ColumnFilter::All` → inféré depuis les clés du premier enregistrement
  - `ColumnFilter::Include(...)` / `Exclude(...)` → filtrage explicite
  - Injecter `columns: Vec<String>` dans le contexte Tera
  - Template list : `id="row-{{ entry.id }}"` + `id="cell-{{ entry.id }}-{{ col }}"` pour ciblage JS
- 🔴 **Pagination** : `page` + `per_page` depuis query params, passer au `list_fn`
- 🔴 **Ordering** : tri par colonne cliquable (query param `?order=col&dir=asc`)
- 🔴 **search_fields** : recherche texte côté backend, route ou query param `?q=...`
- 🔴 **JS assets** : champ `js: ["path/to/file.js"]` dans `admin!{}` → `js_files: Vec<String>` dans `AdminResource` → injecté dans bloc `extra_js` du template
- 🔴 **Permissions runtime** : vérification des rôles par ressource à chaque requête admin (requête DB, option sécurisée)
- 🟢 **i18n des templates admin** : système i18n branché sur tous les templates admin

### Moyen terme

- 🔴 **History / log admin** : table `admin_log` (user_id, resource_key, object_id, action, timestamp, changes jsonb) — hooks via signaux SeaORM
- 🔴 **Bulk actions** : suppression en lot, actions custom déclarées par ressource
- 🔴 **readonly_fields** : champs non-éditables affichés en lecture seule dans les formulaires
- 🔴 **date_hierarchy** : navigation par date (année > mois > jour) en haut de la liste
- 🔴 **list_filter** : filtres latéraux par valeur de colonne
- 🔴 **Toggle boolean** : `PATCH /admin/{resource}/{id}/toggle/{field}` — checkbox cliquable dans la liste pour les champs booléens (`is_active`, `is_staff`, etc.)

### Hors scope v1 (futur)

- **Inlines** : formulaires imbriqués pour relations SeaORM (`has_many`)
- **autocomplete_fields** : widget AJAX pour ForeignKey
- **list_editable** : édition inline dans la liste (hors boolean)
- **date_hierarchy avancé** : navigation drill-down avec agrégations DB

---

## 6. TLS natif + Proxy intégré

**Status :** 🔴 Planifié (après vue admin)

### Objectif

Rendre Runique autonome en production — binaire compilé + TLS natif, sans Nginx ni reverse proxy externe.

### Composants

- 🔴 **TLS natif** : `rustls` + `axum-server` — certificats PEM configurables via builder
- 🔴 **Let's Encrypt** : renouvellement automatique (`instant-acme`) via background task Tokio (`tokio::time::interval`)
- 🔴 **Compression** : `CompressionLayer` de `tower-http` (déjà disponible, à exposer dans le builder)
- 🔴 **Cache statiques** : headers `Cache-Control` configurables via builder
- 🔴 **Feature flags** : `features = ["tls", "proxy"]` — opt-in, binaire minimal par défaut

### Ce qui reste hors scope

- Load balancing multi-instances
- Rate limiting réseau bas niveau (couvert côté applicatif par `RateLimiter`)

### Impact déploiement

`docker run` autonome — zéro dépendance externe, production-ready avec un seul conteneur.

---

## 7. Publication crates.io

**Status :** 🟢 En continu — version actuelle **1.1.47** publiée

### Processus de release

Chaque évolution notable → nouvelle version publiée sur crates.io avec changelog.

- Incrément **patch** (1.1.x) : fix, i18n, sûreté, corrections silencieuses
- Incrément **minor** (1.x.0) : nouvelle feature stable, nouveau middleware, extension admin
- Incrément **major** (x.0.0) : rupture d'API publique

### Objectifs qualité avant chaque release

- 🟡 85% couverture minimum (`bin/` exclu) — actuellement 82.83%
- 🟡 Doctests `ignore`/`no_run` → exemples réels (i18n, migration, forms, builder couverts)
- 🔴 Docs complètes — models, forms, macros procédurales

> `bin/` exclu du calcul de couverture (CLI non couvrable proprement).
> Cible réaliste : **85-88%** après couverture des modules HTTP via helpers Axum.
