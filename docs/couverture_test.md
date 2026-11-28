# Couverture de tests — framework Runique

Snapshot du **2026-11-28** · commande : `cargo llvm-cov -p runique -p runique-cli -p runique_i18n -p runique_atomic -p runique_dsl --features runique/all-databases,runique/admin,runique/test-utils`

4 273 tests passés, 0 en échec, 4 ignorés.

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| runique | 93.33 % | 93.73 % | 93.89 % |
| runique-cli | 91.01 % | 92.52 % | 91.33 % |
| runique_i18n | 94.40 % | 96.33 % | 94.89 % |
| runique_atomic | 97.70 % | 96.76 % | 98.21 % |
| runique_dsl | 95.11 % | 99.13 % | 97.56 % |
| **TOTAL** | **93.18 %** | **93.92 %** | **93.75 %** |

Évolution de `runique` depuis le 2026-11-12 : régions 92.14 % → 93.33 %, fonctions 92.77 % → 93.73 %, lignes 92.93 % → 93.89 %. Les autres crates sont mesurées pour la première fois : jusqu'ici, la commande ne prenait que `runique`.

Depuis le snapshot précédent, la commande `runique` et `makemigrations` sont sortis dans `runique-cli`, et `runique/src` est réorganisé en dossiers par sujet (`forms/`, `security/`, `mailer/`, `http/`, `router/`, `log/`…) : les chemins des fichiers ne correspondent plus à ceux du 2026-11-12.

---

