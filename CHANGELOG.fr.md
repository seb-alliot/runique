🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Journal des modifications

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

---

## [1.1.47] - 2026-03-15

### Rupture

* **CSP — variables d'env supprimées :** toutes les variables `RUNIQUE_POLICY_CSP_*`, `RUNIQUE_ENABLE_CSP`, `RUNIQUE_ENABLE_HEADER_SECURITY`, `ENFORCE_HTTPS`, `RUNIQUE_POLICY_CSP_STRICT_NONCE` sont supprimées. La CSP est désormais configurée exclusivement via le builder.
* **CSP — désactivée par défaut :** `MiddlewareStaging::from_config()` n'active plus la CSP automatiquement. Elle doit être activée explicitement via `.with_csp(...)`.
* **`SecurityPolicy::from_env()` supprimée :** remplacée par `SecurityPolicy::default()`. Tous les appels mis à jour.
* **`builder.rs` :** import inutilisé `SecurityPolicy` supprimé.

### Sécurité

* **Middleware CSRF :** les requêtes mutantes (POST/PUT/DELETE/PATCH) sans header `X-CSRF-Token` et sans `Content-Type` de formulaire (`application/x-www-form-urlencoded` / `multipart/form-data`) sont désormais rejetées avec 403. Elles passaient silencieusement auparavant.
* **Masquage du token CSRF (protection BREACH) :** `extractor.rs` (`build_with_data`) et `template.rs` (`form()`) injectent maintenant le token **masqué** (XOR + base64) dans les champs cachés des formulaires, au lieu du hex HMAC-SHA256 brut. L'AJAX lit ainsi la valeur correcte pour le header `X-CSRF-Token`.
* **`csrf_gate.rs` :** le token soumis via formulaire est désormais **démasqué** avant la comparaison en temps constant contre le token de session brut — le cycle masque/démasque est cohérent de bout en bout.
* **CSRF :** suppression du vecteur de panique `expect()` sur token malformé — remplacé par un fallback gracieux `unwrap_or_else` dans `csrf.rs`.
* **CSRF :** `HeaderMap::contains_key("X-CSRF-Token")` confirmé insensible à la casse — aucun contournement possible via la casse des en-têtes.
* **Sûreté des verrous :** `GLOBAL_LANG` (`RwLock<Lang>`) remplacé par `AtomicU8` — empoisonnement de verrou impossible, plus aucun `unwrap()` nécessaire.
* **Sûreté des verrous :** acquisitions de `url_registry` et `PENDING_URLS` utilisent désormais `unwrap_or_else(|e| e.into_inner())` — survie à un mutex empoisonné en cas de panique dans un thread.

### Corrigé

* **Bug d'accolade CSRF (`csrf.rs`) :** un `} else {` mal placé faisait appartenir la branche `else` à `if requires_csrf` au lieu de `if has_header`, renvoyant "CSRF token required" sur chaque requête GET (toutes les vues cassées). Restructuré pour corriger la portée.

### Ajouté

