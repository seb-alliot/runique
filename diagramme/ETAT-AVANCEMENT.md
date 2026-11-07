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
- **CX2 + STRICT_CSP mort** : fix 2.1.21 (`strict_csp`→`enable_header_security`) reliait un flag mort
  à un autre flag tout aussi mort (rien ne consommait `enable_header_security` non plus —
  `security_headers_middleware` tournait déjà inconditionnellement). Les trois flags supprimés
  le 2026-09-21 (`enable_csp`/`enable_header_security`/`strict_csp`), comportement réel inchangé.
  HSTS reste gaté `should_emit_hsts`. Test `hsts_tests`.
- **CX3 HSTS codé en dur** : header (`preload` inclus) codé en dur sur 3 sites → source unique
  `hsts_header_value()`, preload opt-in (`HSTS_PRELOAD`), max-age/subdomains configurables,
  warning boot si preload invalide. Statiques : HSTS retiré (host-scoped). Tests `hsts_value_*`.
- **CFG1** : `secret_key` faible (vide/défaut/<32) → **échec boot en prod** via `cross_validate`
  (`secret_key_is_weak`, build.rs:293). Debug → warning. Test `secret_key_tests`.
- Tests ajoutés : finalize (2), parse_multipart staging (1), checked_data (1)

## Faux positifs vérifiés (NE PAS re-flaguer)

- **AM1/M1** : makemigrations gère les ALTER (`diff_schemas`/`Changes.modified_columns`)
- **AM2** divergence session_id : faux (on_conflict disjoints, jamais session_id) — résidu perf
- **TR1** : ErrorContext gaté sur `config.debug`
- **E1** : `enable_debug_errors=true` par défaut · **E2** : `attach_middlewares` code mort → ✅ supprimé (2026-09-21) · **E3** : bug réel ouvert (nonce CSP écrasé par slot 31, voir `anomalies.md`)

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

- **CX2** 🟠 ~~`enable_header_security=false` même en `production()` → HSTS/X-Frame absents en prod~~ → faux, `security_headers_middleware` les pose déjà inconditionnellement ; le flag lui-même supprimé (2026-09-21).
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
- Docs `.env` fr/en + `.env.example` + CHANGELOG pour `HSTS_MAX_AGE`/`HSTS_INCLUDE_SUBDOMAINS`/`HSTS_PRELOAD` (CX3)
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
- ~~**SEC2** TrustedProxies défaut large si pas de proxy~~ → ✅ MITIGÉ (2026-07-26) : défaut edge-aware `default_for_edge(acme_enabled)` → `none()` en mode ACME + doc `.none()`. SEC2b (gate XFP) rejeté (casse Cloudflare). Aussi ce jour : NEW3 (charset ext), NEW4 (dédup CSRF + mask fail-safe), NEW6 (fallback rate-limit peer réel). Détail : `audit-securite-diagrammes.md` (racine)
- ~~**CFG1** secret_key vide~~ → ✅ CORRIGÉ (échec boot prod, cf. section Correctifs)

Registre complet : [anomalies.md](anomalies.md).

## Session 2026-08-30 — derive_form/proc-macro.md mis à jour (retrait v1)