# runique

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **93.33 %** | **93.73 %** | **93.89 %** |

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/bulk_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/confirm_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/form.rs | 98.46 % | 94.29 % | 98.96 % |
| builder/list.rs | 98.94 % | 100.00 % | 99.17 % |
| builder/model_admin.rs | 96.23 % | 100.00 % | 95.71 % |
| builder/registered.rs | 93.13 % | 93.44 % | 93.07 % |
| builder/site.rs | 98.07 % | 100.00 % | 99.15 % |
| config/settings.rs | 97.58 % | 94.12 % | 97.03 % |
| middleware/access.rs | 91.83 % | 96.72 % | 93.82 % |
| router/account/memberships.rs | 76.83 % | 87.50 % | 82.69 % |
| router/account/reset_password.rs | 87.90 % | 88.89 % | 92.03 % |
| router/account/summary.rs | 95.10 % | 100.00 % | 98.48 % |
| router/context.rs | 97.76 % | 97.14 % | 97.19 % |
| router/crud/bulk.rs | 89.39 % | 100.00 % | 95.24 % |
| router/crud/create.rs | 91.89 % | 100.00 % | 90.00 % |
| router/crud/delete.rs | 95.77 % | 100.00 % | 95.00 % |
| router/crud/detail.rs | 91.72 % | 100.00 % | 92.57 % |
| router/crud/edit.rs | 96.43 % | 100.00 % | 95.65 % |
| router/crud/list.rs | 89.05 % | 86.67 % | 90.51 % |
| router/history.rs | 90.21 % | 95.83 % | 94.82 % |
| router/mod.rs | 97.14 % | 100.00 % | 98.11 % |
| router/path/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| router/permission/rank.rs | 86.67 % | 100.00 % | 88.00 % |
| router/permission/rights.rs | 90.19 % | 94.44 % | 93.20 % |
| router/permission/transfer.rs | 86.64 % | 92.31 % | 87.20 % |
| router/routes.rs | 87.64 % | 81.08 % | 87.56 % |
| template/assemble.rs | 97.05 % | 97.87 % | 96.83 % |
| template/block.rs | 100.00 % | 100.00 % | 100.00 % |
| template/page.rs | 100.00 % | 100.00 % | 100.00 % |
| template/stylesheet.rs | 98.74 % | 100.00 % | 98.94 % |
| translation/mod.rs | 100.00 % | 100.00 % | 100.00 % |

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 97.30 % | 100.00 % | 95.90 % |
| builder/checks.rs | 86.75 % | 100.00 % | 85.48 % |
| builder/mod.rs | 83.46 % | 82.35 % | 82.26 % |
| builder/routes.rs | 95.20 % | 91.30 % | 96.61 % |
| builder/start.rs | 51.52 % | 60.00 % | 50.00 % |
| builder/static_files.rs | 96.81 % | 100.00 % | 97.14 % |
| builder/templates.rs | 93.15 % | 87.50 % | 98.11 % |
| runique_app.rs | 4.08 % | 10.00 % | 3.57 % |
| staging/admin.rs | 87.37 % | 91.84 % | 88.56 % |
| staging/applicator.rs | 88.38 % | 100.00 % | 85.71 % |
| staging/core.rs | 80.26 % | 90.00 % | 84.38 % |
| staging/middleware.rs | 79.49 % | 70.37 % | 80.19 % |
| staging/statics.rs | 72.22 % | 75.00 % | 77.27 % |

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| current_user.rs | 100.00 % | 100.00 % | 100.00 % |
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/change.rs | 99.21 % | 97.22 % | 98.56 % |
| gate/check.rs | 97.80 % | 98.08 % | 98.70 % |
| gate/decide.rs | 98.98 % | 100.00 % | 99.11 % |
| gate/membership.rs | 93.16 % | 100.00 % | 97.39 % |
| gate/proof.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/rank.rs | 96.78 % | 100.00 % | 98.17 % |
| gate/refusal.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/rights_grid.rs | 95.28 % | 100.00 % | 99.13 % |
| gate/save.rs | 97.55 % | 98.16 % | 98.75 % |
| gate/test_support.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/transfer.rs | 97.25 % | 100.00 % | 99.33 % |
| gate/write.rs | 94.61 % | 95.83 % | 95.89 % |
| guard.rs | 88.80 % | 96.15 % | 91.81 % |
| history.rs | 96.13 % | 100.00 % | 94.96 % |
| meta.rs | 68.42 % | 66.67 % | 64.00 % |
| migrations_table.rs | 92.82 % | 67.74 % | 92.95 % |
| password.rs | 85.93 % | 98.15 % | 86.94 % |
| permissions/group.rs | 81.40 % | 50.00 % | 75.00 % |
| permissions/group_rights.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/mod.rs | 96.33 % | 100.00 % | 96.16 % |
| permissions/rank.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/rights.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/user_groups.rs | 100.00 % | 100.00 % | 100.00 % |
| session.rs | 75.37 % | 68.09 % | 71.91 % |
| user.rs | 88.25 % | 87.18 % | 92.62 % |
| user_trait.rs | 100.00 % | 100.00 % | 100.00 % |

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| env.rs | 98.55 % | 96.55 % | 97.70 % |
| security.rs | 98.51 % | 93.75 % | 99.08 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 96.43 % | 92.59 % | 98.04 % |

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| adb.rs | 73.91 % | 70.59 % | 73.81 % |
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 95.88 % | 98.89 % | 95.99 % |
| engine.rs | 100.00 % | 100.00 % | 100.00 % |
| sql_dialect.rs | 93.08 % | 92.59 % | 94.25 % |
| test_db.rs | 70.37 % | 60.00 % | 65.22 % |
| write.rs | 100.00 % | 100.00 % | 100.00 % |

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 95.00 % | 100.00 % | 97.87 % |

## errors

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app_error.rs | 93.55 % | 100.00 % | 95.24 % |
| build.rs | 85.93 % | 100.00 % | 87.72 % |
| context.rs | 89.83 % | 100.00 % | 90.86 % |
| debug_page.rs | 94.81 % | 100.00 % | 97.62 % |
| error.rs | 76.47 % | 100.00 % | 93.33 % |
| payload_too_large.rs | 100.00 % | 100.00 % | 100.00 % |

