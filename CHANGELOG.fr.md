🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.fr.md)

# Journal des modifications

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

---

## [1.1.47] - 2026-03-14

### Sécurité

* **CSRF :** suppression du vecteur de panique `expect()` sur token malformé — remplacé par un fallback gracieux `unwrap_or_else` dans `csrf.rs`.
* **CSRF :** `HeaderMap::contains_key("X-CSRF-Token")` confirmé insensible à la casse — aucun contournement possible via la casse des en-têtes.
* **Sûreté des verrous :** `GLOBAL_LANG` (`RwLock<Lang>`) remplacé par `AtomicU8` — empoisonnement de verrou impossible, plus aucun `unwrap()` nécessaire.
* **Sûreté des verrous :** acquisitions de `url_registry` et `PENDING_URLS` utilisent désormais `unwrap_or_else(|e| e.into_inner())` — survie à un mutex empoisonné en cas de panique dans un thread.

### Ajouté

* **Rate limiter (`RateLimiter`) :** middleware à fenêtre glissante par IP. Limite de requêtes et fenêtre de temps configurables. Retourne HTTP 429 en cas de dépassement.
* **Login guard (`LoginGuard`) :** protection anti-bruteforce par nom d'utilisateur. Nombre de tentatives et durée de blocage configurables. Complémentaire au `RateLimiter` (IP vs. nom d'utilisateur).
* **Nettoyage périodique (`spawn_cleanup`) :** `RateLimiter` et `LoginGuard` exposent `spawn_cleanup(period)` — démarre une tâche Tokio en arrière-plan qui purge les entrées expirées à intervalle configurable, sur le même modèle que `CleaningMemoryStore`.
* **Template 429 :** template Tera dédié (`errors/429.html`) embarqué dans le binaire, rendu par `error_handler_middleware` sur `TOO_MANY_REQUESTS`. Inclut un fallback HTML inline si le rendu Tera échoue.
* **i18n — clés 429 :** `html.429_title` et `html.429_text` ajoutés aux 9 fichiers de traduction (fr, en, de, es, it, pt, ja, zh, ru).
* **CLI — langue :** la langue de l'application est désormais configurable via la variable d'environnement `RUNIQUE_LANG`. `RuniqueConfig::from_env()` la lit et l'applique automatiquement.
* **Prelude :** `dotenvy` ré-exporté dans `runique::prelude` (section CONFIGURATION) et à la racine de la crate.

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