* **API builder CSP :** nouveau pattern closure — `.middleware(|m| m.with_csp(|c| c.méthode()))`.
* **Struct `CspConfig` :** sous-builder autonome avec contrôle complet des directives : `scripts()`, `styles()`, `images()`, `fonts()`, `connect()`, `objects()`, `media()`, `frames()`, `frame_ancestors()`, `base_uri()`, `form_action()`, `default_src()`.
* **Toggles `CspConfig` :** `.with_nonce(bool)`, `.with_header_security(bool)`, `.with_upgrade_insecure(bool)`.
* **Presets `CspConfig` :** `.policy(SecurityPolicy::strict())`, `.policy(SecurityPolicy::permissive())`.
* **Accesseurs `CspConfig` :** `.get_policy() -> &SecurityPolicy` et `.header_security_enabled() -> bool` (utilisés dans les tests).
* **`MiddlewareConfig` :** nouveau champ `enable_header_security: bool` — contrôle l'activation de `security_headers_middleware` (HSTS, X-Frame-Options, COEP, COOP, CORP) en complément de la CSP.
* **Rate limiter (`RateLimiter`) :** middleware à fenêtre glissante par IP. Limite de requêtes et fenêtre de temps configurables. Retourne HTTP 429 en cas de dépassement.
* **Login guard (`LoginGuard`) :** protection anti-bruteforce par nom d'utilisateur. Nombre de tentatives et durée de blocage configurables. Complémentaire au `RateLimiter` (IP vs. nom d'utilisateur).
* **Nettoyage périodique (`spawn_cleanup`) :** `RateLimiter` et `LoginGuard` exposent `spawn_cleanup(period)` — démarre une tâche Tokio en arrière-plan qui purge les entrées expirées à intervalle configurable, sur le même modèle que `CleaningMemoryStore`.
* **Template 429 :** template Tera dédié (`errors/429.html`) embarqué dans le binaire, rendu par `error_handler_middleware` sur `TOO_MANY_REQUESTS`. Inclut un fallback HTML inline si le rendu Tera échoue.
* **i18n — clés 429 :** `html.429_title` et `html.429_text` ajoutés aux 9 fichiers de traduction (fr, en, de, es, it, pt, ja, zh, ru).
* **CLI — langue :** la langue de l'application est désormais configurable via la variable d'environnement `RUNIQUE_LANG`. `RuniqueConfig::from_env()` la lit et l'applique automatiquement.
* **Prelude :** `dotenvy` ré-exporté dans `runique::prelude` (section CONFIGURATION) et à la racine de la crate.
* **`runique/static/js/color_picker.js` :** nouveau fichier JS statique. Utilise les attributs `data-color-picker` / `data-color-input` / `data-color-text` pour la synchronisation du sélecteur de couleur sans JS inline. Compatible CSP, idempotent sur plusieurs champs couleur par page.

### Modifié

* **`engine/core.rs` :** `SecurityPolicy::from_env()` → `SecurityPolicy::default()`.
* **`MiddlewareStaging::apply_to_router()` :** branche sur `enable_header_security` pour choisir entre `csp_middleware` (CSP seule) et `security_headers_middleware` (CSP + tous les headers de sécurité).
* **`base_color.html` :** le `<script>` inline (sync du sélecteur de couleur) remplacé par `color_picker.js` externe chargé via `<script src defer>`. Aucun nonce nécessaire — les templates de champs sont rendus sans contexte de requête, donc `csp_nonce` n'était jamais disponible.
* **`demo-app/main.rs` :** `upgrade-insecure-requests` est désormais conditionnel : activé uniquement en release (`cfg!(not(debug_assertions))`). Empêche Chrome d'upgrader HTTP→HTTPS en développement localhost.

### Templates

* **Admin — `style=` inline supprimés :** `create.html` (`max-width:60%` → `card card-form`), `dashboard.html` (`grid-column: 1/-1` → `card-full-width`, `text-decoration:none` supprimé), `delete.html` (`display:inline` → `form-inline`), `edit.html` (`max-width:60%` → `card card-form`), `login.html` (`margin-bottom:1rem` supprimé), `admin_base.html` burger mobile (`display:none` → `hidden`).
* **`admin/composant/edit.html` :** le `<script>` inline (prévisualisation image) porte désormais `nonce="{{ csp_nonce }}"`.

### Docs

* **`derive_form/README.md` :** réécriture complète — tableau des types de champs, types de PK, toutes les options, syntaxe FK, exemple blog complet (User/Category/Post/Comment), `impl_objects!` avec toutes les méthodes de requête, paramètres `#[form(...)]`.
* **`doc-tests/macro_db/model_complete.md` :** réécrit avec la macro `model!` et `impl_objects!`.
* **`docs/fr/middleware/csp/` + `docs/en/middleware/csp/` :** réécriture complète de `csp.md`, `directives.md`, `nonce.md`, `headers.md`, `profils.md` / `profiles.md` — variables d'env supprimées, exemples builder ajoutés, tableaux complets directives/toggles/presets.
* **`docs/fr/env/securite/` + `docs/en/env/security/` :** section CSP supprimée, remplacée par une note renvoyant vers la doc builder.
* **`docs/fr/middleware/hosts-cache/` + `docs/en/` :** ligne `RUNIQUE_ENABLE_CSP` supprimée.