## flash

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| extractor.rs | 100.00 % | 100.00 % | 100.00 % |
| message.rs | 100.00 % | 100.00 % | 100.00 % |

## forms

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| config/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| config/field_config.rs | 100.00 % | 100.00 % | 100.00 % |
| config/limit.rs | 100.00 % | 100.00 % | 100.00 % |
| constants.rs | 100.00 % | 100.00 % | 100.00 % |
| data/column_value.rs | 93.17 % | 82.14 % | 95.43 % |
| data/form_data.rs | 85.29 % | 66.67 % | 85.71 % |
| data/save.rs | 88.89 % | 80.00 % | 84.00 % |
| database_error.rs | 91.69 % | 88.00 % | 92.74 % |
| fields/binary.rs | 97.45 % | 95.24 % | 96.15 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 93.81 % | 93.75 % | 90.70 % |
| fields/datetime.rs | 78.95 % | 89.47 % | 81.37 % |
| fields/file.rs | 97.80 % | 95.43 % | 97.37 % |
| fields/hidden.rs | 90.62 % | 80.00 % | 86.76 % |
| fields/media.rs | 96.15 % | 94.41 % | 95.20 % |
| fields/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/number.rs | 95.08 % | 100.00 % | 95.83 % |
| fields/special.rs | 82.80 % | 80.00 % | 79.71 % |
| fields/text.rs | 96.55 % | 96.88 % | 97.93 % |
| form.rs | 82.52 % | 90.48 % | 86.44 % |
| generic.rs | 91.40 % | 100.00 % | 97.92 % |
| prisme/aegis.rs | 88.03 % | 83.33 % | 85.34 % |
| prisme/extractor.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/multipart.rs | 94.46 % | 95.80 % | 91.43 % |
| prisme/refusal.rs | 100.00 % | 100.00 % | 100.00 % |
| renderer.rs | 83.95 % | 84.62 % | 88.60 % |
| traits/form_entity.rs | 100.00 % | 100.00 % | 100.00 % |
| traits/form_field.rs | 97.57 % | 96.83 % | 96.84 % |
| traits/model_form.rs | 100.00 % | 100.00 % | 100.00 % |
| traits/runique_form.rs | 73.74 % | 81.25 % | 75.15 % |
| validation_form/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| validation_form/write.rs | 82.23 % | 81.25 % | 84.35 % |
| validator.rs | 88.35 % | 100.00 % | 89.23 % |

## http

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| context.rs | 93.33 % | 100.00 % | 100.00 % |
| extension.rs | 100.00 % | 100.00 % | 100.00 % |
| og_image.rs | 100.00 % | 100.00 % | 100.00 % |
| request.rs | 91.84 % | 87.18 % | 90.48 % |
| url_params.rs | 100.00 % | 100.00 % | 100.00 % |

## log

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| admin.rs | 100.00 % | 100.00 % | 100.00 % |
| auth.rs | 85.71 % | 80.00 % | 85.00 % |
| builder.rs | 90.32 % | 85.71 % | 90.00 % |
| db.rs | 100.00 % | 100.00 % | 100.00 % |
| errors.rs | 100.00 % | 100.00 % | 100.00 % |
| forms.rs | 90.32 % | 85.71 % | 90.00 % |
| init.rs | 100.00 % | 100.00 % | 100.00 % |
| mailer.rs | 100.00 % | 100.00 % | 100.00 % |
| middleware.rs | 100.00 % | 100.00 % | 100.00 % |
| migration.rs | 100.00 % | 100.00 % | 100.00 % |
| mod.rs | 90.43 % | 79.41 % | 88.76 % |
| output.rs | 81.25 % | 87.50 % | 87.84 % |
| session.rs | 100.00 % | 100.00 % | 100.00 % |
| templates.rs | 100.00 % | 100.00 % | 100.00 % |
| trace_ext.rs | 88.08 % | 100.00 % | 94.94 % |

