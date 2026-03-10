## [1.1.45] - 2026-03-10

### Fixed
- **Docs** : `admin!{}` — suppression des champs `template_*` (surcharge de templates via builder uniquement)
- **Docs** : `.with_proto_state()` → `.with_state()` dans `admin/setup.md` (méthode inexistante en code)
- **Docs** : `mon_theme/` → `my_theme/` dans `admin/template/surcharge/surcharge.md` (EN — noms FR non traduits)
- **Docs** : labels de navigation inversés dans `admin/template/surcharge/` et `admin/template/clef/` (FR)
- **Docs** : syntaxe `urlpatterns!` corrigée dans `architecture/` (FR+EN) — `get "/path" handler` → `"/path" => view!{ handler }, name = "name"`
- **Docs** : `src/forms.rs` → `src/entities/` + `src/formulaire/` dans `architecture/` (FR+EN)
- **Docs** : avertissement migrations — `runique migration up/down/status` contournait le suivi SeaORM — restructuré en sections "recommandé" vs "avancé"
- **Docs** : `model!` — syntaxe `model!(...)` → `model! { ... }` (accolades, sans point-virgule)
- **Docs** : `impl_objects!` — présenté comme déclaration manuelle → corrigé : généré automatiquement par le daemon ; ajout note "sucre syntaxique pur, SQL identique au SeaORM natif"
- **Docs** : `use demo_app::models::users` → `use demo_app::entities::users` (6 occurrences dans orm/ et routing/)
- **Clippy** : suppressions d'emprunts inutiles `&` sur retours `&'static str` dans `admin_main.rs` et `admin_router.rs`
- **Clippy** : `.to_string().into()` → `.to_string()` (conversions inutiles dans `demo-app/admins/admin_panel.rs`)

### Added
- **Docs** : section "Démarrer un nouveau projet" dans `architecture/` (FR+EN)
- **Docs** : sections 12–15 (Model, Auth, Sessions, Env) ajoutées aux hubs README (FR+EN)
- **Docs** : architecture EN réécrite pour correspondre à la version FR

---

## [1.1.44]

- **Fix** => Cli ok

## [1.1.42]

- **Fix sécurity csrf** : Delete csrf on method Get


## [1.1.38] 2026-03-06

### Fixed

- **Memory leak** : `MemoryStore` (tower-sessions) never deleted expired sessions — memory grew unboundedly under load
  (~1 369 MB after 5 min at 500 concurrent). Replaced by `CleaningMemoryStore` with automatic periodic cleanup.
  Peak memory under same load: **79 MB** (-94%). See [benchmark.md](benchmark.md).

### Added

- `CleaningMemoryStore` : in-process session store with periodic cleanup (60s timer, configurable via `RUNIQUE_SESSION_CLEANUP_SECS`).
- Two-tier watermark system: low watermark (128 MB) triggers async background purge of expired anonymous sessions;
  high watermark (256 MB) triggers synchronous emergency purge + 503 refusal if store remains saturated.
  Configurable via `RUNIQUE_SESSION_LOW_WATERMARK` / `RUNIQUE_SESSION_HIGH_WATERMARK`.
- Session protection: sessions containing `user_id` (authenticated) or `session_active` (future timestamp set by
  `protect_session()`) are never sacrificed under memory pressure.
- `protect_session(&session, duration_secs)` / `unprotect_session(&session)` helpers for high-value anonymous sessions
  (shopping carts, multi-step forms).
- Builder methods: `with_session_memory_limit(low, high)` and `with_session_cleanup_interval(secs)`.
- Alert log when a session record exceeds 50 KB (file or image accidentally stored in session).

### Changed
- Les sessions anonymes expirent désormais après 5 minutes d'inactivité (configurable).
- Lorsqu'un utilisateur s'authentifie, la session est automatiquement prolongée à 24h (configurable).
- Middleware slot 55 : upgrade dynamique du TTL de session après login, sans impact sur la logique CSRF ou les handlers applicatifs.

### Dev
- Ajout des méthodes `with_session_duration` et `with_anonymous_session_duration` dans le builder pour personnaliser les TTL.

## [1.1.35] - 2026-03-04

### Changed
- Form system stabilized with multiple internal improvements.
- Builder updated with a new, more flexible middleware system.

### Security
- CSRF protection is now transparently enforced in all forms by default.

### Upcoming
- Initial work and design phase for a basic admin view.


## [1.1.35] - 2026-03-04

### Modifié
- Stabilisation du système de formulaires avec plusieurs améliorations internes.
- Mise à jour du builder avec un nouveau système de middleware plus flexible.

### Sécurité
- La protection CSRF est désormais imposée de manière transparente sur tous les formulaires.

### À venir
- Début de réflexion et de conception pour une vue d’administration basique.