Chantier code (hors diagrammes) : suppression de la grammaire v1 de `model!{}` (`fields: {}`),
parité v2 complétée (types manquants, `readonly`/`label`, `index`/`select_as` confirmés morts et
supprimés), feature `pk-uuid` + `Pk` utilisable sur un champ normal (pas seulement `pk:`).
Diagramme `derive_form/proc-macro.md` réécrit en conséquence : classe AST scindée en
« grammaire DSL » (`FormFieldDecl`/`FormFieldKind`/`FormFieldAttr`, jusqu'ici absente du
diagramme) et « représentation universelle » (`FieldDef`/`FieldType`/`FieldOption`, mise à jour
avec les types actuels) ; pipeline étendu au flux `extend!{}` (jusqu'ici non représenté) ;
nouvelle entrée **DF4** 🟢 documentant les 3 bugs silencieux fermés pendant l'audit v1→v2
(`json` mal routé, 6 options v2 sans effet, `var_binary` mal mappé — détail dans
`CHANGELOG.md [2.2.0]`).

## Session 2026-09-22 — ValidationForm<F>, FormField async, fix silence CSRF, suppression is_get/is_post/is_put/is_delete

Chantier code (branche `validation`) : `FormField::validate`/`finalize` passés en
`async fn` réels (I/O fichier migrée vers `tokio::fs`, bug de fond
`thread_local!`→`tokio::task_local!` corrigé au passage sur le garde de
récursion) ; nouveau type `ValidationForm<F>` (`forms/validation_form.rs`),
garantie de **type** (pas seulement runtime) qu'un form a été validé avant
qu'on puisse agir dessus, avec 3 nouveaux hooks `RuniqueForm`
(`register_dynamic_fields`, `allow_get`, `allow_post`) ; tous les
handlers de demo-app + `auth/password.rs` + le scaffold `runique new` migrés
vers ce pattern. Trouvé en route : un vrai bug de sécurité/UX préexistant —
l'échec CSRF était **totalement silencieux** (aucun message nulle part),
corrigé au niveau framework. Décision finale (après reconsidération) :
`Request::is_get/is_post/is_put/is_delete` **supprimées** (rupture d'API,
2.2.0 déjà publiée) — leur unique usage restant (honeypot) est inliné,
plus aucune raison d'être une fois `ValidationForm` en place.

Diagrammes mis à jour en conséquence : `uml/forms/formulaires.md` (trait
`FormField`/`RuniqueForm` async + hooks, nouvelle classe `ValidationForm<F>`,
entrées **F6** silence CSRF et **F7** suppression is_get&co) ;
`flux/requete-csrf-upload.md` (séquence POST → `ValidationForm::try_new`,
nouvelle entrée **C6**) ; `anomalies.md` (C6 ajoutée en 🟠 Sérieux). Détail
complet du chantier : mémoire `project_form_auto_validate_idea.md` (hors dépôt).

**Suite (même jour)** : migration de Campanile (repo séparé) vers
`ValidationForm`, qui a révélé **F8** — le défaut `allow_get` (`is_submitted()`)
aurait exigé une surcharge manuelle `-> false` sur chaque formulaire mutant de
l'app (login/inscription/contact/devis), répétant un garde de sécurité qui
doit vivre dans le framework. Défaut basculé à `false` inconditionnel dans
`forms/field.rs` ; les surcharges devenues redondantes retirées côté framework
(`ForgotPasswordForm`/`PasswordResetForm`) et côté Campanile (4 formulaires).
Seul cas d'opt-in réel : `UsernameForm` (demo-app, recherche GET-only).

## Session 2026-10-06 — 3.0.0 : runique_dsl, chantier C, champs liste

`makemigrations` lit désormais le DSL avec `runique_dsl`, le même parseur que la macro (fin
des deux lecteurs qui divergeaient : nullabilité, noms magiques, source des FK). Diagrammes mis
à jour :

- `uml/migration/types-builder-et-parsed.md` : `relation` supprimé, `ParsedColumn` (sans
  `created_at`/`updated_at`, avec `max_length`), lecture via `MacroCollector`/`ParsedModel`/
  `to_schema`, `CycleKeys` ; deux entrées ✅ (deux lecteurs du DSL, `max_length` jamais migré).
- `uml/migration/schema-et-diff.md` : `ModelSchema` réduit aux formulaires (`enforce_limits`,
  plus de `to_migration`/`to_model`/`relations`) ; M1 marqué faux positif.
- `uml/derive_form/proc-macro.md` : AST et parseur déplacés dans `runique_dsl`, validations au
  parsing, `Relation` écrit à la main, champs liste ; entrées ✅ FK et enums entiers à 0.
- `flux/auth-session-et-makemigrations.md` / `flux/admin-crud-reset-makemigrations.md` :
  vrai chemin de `makemigrations`, plus de triggers, admin en transaction avec les listes.
- `uml/macros/macros.md` : `HasLists` / `ListField` (`search!(… => Genres has v)`).
- `merise/modele-donnees.md` : `RUNIQUE_USER_TABLE` supprimé ; `uml/auth/authentification.md` :
  méthodes réelles de `BuiltinUserEntity`.

## Session 2026-10-08 — passe mutants, couverture, LogLevel

Tous les survivants de `cargo mutants --iterate` traités (tests vérifiés à la main contre leur
mutation, ou classés avec raison dans le tri). Couverture `runique` : 87.25 % régions,
88.72 % fonctions, 88.81 % lignes (`docs/couverture_test.md`). Diagrammes mis à jour :

- `uml/transverse/utilitaires.md` : `ErrorType::BadRequest` (page 400 de `request.query()?`).
- `uml/utils/tracing-securite-tokens.md` : `Option<LogLevel>` (`tracing::Level` exporté sous ce
  nom dans le prelude).
- `uml/config/configuration.md` : noms réels de `DatabaseEngine` ; `connect()` vérifie le driver
  et masque le mot de passe dans l'erreur.
- `uml/migration/schema-et-diff.md` : `auto_now`/`auto_now_update` sur `ColumnDef`, fuseau gardé
  pour `timestamp_tz`.
- `uml/migration/types-builder-et-parsed.md` : index d'une FK encadré par `unless_mysql_fk`.
- `merise/modele-donnees.md` : `activated_at` et sa contrainte `CHECK` sur `eihwaz_users`.
- `anomalies.md` : TZ1, IX1, SU1, DB1 (trouvés et corrigés les 07-08/10).

## Session 2026-03-22 — alignement diagrammes sur admin 4.0

Le code admin/auth a été entièrement refondu (workspace **4.0.0**) : plus de
`AdminRegistry` / `ResourceEntry` / `admin_main` / closures CRUD. Nouvelle stack :

- `admin/builder` : `ModelAdmin<E: ModelMeta>`, `ListConfig`, `FormConfig`, `AdminSite`,
  `RegisteredModel`, bulk/confirm forms
- `admin/router` : `AdminPaths`, `AdminState`, `build_admin_router`, crud/*, permission/*,
  account/*, history
- `admin/template` + `config` + `middleware` + `trad`
- `auth/gate` : preuves `Authorized<O,E>`, opérations List/View/Create/Update/Delete/ResetPassword,
  `save_as`, rank, transfer, rights_grid
- `auth/meta::ModelMeta`, `auth/permissions::{Rang, rights, groupes_droits colonnes}`,
  `auth/history` (+ `object_repr`), `auth/migrations_table` (ex-`admin/table_admin`)

Schéma données 4.0 : `is_staff` quitte `users` pour `groupes` ; rangs
`is_admin`/`is_dirigeant`/`is_superuser` + CHECK ; `groupes_droits` holder groupe XOR compte
+ granularité colonne + `can_read_own` ; groupes avec responsable/repris_par/is_active.

### Diagrammes réécrits cette session

- `uml/admin/admin-resource-permissions.md` — ModelAdmin + AdminSite + gate + Rang
- `uml/admin/admin-complements.md` — config, templates, MW, history, bulk
- `uml/auth/authentification.md` — CurrentUser, Rang, gate, permissions base
- `merise/modele-donnees.md` — MCD/MLD 4.0
- `flux/admin-crud-reset-makemigrations.md` — séquences gate
- `README.md` + `anomalies.md` (A1/A2/A3 sans objet ; chemins migrations)

### Anomalies 3.x devenues sans objet

- **A1** closures CRUD Option → plus de closures
- **A2** `own_field` → `ModelMeta::OWNER` + preuves
- **A3** `list_filter` builtins sans `filter_fn` → `ListConfig::filter`
- **AM4** cache permissions → déjà lu DB à chaque requête admin

### Reste ouvert (inchangé)

- D2/D3/D4, S1/S2, multi-instance lockout/rate-limit, A7 queryset métier
- Passe fine éventuelle sur uml/app (AdminStaging.site), forms, middleware si drift mineur

## Session 2026-11-04 — diagrammes alignés sur la 4.0 finale

Mis à jour d'après le code : `flux/requete-csrf-upload` (refus cross-site, `request.form(nom)`,
`try_form`, nouvelle séquence `save` + fichier), `flux/auth-session-et-makemigrations` (login,
nouvelle séquence de fermeture), `uml/forms/*` (`FormAccess`, `DbWrite`, `MediaField`,
`UploadMedia`), `uml/middleware/*`, `uml/auth/authentification`, `merise/modele-donnees`
(`session_version`, `session_data` chiffré, `reset_tokens.user_id` unique), `uml/context/*`,
`uml/engine`, `uml/utils`, `uml/derive_form`, `uml/admin/admin-complements`.

Non refait : les liens de ligne des anomalies historiques, conservés pour mémoire et sans doute
décalés. Les 64 blocs Mermaid passent l'analyse de syntaxe (Mermaid 11.4.1), dont 13 corrigés dans cette passe ; leur rendu visuel n'a pas été contrôlé.

## Session 2026-11-07 — diagrammes alignés sur les changements du 05 au 07

Mis à jour d'après le code (40 commits depuis la passe du 04 + lot non commité) :
- `uml/admin/admin-complements` : `AdminConfig` (`filter_size`, `user_resources` retiré),
  système de templates réécrit (pages + blocs, `Overrides<K>`, `ModelTemplates`, `Page` à 11
  valeurs, priorité modèle > global > Runique), contrôles des surcharges au boot
  (`check_replaced_views`, `dispatch`, `missing_pages`, `unknown_variables`), contexte des
  pages (`ListContext`, `Pager`, `FilterOption`, `AccountSummary`, `detail_columns`,
  `admin_page_context`), lien retour au site sur la connexion.
- `uml/admin/admin-resource-permissions` + `flux/admin-crud-reset-makemigrations` : liste
  (`per_page` / `filter_size` bornés, filtres multi-valeurs paginés, `ListFilter`), séquence
  de reset refaite (jeton dans le chemin, `password_get()`, transaction, `no-store`).
- `uml/auth/authentification` : `PasswordResetConfig` réel, `ResetLinkConfig`, clé du `LoginGuard`.
- `uml/app/builder-staging` : `RUNIQUE_ENV` / `config.dev`, étapes de `build()`, ACME à chaud,
  slots (17 HttpsRedirect ajouté, CSP fusionnée au slot 30).
- `uml/config/configuration`, `uml/transverse/utilitaires` (`RuniqueError::Template`,
  `EnvironmentInfo`, `password_init/get`, plafonds de coût), `uml/utils/tracing-securite-tokens`
  (`is_dev`, `css_token`, `private_file`), `uml/forms/*` (`code_values`, champs verrouillés,
  bornes numériques, `ACTIVE_CONTENT`, `register_dynamic_fields` retiré), `flux/requete-csrf-upload`
  (`changes()`), `uml/middleware/securite`, `uml/macros/macros` (`search!` littéral),
  `uml/context/extensions-et-middleware-config` (`display_time`).
- `anomalies.md` : tableau « Passe du 2026-11-05 au 2026-11-07 ».

Complément du même jour (après CI verte) : `uml/derive_form/proc-macro.md` et `uml/forms/formulaires.md`
(refus de la PK à la compilation, `PRIMARY_KEY`, `__same_name`), `uml/app/builder-staging.md`
(ACME décidé sur le certificat), `anomalies.md` (F20, AC2, T1).

## Session 2026-11-07 (soir) — validation empirique + code livré

Code livré cette session, CI verte (confirmée par Sébastien) :
- Refus de la PK à la compilation dans `#[form(fields=[...])]` (`PRIMARY_KEY` généré par
  `model!`/`extend!`, écrit à la main dans `auth/user.rs`, `__same_name` const, doctest
  `compile_fail,E0080`). Diagrammes déjà à jour (voir complément ci-dessus).
- ACME : renouvellement décidé sur le certificat lui-même (`x509-parser`, domaine couvert +
  tiers de vie restant), `expires.txt` abandonné ; correction `axum-server` 0.9
  (`parse::<SocketAddr>()`). Diagramme `uml/app/builder-staging` à jour.
- Tests des modes de reset (`Delegated`/`Manual`) sortis en deux binaires
  (`tests/reset_delegated.rs`, `reset_manual.rs`) à cause du `password_init` global par
  processus. Pas d'impact diagramme.

Pentest local (demo-app sur 127.0.0.1, Postgres local) — VALIDE les diagrammes `uml/admin/*`,
`uml/auth/*` et `anomalies.md` contre l'app réelle :
- Aucune faille : pas d'élévation de rang (plafonds admin/dirigeant tenus en POST direct),
  pas de mass-assignment (colonnes interdites ignorées), pas de bypass, pas d'injection
  SQL/XSS, CSRF/open-redirect/traversal refusés. En-têtes (CSP, HSTS, XFO), rate limiter et
  cookie (HttpOnly, SameSite=Strict) conformes.
- Grille d'admin volontairement incohérente neutralisée : `_own` sans colonne propriétaire
  refusé, colonne `can_read=f` masquée, écriture table `eihwaz_*` refusée.
- COMPORTEMENT VOULU confirmé (pas une anomalie) : un droit `read` au niveau table SANS
  aucune colonne lisible → `authorize_list` refuse avec `NoReadableColumn` (`gate/check.rs:97`,
  « a page of bare keys shows nothing »). Le menu montre la ressource (`visible_to` sur le
  droit table) alors que la liste refuse — attendu. Candidat à documenter dans le diagramme
  de la porte si on veut tracer cette règle.