## macros

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| context/flash.rs | 100.00 % | 100.00 % | 100.00 % |
| context/helper.rs | 100.00 % | 100.00 % | 100.00 % |
| context/impl_error.rs | 100.00 % | 100.00 % | 100.00 % |
| database/list.rs | 89.08 % | 77.78 % | 78.87 % |
| database/objects.rs | 88.57 % | 94.74 % | 97.15 % |
| database/query.rs | 86.53 % | 90.20 % | 93.09 % |
| forms/enums.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/impl_form.rs | 100.00 % | 100.00 % | 100.00 % |

## mailer

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| mod.rs | 83.46 % | 86.79 % | 87.11 % |

## middleware

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| config/compression_config.rs | 60.61 % | 40.00 % | 60.61 % |
| config/cors_config.rs | 87.84 % | 100.00 % | 90.00 % |
| config/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| config/extensions.rs | 100.00 % | 100.00 % | 100.00 % |
| config/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| config/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| config/permissions_policy_config.rs | 100.00 % | 100.00 % | 100.00 % |
| config/trusted_proxies_config.rs | 97.38 % | 100.00 % | 99.10 % |
| dev/cache.rs | 100.00 % | 100.00 % | 100.00 % |
| errors/error.rs | 88.91 % | 88.89 % | 88.61 % |
| lang/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| security/allowed_hosts.rs | 97.15 % | 95.24 % | 97.42 % |
| security/anti_bot.rs | 96.83 % | 100.00 % | 97.62 % |
| security/csp.rs | 94.21 % | 90.00 % | 97.65 % |
| security/csrf.rs | 90.11 % | 89.47 % | 90.48 % |
| security/open_redirect.rs | 97.83 % | 100.00 % | 96.71 % |
| security/permissions_policy.rs | 100.00 % | 100.00 % | 100.00 % |
| security/rate_limit.rs | 93.38 % | 91.18 % | 93.39 % |
| security/trusted_proxies.rs | 99.53 % | 100.00 % | 99.58 % |
| session/auth.rs | 95.45 % | 100.00 % | 95.12 % |
| session/cleaning_store.rs | 91.43 % | 94.00 % | 93.56 % |
| session/session_db.rs | 93.85 % | 96.00 % | 95.19 % |
| session/ttl_upgrade.rs | 100.00 % | 100.00 % | 100.00 % |

## router

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| register_url.rs | 87.02 % | 64.71 % | 89.66 % |
| router_ext.rs | 100.00 % | 100.00 % | 100.00 % |
| routes.rs | 95.79 % | 91.67 % | 94.67 % |

## runique_test

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder.rs | 89.04 % | 87.80 % | 88.64 % |
| expect.rs | 80.95 % | 100.00 % | 93.75 % |
| outcome.rs | 97.40 % | 100.00 % | 98.43 % |
| transaction.rs | 92.59 % | 81.25 % | 94.52 % |

## schema

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| column.rs | 73.35 % | 85.00 % | 84.84 % |
| foreign_key.rs | 100.00 % | 100.00 % | 100.00 % |
| hooks.rs | 100.00 % | 100.00 % | 100.00 % |
| index.rs | 100.00 % | 100.00 % | 100.00 % |
| mod.rs | 95.62 % | 95.45 % | 95.42 % |
| primary_key.rs | 100.00 % | 100.00 % | 100.00 % |

## security

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| constants.rs | 100.00 % | 100.00 % | 100.00 % |
| csp_nonce.rs | 100.00 % | 100.00 % | 100.00 % |
| csrf.rs | 100.00 % | 100.00 % | 100.00 % |
| escape.rs | 100.00 % | 100.00 % | 100.00 % |
| password.rs | 88.45 % | 89.29 % | 92.13 % |
| reset_token/entity.rs | 92.86 % | 100.00 % | 100.00 % |
| reset_token/mod.rs | 87.21 % | 100.00 % | 86.73 % |
| sanitizer.rs | 99.10 % | 100.00 % | 99.18 % |
| session_cipher.rs | 98.84 % | 100.00 % | 100.00 % |

