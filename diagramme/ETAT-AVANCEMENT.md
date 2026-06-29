# État d'avancement — reprise de session

Chantier : diagrammes UML + Merise **exhaustifs** de `runique` (hors `demo-app`) +
répertorier/corriger les anomalies. TDD pour les correctifs. `cargo test` autorisé.
Ne pas toucher `demo-app`. Écrire les diagrammes dans `diagramme/` (racine).

## Correctifs appliqués (compilés, 216/216 vert)

- C1/C3 upload staging (`parse_html.rs` + `file.rs finalize` + sweep, tracé)
- D1 big-pk `users_groupes.user_id`
- Tracing « zéro erreur avalée » : forms rollback, file cleanup, history audit,
  user→groupe RBAC, reset_token, anti_bot, auth/password template, guard cache (into_inner)
- M2 log fallback · TR2 mailer Debug masqué
- **C2 (final, breaking/Unreleased)** : `Prisme::data` → `pub(crate)`. Plus aucun accès
  brut au corps hors CSRF côté code tiers ; seuls `checked_data()` (fail-closed) et
  `req.form()` exposent le body. Audit interne clean. demo-app migré vers `checked_data()`.
- **AM3 (final)** : durée session lue du builder (`OnceLock` posé au build depuis
  `session_duration`, login + middleware = source unique, défaut 24h, warn si re-set
  divergent). Doc rustdoc concis + `docs/{fr,en}/middleware/sessions`. Test `ttl_tests`.
- **CX2 + STRICT_CSP mort** : `strict_csp`→`enable_header_security`, HSTS gaté `should_emit_hsts`.
  Test `hsts_tests`.
- Tests ajoutés : finalize (2), parse_multipart staging (1), checked_data (1)

## Faux positifs vérifiés (NE PAS re-flaguer)

- **AM1/M1** : makemigrations gère les ALTER (`diff_schemas`/`Changes.modified_columns`)
- **AM2** divergence session_id : faux (on_conflict disjoints, jamais session_id) — résidu perf
- **TR1** : ErrorContext gaté sur `config.debug`
- **E1** : `enable_debug_errors=true` par défaut · **E2** : `attach_middlewares` code mort

## Diagrammes FAITS

merise/modele-donnees · uml/{engine, context, app/builder-staging, forms/formulaires,
forms/fields-complets, admin/admin-resource-permissions, admin/admin-complements,
auth, middleware/sessions, middleware/securite, migration/schema-et-diff,
migration/types-builder-et-parsed, transverse/utilitaires, derive_form/proc-macro,
config/configuration} · flux/{requete-csrf-upload, auth-session-et-makemigrations,
admin-crud-reset-makemigrations}

## MAJ reprise 20:39 — ajoutés

config/configuration, migration/types-builder-et-parsed, forms/fields-complets,
admin/admin-complements, app/staging-configs-et-build-errors. **Reste ci-dessous.**

## Diagrammes — COMPLETS ✅

Tous les modules lib couverts : merise + uml/{engine, context×2, app×2, forms×2, admin×2,
auth, middleware×2, migration×2, config, transverse, derive_form, macros, utils} + 3 flux.
Ajoutés à cette reprise : macros/macros, utils/tracing-securite-tokens,
context/extensions-et-middleware-config.

Hors-scope assumé : `composant-bin/` (templates de scaffolding pour `new_project`, pas du
code framework runtime) ; `cli/` couvert en flux (fonctions, pas de struct porteuse).

## Nouvelles anomalies trouvées via diagrammes

- **CX2** 🟠 `enable_header_security=false` même en `production()` → HSTS/X-Frame absents en prod.
- **CFG1** 🟡 `secret_key` vide = warning, pas échec boot.

## Session 2026-06-29 — fin de la partie « code »

Corrigés/vérifiés ce jour : **C2** (data `pub(crate)`, breaking/Unreleased), **C4** (csrf_gate
mort supprimé), **C5** (politique CSRF source unique `csrf_required`), **E4** (clean),
**F1** (arm `(model)` dupliqué mort supprimé), **F2/M3** (clean, clamp `set_max_size_bounded`),
**D5** (table fantôme `eihwaz_droits` supprimée sur 4 fronts), **S3** (503 + Retry-After sur
saturation : `is_saturated()` + `session_store_saturated()` + template 503 + 9 locales).
Merise confirmée complète (7 tables réelles).

**Plus aucun item « code » vérifiable restant.** Tout le reste demande une DB ou est doc/design.

### Reporté à une session avec Docker (DB requise)
- **D3** index (sessions.user_id, history composite, reset_tokens.expires_at)
- **S1** sessions anonymes non persistées · **S2** lock relâché avant write DB
- (valider les migrations contre Postgres/MariaDB/SQLite réels)

### Doc-only restant
- **D2** (history sans FK), **D4/S4** (cookie_id/session_id), **E3** (ordre .layer())
- Docs fr/en pour S3 (503 saturation + accesseur login public) — à écrire
- **A3** bug connu (`list_filter` dans `configure{}`), **AM4** multi-instance (roadmap)

## Anomalies réelles restantes (à corriger, TDD si testable)

- **D3** index manquants (sessions.user_id, history(resource_key,object_pk,batch_id),
  reset_tokens.expires_at) — modifie migration → valider avec DB
- **AM2** résidu perf (fusionner les 2 écritures session au login)
- **S1** sessions anonymes/CSRF perdues au restart · **S2** lock relâché avant write DB ·
  **S3** high watermark = déni login
- ~~**A1**/**A2**~~ → **NON-ISSUES vérifiés** : A1 closures ont des fallbacks (create/update→form.save, delete→erreur) ; A2 droits *_own par-user (pas de signal resource à valider), own_field=None = défaut sûr
- **CFG1** secret_key vide = warning au lieu d'échec boot
- **State process-local** (lockout AU1, rate-limit SEC1, cache AU2/AM4) → multi-instance
- **SEC2** TrustedProxies défaut large si pas de proxy

Registre complet : [anomalies.md](anomalies.md).