### Tests

* **`tests/middleware/test_csp.rs` :** tous les accès directs aux champs (`csp.policy.*`, `csp.enable_header_security`) remplacés par les accesseurs. Tests `from_env()` supprimés et remplacés par des tests `CspConfig` builder. Ajout des tests HTTP middleware : `csp_middleware`, `csp_report_only_middleware`, `security_headers_middleware` (HSTS, nonce, X-Frame-Options), `https_redirect_middleware` (redirection 308, bypass `x-forwarded-proto: https`).
* **`tests/formulaire/test_csrf_gate.rs` :** `test_csrf_gate_token_valide_retourne_none` mis à jour pour utiliser un token hex 64 chars valide + `mask_csrf_token()` — conforme au nouveau contrat token masqué.
* **`tests/middleware/test_csrf_integration.rs` :** `test_csrf_post_sans_header_passe` → `test_csrf_post_sans_header_sans_content_type_retourne_403` (attend 403) ; idem pour la variante DELETE. Ajout des tests AJAX : POST/DELETE JSON avec token valide (roundtrip session réelle), sans token → 403, token invalide → 403, `X-Requested-With` seul → 403, token volé d'une autre session → 403.
* **`tests/middleware/test_cleaning_store.rs` :** ajout des tests watermark et protection sessions — `purge_anonymous_expired` (low watermark), sessions protégées (`user_id`, `session_active`) survivent à la passe 1, store saturé (sessions vivantes protégées) → refus.
* **`tests/context/test_template_request.rs` :** nouveau fichier — extraction `TplRequest` depuis `FromRequestParts`, `is_get/post/put/delete`, `render` (succès et erreur), `insert`, `render_with`, extraction sans engine → 500.
* **`tests/errors/test_runique_error.rs` :** ajout des tests `log` (toutes variantes), `into_response` (codes HTTP), `from_tera_error`, `with_request` / `with_request_helper` (filtrage headers sensibles), constructeur `database`, `From<BuildError>`.

---

## [1.1.46] - 2026-03-13

### Ajouté

* **Système i18n :** internationalisation intégrée au framework. 8 langues : `en` (défaut), `fr`, `de`, `es`, `it`, `pt`, `ja`, `zh`. 14 sections par langue : `forms`, `csrf`, `error`, `build`, `middleware`, `admin`, `html`, `debug`, `flash`, `log`, `cli`, `daemon`, `macro`, `parser`.
* **`t(key)` :** macro de traduction retournant `Cow<'static, str>`. Fallback automatique vers `Lang::En` pour toute clé manquante — aucune panique possible.
* **`switch_lang.rs` :** stockage de la langue active via `AtomicU8` — sans verrou, sans `unwrap()`.
* **`RUNIQUE_LANG` :** variable d'environnement pour configurer la langue au démarrage. Lue par `RuniqueConfig::from_env()`.

---

## [1.1.45] - 2026-03-10

### Corrigé

* **Docs :** `admin!{}` — suppression des champs `template_*` (la surcharge des templates est désormais gérée uniquement via le builder).
* **Docs :** `.with_proto_state()` → `.with_state()` dans `admin/setup.md` (méthode inexistante dans le code).
* **Docs :** `mon_theme/` → `my_theme/` dans `admin/template/surcharge/surcharge.md` (EN — noms FR non traduits).
* **Docs :** labels de navigation inversés dans `admin/template/surcharge/` et `admin/template/clef/` (FR).
* **Docs :** correction de la syntaxe `urlpatterns!` dans `architecture/` (FR+EN) :
  `get "/path" handler` → `"/path" => view!{ handler }, name = "name"`.