## templates

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| constants.rs | 100.00 % | 100.00 % | 100.00 % |
| contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| form.rs | 93.33 % | 96.88 % | 91.10 % |
| integrity.rs | 95.45 % | 100.00 % | 100.00 % |
| loader.rs | 97.38 % | 96.30 % | 97.49 % |
| static_tera.rs | 98.26 % | 97.56 % | 98.67 % |
| url.rs | 95.83 % | 100.00 % | 97.83 % |

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| aliases/helpers.rs | 69.23 % | 66.67 % | 66.67 % |

---

# runique-cli

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **91.01 %** | **92.52 %** | **91.33 %** |

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder.rs | 12.50 % | 16.67 % | 12.50 % |
| config.rs | 95.62 % | 97.26 % | 95.25 % |
| engine.rs | 100.00 % | 100.00 % | 100.00 % |
| sql_dialect.rs | 100.00 % | 100.00 % | 100.00 % |

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| helpers/markers.rs | 100.00 % | 100.00 % | 100.00 % |
| helpers/schemas.rs | 100.00 % | 100.00 % | 100.00 % |
| makemigrations.rs | 58.62 % | 33.33 % | 65.22 % |
| makemigrations/orchestrator/build.rs | 96.00 % | 100.00 % | 100.00 % |
| makemigrations/orchestrator/check.rs | 96.15 % | 100.00 % | 100.00 % |
| makemigrations/orchestrator/compare.rs | 96.10 % | 100.00 % | 95.74 % |
| makemigrations/orchestrator/create_files.rs | 100.00 % | 100.00 % | 100.00 % |
| makemigrations/orchestrator/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| makemigrations/orchestrator/nothing_to_migrate.rs | 97.14 % | 100.00 % | 100.00 % |
| makemigrations/orchestrator/order.rs | 92.86 % | 100.00 % | 100.00 % |
| makemigrations/orchestrator/read.rs | 90.00 % | 100.00 % | 100.00 % |
| master/create_superuser.rs | 26.92 % | 66.67 % | 28.57 % |
| master/makemigrations.rs | 90.70 % | 80.00 % | 92.59 % |
| master/migration.rs | 69.23 % | 80.00 % | 79.31 % |
| master/mod.rs | 69.57 % | 60.00 % | 70.59 % |
| master/project.rs | 87.50 % | 100.00 % | 100.00 % |
| paths.rs | 100.00 % | 100.00 % | 100.00 % |
| runtime/check/key_changes.rs | 75.00 % | 100.00 % | 83.33 % |
| runtime/create/enum_type.rs | 81.82 % | 100.00 % | 90.91 % |
| runtime/create/enum_value.rs | 81.82 % | 100.00 % | 90.91 % |
| runtime/create/key.rs | 81.82 % | 100.00 % | 90.91 % |
| runtime/delete/enum_type.rs | 80.00 % | 100.00 % | 85.71 % |
| runtime/delete/key.rs | 83.33 % | 100.00 % | 90.91 % |
| runtime/engine/mod.rs | 50.00 % | 50.00 % | 33.33 % |
| runtime/engine/postgres.rs | 98.33 % | 100.00 % | 100.00 % |
| runtime/engine/server.rs | 100.00 % | 100.00 % | 100.00 % |
| runtime/engine/sqlite.rs | 100.00 % | 100.00 % | 100.00 % |
| runtime/engine/text.rs | 100.00 % | 100.00 % | 100.00 % |
| runtime/launch.rs | 100.00 % | 100.00 % | 100.00 % |
| runtime/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| runtime/rename/enum_type.rs | 81.82 % | 100.00 % | 90.91 % |
| runtime/rename/enum_value.rs | 87.50 % | 100.00 % | 92.31 % |
| runtime/rename/key.rs | 80.00 % | 100.00 % | 92.31 % |
| steps/apply/applied.rs | 98.68 % | 100.00 % | 100.00 % |
| steps/apply/applied_renames.rs | 98.57 % | 97.50 % | 97.32 % |
| steps/apply/migrate.rs | 93.08 % | 94.94 % | 92.04 % |
| steps/changes/check_parallel.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/create/cascade_key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/create/cycle_keys.rs | 96.97 % | 100.00 % | 97.22 % |
| steps/changes/create/table_order.rs | 99.09 % | 100.00 % | 100.00 % |
| steps/changes/delete/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/delete/dropped_as_changes.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/delete/key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/delete/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/delete/table_order.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/destructive.rs | 91.45 % | 91.67 % | 93.33 % |
| steps/changes/modify/column_decimal.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/modify/column_length.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/modify/column_required.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/modify/column_type.rs | 94.59 % | 100.00 % | 88.57 % |
| steps/changes/modify/decimal_may_shrink.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/modify/length_may_shrink.rs | 90.91 % | 100.00 % | 85.71 % |
| steps/changes/modify/primary_key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/modify/unmigrated_type_changes.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/name_length_limit.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/rename/follow_renames.rs | 98.82 % | 100.00 % | 98.36 % |
| steps/changes/rename/settle_renamed_keys.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/changes/report.rs | 64.18 % | 100.00 % | 71.43 % |
| steps/diff/create/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/create/enum_value.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/create/index.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/create/key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/create/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/delete/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/delete/enum_value.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/delete/index.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/delete/key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/delete/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/extensions.rs | 96.00 % | 100.00 % | 100.00 % |
| steps/diff/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/models.rs | 98.21 % | 100.00 % | 100.00 % |
| steps/diff/modify/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/modify/redefined.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/rename/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/rename/enum_type.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/rename/enum_value.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/diff/rename/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/alter.rs | 95.00 % | 100.00 % | 97.01 % |
| steps/generate/builder.rs | 99.83 % | 97.87 % | 99.79 % |
| steps/generate/columns/col_type_method.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/columns/enum_column_type.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/columns/render_column_def.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/constraints/fk_constraint_name.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/constraints/fk_index_column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/constraints/key_changes_guard.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/constraints/render_fk_create.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/constraints/unless_mysql_fk.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/enum_type.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/enum_value.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/index/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/index/statement.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/key/cycle_keys.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/key/snapshot_key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/table/columns.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/table/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/create/table/primary_key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/enum_type.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/enum_value.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/index.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/key/cycle_keys.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/key/snapshot_key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/delete/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/enum_file.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/enum_snapshot.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/init_migrate.rs | 99.14 % | 100.00 % | 100.00 % |
| steps/generate/instruction.rs | 95.52 % | 100.00 % | 94.12 % |
| steps/generate/lib_file.rs | 98.25 % | 100.00 % | 98.50 % |
| steps/generate/mod.rs | 99.08 % | 100.00 % | 99.35 % |
| steps/generate/modify/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/modify/column_type.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/modify/enum_column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/build_drop_plan.rs | 97.83 % | 100.00 % | 95.83 % |
| steps/generate/plan/build_enum_plan.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/build_extend_plan.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/build_main_plan.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/commit_plan.rs | 93.62 % | 100.00 % | 90.91 % |
| steps/generate/plan/replace_waiting.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/settled_migrations.rs | 91.67 % | 100.00 % | 100.00 % |
| steps/generate/plan/stage.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/stages.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/waiting_migrate.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/plan/waiting_migrations.rs | 94.44 % | 100.00 % | 100.00 % |
| steps/generate/rename/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/rename/enum_type.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/rename/enum_value.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/rename/index.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/rename/key.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/generate/rename/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/enums/of_schemas.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/enums/previous.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/enums/read_snapshots.rs | 93.88 % | 80.00 % | 96.43 % |
| steps/parser/enums/snapshot.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/extend.rs | 91.43 % | 75.00 % | 95.00 % |
| steps/parser/files/enums_of.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/project_engine.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/project_pk/alias_used.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/project_pk/cargo_metadata.rs | 57.58 % | 50.00 % | 47.83 % |
| steps/parser/files/project_pk/features_of.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/project_pk/has_word.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/project_pk/host_triple.rs | 91.67 % | 100.00 % | 100.00 % |
| steps/parser/files/project_pk/mod.rs | 99.52 % | 90.91 % | 98.90 % |
| steps/parser/files/project_pk/pk_of.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/project_pk/with_project_pk.rs | 50.00 % | 100.00 % | 80.00 % |
| steps/parser/files/rust_files.rs | 99.26 % | 100.00 % | 100.00 % |
| steps/parser/files/scan/merge_extend_schemas.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/scan/scan_entities.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/scan/scan_extend_blocks.rs | 82.35 % | 50.00 % | 88.89 % |
| steps/parser/files/scan/scan_models.rs | 94.29 % | 77.78 % | 94.85 % |
| steps/parser/files/snapshot/extend_snapshot_content.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/snapshot/extended_tables.rs | 93.33 % | 100.00 % | 100.00 % |
| steps/parser/files/snapshot/parse_create_file.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/snapshot/previous_snapshot.rs | 85.00 % | 100.00 % | 100.00 % |
| steps/parser/files/snapshot/read_all.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/snapshot/read_extend.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/files/snapshot/records_lengths.rs | 85.71 % | 100.00 % | 100.00 % |
| steps/parser/files/snapshot/snapshot_upgrades.rs | 93.75 % | 100.00 % | 100.00 % |
| steps/parser/model/declared_renames.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/located.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/parse_model_from_source.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/parse_schema_from_source.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/to_schema/column.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/to_schema/column_type.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/to_schema/enum_values.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/to_schema/find_enum.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/model/to_schema/list_table.rs | 96.92 % | 100.00 % | 96.61 % |
| steps/parser/model/to_schema/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/seaorm/column_decimal.rs | 84.00 % | 100.00 % | 93.33 % |
| steps/parser/seaorm/column_default.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/seaorm/column_length.rs | 92.00 % | 100.00 % | 92.31 % |
| steps/parser/seaorm/enum_struct.rs | 92.94 % | 100.00 % | 90.57 % |
| steps/parser/seaorm/enum_type_info.rs | 85.19 % | 100.00 % | 78.95 % |
| steps/parser/seaorm/index.rs | 87.80 % | 50.00 % | 96.55 % |
| steps/parser/seaorm/is_column_def.rs | 83.33 % | 100.00 % | 75.00 % |
| steps/parser/seaorm/key.rs | 91.67 % | 50.00 % | 96.88 % |
| steps/parser/seaorm/mod.rs | 80.30 % | 55.56 % | 85.29 % |
| steps/parser/seaorm/table.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/syn_extract/collect_chain.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/syn_extract/collect_method_names.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/syn_extract/collect_str_args.rs | 94.87 % | 100.00 % | 93.10 % |
| steps/parser/syn_extract/extract_alias_new_str.rs | 97.44 % | 100.00 % | 96.43 % |
| steps/parser/syn_extract/extract_alias_new_str_inner.rs | 90.48 % | 100.00 % | 85.71 % |
| steps/parser/syn_extract/extract_all_str_args.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/syn_extract/extract_fk_action.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/syn_extract/extract_fk_action_value.rs | 88.89 % | 100.00 % | 83.33 % |
| steps/parser/syn_extract/extract_references_from_expr.rs | 44.83 % | 100.00 % | 42.11 % |
| steps/parser/syn_extract/extract_str_from_call.rs | 75.00 % | 100.00 % | 82.61 % |
| steps/parser/syn_extract/get_root_expr.rs | 100.00 % | 100.00 % | 100.00 % |
| steps/parser/syn_extract/method_names_in_expr.rs | 100.00 % | 100.00 % | 100.00 % |
| types.rs | 100.00 % | 100.00 % | 100.00 % |

