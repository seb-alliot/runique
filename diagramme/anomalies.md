# Synthèse des anomalies & bugs latents

Révélés par la mise en diagramme + suivi des flux de données. Sévérité :
🔴 bloquant/sécurité · 🟠 sérieux · 🟡 mineur/dette. Chaque entrée pointe le code.

> Aucune correction appliquée — ce document est le plan de remédiation.

## 🔴 Critiques

| ID | Anomalie | Localisation | Correctif proposé |
|----|----------|--------------|-------------------|
| **C1** | Upload **commité en MEDIA_ROOT avant** CSRF/validation/auth | [parse_html.rs](../runique/src/utils/forms/parse_html.rs) | ✅ **CORRIGÉ (TDD)** — `parse_multipart` écrit en `.staging-{uuid}` ; `finalize` est le seul committer (gère aussi le cas sans `upload_to`). Test `parse_multipart_stages_file_without_eager_commit` |
| **C3** | Aucun **rollback** du fichier si la requête est rejetée → orphelin permanent | [parse_html.rs](../runique/src/utils/forms/parse_html.rs) | ✅ **CORRIGÉ** — `sweep_stale_staging` purge les staging orphelins (TTL), erreurs **loggées** (pas avalées) |
| **D1** | `eihwaz_users_groupes.user_id` codé `.integer()` sans `#[cfg(big-pk)]` → FK incompatible avec `users.id` BIGINT sous feature `big-pk` | [migrations_table.rs:184](../runique/src/admin/table_admin/migrations_table.rs#L184) | ✅ **CORRIGÉ** — bloc `cfg big-pk/else` appliqué |

## 🟠 Sérieux

| ID | Anomalie | Localisation | Correctif proposé |
|----|----------|--------------|-------------------|
| **C2** | CSRF des forms HTML repose **uniquement** sur `req.form()` ; lire `req.prisme.data` direct **contourne** la CSRF | [csrf.rs:152](../runique/src/middleware/security/csrf.rs#L152), [template.rs:406](../runique/src/context/template.rs#L406) | Fail-closed structurel : rejeter au middleware, ou marquer `prisme` inexploitable tant que CSRF KO |
| **A1** | Closures CRUD `Option` → action **no-op silencieuse** si closure absente (bug daemon) | [resource_entry.rs:149](../runique/src/admin/helper/resource_entry.rs#L149) | `None` inattendu → 501/erreur loggée, pas dégradation muette |
| **A2** | `own_field = None` rend `can_*_own` inopérant **sans avertissement** | [resource_entry.rs:162](../runique/src/admin/helper/resource_entry.rs#L162) | Warning au boot si droit `*_own` déclaré sans `own_field` |
| **A3** | `list_filter` dans `configure {}` builtin → 500 `filter_values[col] not found` (bug connu) | `admin/daemon` configure | Pousser `filter_values` dans le contexte pour le chemin `configure` |
| **S1** | Sessions anonymes **non persistées** → CSRF des forms publics perdu au restart | [cleaning_store.rs:148](../runique/src/middleware/session/cleaning_store.rs#L148) | Persister aussi un minimum anonyme (ou CSRF indépendant de la session mémoire) |
| **S2** | `save` relâche le lock **avant** l'écriture DB async → backup DB potentiellement périmé d'un cran | [cleaning_store.rs:378](../runique/src/middleware/session/cleaning_store.rs#L378) | Sérialiser le persist par session (ou versionner le snapshot) |
| **S3** | High watermark → **refus de login** sous pression mémoire | [cleaning_store.rs](../runique/src/middleware/session/cleaning_store.rs) | Garantir une erreur lisible ; isoler la capacité auth de la capacité anonyme |
| **M2** | `to_form_field` retombe en `TextField` pour tout `ColumnType` non géré (binary/inet/interval…) silencieusement | [column/mod.rs:355](../runique/src/migration/column/mod.rs#L355) | Log/warn sur le fallback ; brancher les types manquants |
| **AM2** | Double écriture `eihwaz_sessions` au login (`create` + `upsert`) → **2 aller-retours DB** (perf) | [session.rs:382](../runique/src/auth/session.rs#L382) | 🟡 perf seulement (fusionner les 2 écritures). **Pas de divergence** : voir faux positifs |
| **F3** | `honeypot`/`force_invalid` posés par middleware vs ordre `fill`/`validate` à vérifier | [template.rs:396](../runique/src/context/template.rs#L396) | Confirmer que `force_invalid` est posé avant tout `is_valid()` |
| **D2** | `eihwaz_history.user_id` sans FK (probablement voulu, mais implicite) | [migrations_table.rs:283](../runique/src/admin/table_admin/migrations_table.rs#L283) | **Documenter** le choix (audit survit à la suppression user) |
| **D3** | Index manquants : `history(resource_key,object_pk,batch_id)`, `sessions(user_id)`, `reset_tokens(expires_at)` | [migrations_table.rs](../runique/src/admin/table_admin/migrations_table.rs) | Ajouter les index de requête/purge |
| **CX2 + STRICT_CSP mort** | `enable_header_security=false` partout + `STRICT_CSP` (env, défaut true) **stocké mais jamais consommé** → headers durcis (HSTS/X-Frame/COOP/CORP) jamais posés, même en ACME (Runique = edge TLS) | from_config + csp.rs | ✅ **CORRIGÉ (TDD)** — `from_config` : `enable_header_security = security.strict_csp` (ranime le flag, secure-by-default, builder prioritaire). **HSTS gaté** sur `should_emit_hsts()` (`enforce_https‖acme_enabled`) → pas de lock-in HTTPS. Test `hsts_only_over_real_https` |
| **CFG1** | `secret_key` vide = warning au lieu d'échec boot (CSRF/HMAC cassés en silence) | [server.rs](../runique/src/config/server.rs) | `CheckError` bloquant au boot si vide en mode non-debug |

## 🟡 Mineurs / dette

| ID | Anomalie | Localisation |
|----|----------|--------------|
| **A4** | `bulk POST` exige `can_create` en plus du droit d'opération (quirk préservé) | [admin_main/action.rs](../runique/src/admin/admin_main/action.rs) |
| **C4** | `csrf_gate` possiblement mort ; commentaire `form.rs:194` trompeur | [forms/prisme/csrf_gate.rs](../runique/src/forms/prisme/csrf_gate.rs) |
| **C5** | Seuls GET/HEAD exemptés de CSRF (OPTIONS/TRACE traités mutants) | [extractor.rs:75](../runique/src/forms/extractor.rs#L75) |
| **E3** | Écart ordre d'écriture vs ordre d'exécution des `.layer()` Axum | [engine/core.rs:110](../runique/src/engine/core.rs#L110) |
| **E4** | `session_store`/`session_db_store` `LazyLock<RwLock<Option>>` → vérifier aucun `unwrap` avant init | [engine/core.rs:48](../runique/src/engine/core.rs#L48) |
| **F1** | Hook `customize` seulement sur l'arm `(model)` de `impl_form_access!` | [impl_form.rs](../runique/src/macros/forms/impl_form.rs) |
| **F2/M3** | `max_size` appliqué par deux chemins (`to_form_field` + AdminForm généré) → risque de divergence | [column/mod.rs](../runique/src/migration/column/mod.rs) |
| **AM3** | TTL 24h codé en dur en double (set_expiry + expires_at) | [session.rs:363](../runique/src/auth/session.rs#L363) |
| **AM4** | Cache permissions process-local → périmé en multi-instance | [auth/session.rs](../runique/src/auth/session.rs) |
| **D4/S4** | Double identifiant `cookie_id`/`session_id` à documenter | [migrations_table.rs:209](../runique/src/admin/table_admin/migrations_table.rs#L209) |
| **E1** | `enable_debug_errors` : nom trompeur ; défaut `true` partout (le handler EST monté en prod) mais le désactiver retirerait toutes les pages d'erreur | [config.rs:65](../runique/src/middleware/config.rs#L65) |
| **E2** | `RuniqueEngine::attach_middlewares` = **code mort** (aucun appelant ; le staging applicator est le chemin vivant) | [engine/core.rs:110](../runique/src/engine/core.rs#L110) |

## Faux positifs écartés après vérification

| ID | Hypothèse initiale | Réalité vérifiée |
|----|--------------------|------------------|
| **M1/AM1** | `makemigrations` ne gérerait pas les `ALTER COLUMN` | **Faux.** makemigrations utilise `diff_schemas` ([makemigration.rs:489](../runique/src/utils/cli/makemigration.rs#L489)) qui calcule `modified_columns` ([diff.rs:90](../runique/src/migration/utils/diff.rs#L90)). `ModelSchema::diff` (limité à add/drop) est un diff secondaire **non** utilisé par la CLI. |
| **E1 (sévérité)** | Pas de pages d'erreur en prod | **Faux par défaut.** `enable_debug_errors` vaut `true` dans tous les presets ; handler monté en prod. Reste seulement le risque si on le désactive (rétrogradé en 🟡). |
| **AM2 (divergence)** | `session_id` divergent entre `create` et `upsert` | **Faux.** `save()`/`upsert_session` insère en 1er ; `create()` arrive en conflit et son `on_conflict` ne met à jour que `[UserId, ExpiresAt]` (jamais `session_id`). `session_id` final déterministe. Résidu réel = double écriture (perf 🟡). |
| **TR1** | `ErrorContext` exposé en prod | **Faux.** Rendu gaté sur `config.debug` ([error.rs:96](../runique/src/middleware/errors/error.rs#L96)). |

## Tracing — erreurs avalées corrigées (chantier « zéro erreur avalée »)

Toutes compilées + testées (215/215 vert).

| Site | Avant | Après |
|------|-------|-------|
| `forms/field.rs` rollback txn (×4) | `let _ = txn.rollback()` | `rollback_traced()` → `warn!` |
| `forms/fields/file.rs` cleanup + ancien fichier | `let _ =` | `warn!` (filtre NotFound) |
| `admin/admin_main/handle_crud.rs` ancien upload | `let _ =` | `warn!` (filtre NotFound) |
| `admin/history.rs` insert audit | `let _ =` muet | `warn!` (audit row lost) |
| `admin/builtin/user.rs` user→groupe (×2, RBAC) | `let _ =` | let-chain + `warn!` |
| `utils/reset_token` cleanup | `let _ =` | `warn!` |
| `middleware/security/anti_bot.rs` session honeypot | `let _ =` | `warn!` |
| `auth/password.rs` template email reset | `if let Ok` sans else | `else` → `warn!` (email non envoyé visible) |
| `auth/guard.rs` cache permissions (×4) | `if let Ok(write())` skip | `unwrap_or_else(into_inner)` + `warn!` (récupère le lock empoisonné) |
| `utils/forms/parse_html.rs` (C3) | `let _ =` | `warn!` ×3 + sweep tracé |
| `utils/mailer` `MailerConfig` Debug | `derive(Debug)` (password en clair) | Debug manuel masquant `password: ***` |

Bénins classés (pas des avalages) : `write!(String,…)` (generator/parsers), `dotenv().ok()`,
`OnceCell::set().ok()`, `rustls install_default()`, `LoginGuard`/`RateLimiter` (déjà
`into_inner()`).

### Nouvelles anomalies transverses (uml/transverse)

| ID | Anomalie | Statut |
|----|----------|--------|
| **TR2** | `MailerConfig` dérivait `Debug` → password SMTP en clair dans les logs | ✅ **CORRIGÉ** (Debug manuel `***`) |
| **TR1** | `ErrorContext` (debug_repr/stack_trace/headers) exposés en prod ? | ✅ **FAUX POSITIF** — `error.rs:96` gate sur `config.debug` : prod → `render_500/404` sans détails ; page debug seulement en debug |

## Thèmes transverses

1. **Erreurs avalées silencieusement** (A1, A2, M2, S1) — rejoint le chantier « zéro erreur
   avalée » : un `None`/fallback inattendu doit logger/erreur, jamais dégrader en silence.
2. **Deux sources de vérité** (E2, AM2, F2, AM3) — collapser chaque paire en un seul chemin.
3. **Ordre des opérations sécurité** (C1, C2, C3, F3) — la validation (CSRF/extension) doit
   précéder tout effet de bord persistant (écriture fichier, save).
4. **`big-pk` non propagé** (D1) — auditer **tous** les `user_id`/PK pour la cohérence `cfg`.