* **Docs :** `src/forms.rs` → `src/entities/` + `src/formulaire/` dans `architecture/` (FR+EN).
* **Docs :** avertissement sur les migrations — `runique migration up/down/status` contournait le suivi de SeaORM. La documentation a été restructurée en sections **« recommandé »** et **« avancé »**.
* **Docs :** correction de la syntaxe `model!` : `model!(...)` → `model! { ... }` (accolades, sans point-virgule).
* **Docs :** `impl_objects!` précédemment présenté comme une déclaration manuelle → précisé comme **généré automatiquement par le daemon**. Ajout d’une note : *« simple sucre syntaxique, SQL identique à SeaORM natif »*.
* **Docs :** `use demo_app::models::users` → `use demo_app::entities::users` (6 occurrences dans `orm/` et `routing/`).
* **Clippy :** suppression d’emprunts `&` inutiles sur les retours `&'static str` dans `admin_main.rs` et `admin_router.rs`.
* **Clippy :** `.to_string().into()` → `.to_string()` (conversions inutiles dans `demo-app/admins/admin_panel.rs`).

### Ajouté

* **Docs :** section **« Démarrer un nouveau projet »** ajoutée dans `architecture/` (FR+EN).
* **Docs :** sections **12–15** (Model, Auth, Sessions, Env) ajoutées aux hubs README (FR+EN).
* **Docs :** documentation de l’architecture EN entièrement réécrite pour correspondre à la version FR.

---

## [1.1.44]

### Corrigé

* CLI fonctionnelle.

---

## [1.1.42]

### Sécurité

* **CSRF :** suppression du token CSRF sur les requêtes `GET`.

---

## [1.1.38] - 2026-03-06

### Corrigé

* **Fuite mémoire :** `MemoryStore` (tower-sessions) ne supprimait jamais les sessions expirées, ce qui provoquait une croissance mémoire illimitée sous charge
  (~1 369 MB après 5 minutes avec 500 utilisateurs concurrents).
  Remplacé par `CleaningMemoryStore` avec nettoyage périodique automatique.

  Pic mémoire sous la même charge : **79 MB** (**-94 %**).
  Voir `benchmark.md`.

### Ajouté

* `CleaningMemoryStore` : stockage de session en mémoire avec nettoyage périodique (timer 60s, configurable via `RUNIQUE_SESSION_CLEANUP_SECS`).
* **Système de watermark à deux niveaux :**

  * **Watermark bas (128 MB)** : purge asynchrone en arrière-plan des sessions anonymes expirées.
  * **Watermark haut (256 MB)** : purge d’urgence synchrone + refus **503** si le store reste saturé.
    Configurable via `RUNIQUE_SESSION_LOW_WATERMARK` et `RUNIQUE_SESSION_HIGH_WATERMARK`.
* **Protection des sessions :** les sessions contenant `user_id` (authentifiées) ou `session_active` (timestamp futur défini par `protect_session()`) ne sont jamais sacrifiées sous pression mémoire.
* Helpers :

  * `protect_session(&session, duration_secs)`
  * `unprotect_session(&session)`
    pour les sessions anonymes à forte valeur (paniers, formulaires multi-étapes).
* Méthodes du builder :

  * `with_session_memory_limit(low, high)`
  * `with_session_cleanup_interval(secs)`
* Log d’alerte lorsqu’un enregistrement de session dépasse **50 KB** (fichier ou image stocké accidentellement dans la session).

### Modifié

* Les sessions anonymes expirent désormais après **5 minutes d’inactivité** (configurable).
* Lorsqu’un utilisateur s’authentifie, la durée de session est automatiquement prolongée à **24 heures** (configurable).
* **Middleware slot 55 :** mise à niveau dynamique du TTL de session après connexion, sans impact sur la logique CSRF ni sur les handlers applicatifs.

### Dev

* Ajout des méthodes du builder :

  * `with_session_duration`
  * `with_anonymous_session_duration`
    pour personnaliser les TTL de session.

---

## [1.1.35] - 2026-03-04

### Modifié

* Stabilisation du système de formulaires avec plusieurs améliorations internes.
* Mise à jour du builder avec un nouveau système de middleware plus flexible.

### Sécurité

* La protection CSRF est désormais appliquée automatiquement à tous les formulaires.

### À venir

* Début de la phase de réflexion et de conception pour une vue d’administration basique.

---