## new_project

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| mod.rs | 90.47 % | 96.30 % | 87.77 % |

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| mod.rs | 0.00 % | 0.00 % | 0.00 % |
| password.rs | 30.70 % | 35.29 % | 36.08 % |

## racine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| auth.rs | 100.00 % | 100.00 % | 100.00 % |
| create_superuser.rs | 70.30 % | 78.08 % | 70.05 % |
| main.rs | 0.00 % | 0.00 % | 0.00 % |
| test_runner.rs | 73.79 % | 86.30 % | 74.20 % |
| versions.rs | 100.00 % | 100.00 % | 100.00 % |

---

# runique_i18n

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **94.40 %** | **96.33 %** | **94.89 %** |

## racine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| project.rs | 90.91 % | 94.12 % | 91.37 % |
| switch_lang.rs | 98.09 % | 97.33 % | 97.95 % |

---

# runique_atomic

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **97.70 %** | **96.76 %** | **98.21 %** |

## action

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| create.rs | 100.00 % | 100.00 % | 100.00 % |
| finish.rs | 88.89 % | 100.00 % | 100.00 % |
| mod.rs | 89.47 % | 100.00 % | 100.00 % |
| modify.rs | 89.29 % | 100.00 % | 100.00 % |
| rollback.rs | 82.76 % | 100.00 % | 100.00 % |

