# Roadmap Runique

## 1. I18n et Tracing

**Status :** 🟡 En cours

### 1.a. I18n (Internationalisation)

**Status :** Fini

- Intégration runtime (config/session/request) — `set_lang()` depuis l’env ou la requête

### 1.b. Tracing d’erreur

- optionnel
- `debug = false` : tracing off.
- `debug = true` : tracing on (console + page debug).

---

## 2. Vue Admin

**Status :** 🟡 En cours

### À implémenter — court terme

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

### À implémenter — moyen terme

- 🔴 **History / log admin** : table `admin_log` (user_id, resource_key, object_id, action, timestamp, changes jsonb) — hooks via signaux SeaORM
- 🔴 **Bulk actions** : suppression en lot, actions custom déclarées par ressource
- 🔴 **readonly_fields** : champs non-éditables affichés en lecture seule dans les formulaires
- 🔴 **date_hierarchy** : navigation par date (année > mois > jour) en haut de la liste
- 🔴 **list_filter** : filtres latéraux par valeur de colonne
- 🔴 **Toggle boolean** : `PATCH /admin/{resource}/{id}/toggle/{field}` — checkbox cliquable dans la liste pour les champs booléens (`is_active`, `is_staff`, etc.)

### Hors scope v1 (futur)

- **Inlines** : formulaires imbriqués pour relations SeaORM (`has_many`) — nécessite refonte du modèle form + JS add/remove + transactions groupées
- **autocomplete_fields** : widget AJAX pour ForeignKey — nécessite route `/admin/{resource}/search` + JS Select2-style
- **list_editable** : édition inline dans la liste (hors boolean — voir toggle ci-dessous)
- **date_hierarchy avancé** : navigation drill-down avec agrégations DB

### Personnalisation templates

- 🔴 Documentation des clés Tera disponibles par vue (list, create, edit, detail, delete)
- 🔴 **i18n des templates admin** : brancher le système i18n existant sur les templates admin (labels, messages flash, erreurs, boutons) — section `admin` déjà présente dans les 8 langues

### Sécurité / permissions admin

- 🔴 Vérification runtime des rôles (requête DB par requête admin)
- 🔴 Contrat `is_staff` / `is_superuser` / rôles custom à clarifier
- 🔴 Tests d’autorisation par opération CRUD

---

## 3. Sécurité middleware et stabilité

**Status :** 🟡 En cours

### 3.a. Middleware CSP

- Peaufiner la configuration pour la rendre plus simple et lisible.
- Réduire les directives permissives par défaut (`unsafe-inline`, `unsafe-eval`).
- Harmoniser la gestion des nonces.

### 3.b. CSRF secure-by-default

- **Principe directeur :** respect forcé du contrat d’utilisation `methode http -> prisme -> handler` pour stabiliser la sécurité CSRF.
- **Règle 1 (mutations) :** toutes les routes `POST`/`PUT`/`PATCH`/`DELETE` passent obligatoirement par Prisme.
- **Règle 2 (lecture body) :** Prisme reste l’unique lecteur du body (pas de relecture middleware, pas de buffering global).
- **Règle 3 (source token) :** `form-data` / `x-www-form-urlencoded` => token CSRF dans les champs ; `json/ajax` => token CSRF dans le header.
- **Règle 4 (GET) :** `GET` reste classique (normalisation query/headers), sans vérification CSRF.
- **Application progressive :** mode compat (warning) puis mode strict (refus des routes mutantes hors contrat).
- **Effet attendu :** réduction des failles liées au non-respect du contrat, simplification des handlers, stabilité sécurité renforcée.

### 3.c. Stabilité et couverture

- **Tests exhaustifs** : 🟡 76.66% fonctions (objectif 85% minimum).
- **Audit sécurité** : 🔴 À faire (identifier et corriger les failles).

### 3.d. Robustesse runtime

- Réduire `panic!/unwrap/expect` sur les chemins runtime.
- Propager des erreurs typées (`Result`) sur les points critiques (middleware, daemon, CLI, i18n).

---

## 4. Formulaires — `#[derive(DeriveModelForm)]`

**Status :** 🟡 À évaluer — potentiellement supprimé

- **Étape 1 — Check viabilité :** inventorier les usages réels (code + docs + exemples) et confirmer que le couple `model!(...)` + `#[form(...)]` couvre tous les cas actuels.
- **Étape 2 — Mesure des pertes occasionnées :** lister précisément ce qui serait perdu (ergonomie, rétrocompatibilité, snippets existants, onboarding) et estimer l’impact migration.
- **Étape 3 — Plan de transition :** préparer une migration douce (dépréciation documentée, alias temporaire éventuel, guide de remplacement).
- **Étape 4 — Validation technique :** vérifier compile/tests/docs après remplacement des usages critiques.
- **Étape 5 — Décision finale :** `GO` suppression ou `NO-GO` maintien selon coût réel vs bénéfice architecture.

---

## 5. Publication crates.io

**Status :** 🔴 À faire

### Étapes avant publication

- 85% couverture minimum (`bin/` exclu) : 🟡 76.66% actuellement.
- Remplacer doctests `ignore`/`no_run` par exemples réels : 🟡 En cours (i18n, migration, forms, sanitizer, aliases, builder couverts).
- Docs complètes (models, forms, macros procédurales, etc) : 🔴 À faire.
- Publish crates.io avec une réel documentation complete et lisible : 🔴 À faire.

> Note : `bin/` est exclu du calcul de couverture (CLI non couvrable proprement).
> Cible réaliste : **85-88%** après couverture des modules HTTP via helpers Axum.

---

## 6. Gouvernance globale de la config API

**Status :** 🟡 En cours

### 6.a. Résolution unifiée de configuration

- Définir un ordre de priorité unique pour toute l’API :
    1. Overrides explicites de démarrage
    2. Configuration applicative (`RuniqueConfig`)
    3. Variables d’environnement
    4. Valeurs par défaut framework

### 6.b. Validation au boot (fail-fast)

- Valider toute la config critique avant le démarrage serveur (security, middleware, db, password, admin).
- Refuser le boot en production si incohérence ou valeur manquante.
- Autoriser des fallbacks contrôlés en dev/test avec warning explicite.

### 6.c. Contrat développeur

**Status :** 🟠 En Stand by => Choix de is_valid() en cour de reflexion pour un meilleur usage d'utilisation

- Documenter un point d’entrée clair de l’initialisation globale (pas de logique implicite cachée).
- Éviter les doubles sources de vérité entre config runtime et valeurs par défaut internes.
- Ajouter des tests d’intégration sur la résolution de config (priorités + erreurs de validation).

## 7. Configuration du pool Database

**Status :** 🔴 À faire

- Permettre la configuration du pool via `.env`
- Découpage `.env` envisagé :
  - `.env` — config basique dev + redirections
  - `.env.conf` — pool, lang, timezone
  - `.env.security` — CSP (interrupteur + directives), rate limite