## step

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| done.rs | 100.00 % | 100.00 % | 100.00 % |
| mod.rs | 100.00 % | 100.00 % | 100.00 % |
| placed.rs | 100.00 % | 100.00 % | 100.00 % |
| prepared.rs | 94.44 % | 100.00 % | 100.00 % |
| replaced.rs | 100.00 % | 100.00 % | 100.00 % |
| waiting.rs | 96.43 % | 100.00 % | 100.00 % |

## racine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| disk.rs | 98.37 % | 100.00 % | 98.62 % |
| journal.rs | 96.70 % | 100.00 % | 97.90 % |
| manager.rs | 98.26 % | 93.83 % | 97.93 % |
| private_file.rs | 95.89 % | 100.00 % | 100.00 % |
| support.rs | 96.00 % | 88.89 % | 93.33 % |

---

# runique_dsl

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **95.11 %** | **99.13 %** | **97.56 %** |

## parser

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| enum_def.rs | 93.44 % | 100.00 % | 98.60 % |
| fk.rs | 100.00 % | 100.00 % | 100.00 % |
| form_field/attr_name.rs | 61.54 % | 100.00 % | 61.54 % |
| form_field/parse.rs | 95.36 % | 100.00 % | 99.20 % |
| form_field/size.rs | 98.94 % | 100.00 % | 100.00 % |
| form_field/suggest_type.rs | 100.00 % | 100.00 % | 100.00 % |
| form_field/to_field_def.rs | 96.86 % | 97.22 % | 98.10 % |
| form_field/validate_attrs.rs | 93.03 % | 100.00 % | 93.55 % |
| meta.rs | 97.18 % | 100.00 % | 100.00 % |
| model_input.rs | 95.22 % | 100.00 % | 99.20 % |
| pk.rs | 96.15 % | 100.00 % | 98.21 % |
| relation.rs | 91.58 % | 100.00 % | 94.01 % |
| sql_identifier.rs | 100.00 % | 100.00 % | 100.00 % |

## racine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| ast.rs | 100.00 % | 100.00 % | 100.00 % |
| column_type.rs | 98.42 % | 91.67 % | 98.32 % |
| extend.rs | 91.67 % | 87.50 % | 97.92 % |
| transform_name.rs | 100.00 % | 100.00 % | 100.00 % |
| types.rs | 93.57 % | 100.00 % | 92.35 % |
