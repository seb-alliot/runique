# Couverture de tests — package `runique`

Snapshot du **2026-11-12** · commande : `cargo llvm-cov --package runique --features all-databases,admin,test-utils`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **92.14 %** | **92.77 %** | **92.93 %** |

Évolution depuis le 2026-11-07 : régions 91.68 % → 92.14 %, fonctions 92.65 % → 92.77 %, lignes 92.57 % → 92.93 %.

Depuis ce snapshot, `migration/` est réorganisé (`schema/`, `parse/`, `diff.rs`, `changes/`, `generate/`, `cli_migration/`), `cli/` ne contient plus que des appels, et `new_project/` et `auth/create_superuser.rs` portent la logique des commandes correspondantes.

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/admin_builder.rs | 98.13 % | 100.00 % | 97.96 % |
| builder/bulk_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/confirm_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/form.rs | 98.46 % | 94.29 % | 98.96 % |
| builder/list.rs | 99.29 % | 100.00 % | 99.40 % |
| builder/registered.rs | 96.48 % | 94.83 % | 96.67 % |
| builder/site.rs | 98.06 % | 100.00 % | 99.14 % |
| config/config_admin.rs | 97.73 % | 94.44 % | 97.22 % |
| middleware/admin_middleware.rs | 92.89 % | 96.72 % | 93.19 % |
| router/account/memberships.rs | 76.83 % | 87.50 % | 82.00 % |
| router/account/reset_password.rs | 94.59 % | 100.00 % | 93.81 % |
| router/account/summary.rs | 95.59 % | 100.00 % | 100.00 % |
| router/admin_router.rs | 87.63 % | 82.05 % | 88.48 % |
| router/context.rs | 97.76 % | 97.14 % | 97.19 % |
| router/crud/bulk.rs | 89.39 % | 100.00 % | 94.44 % |
| router/crud/create.rs | 91.00 % | 100.00 % | 93.33 % |
| router/crud/delete.rs | 95.77 % | 100.00 % | 95.35 % |
| router/crud/detail.rs | 91.77 % | 100.00 % | 83.06 % |
| router/crud/edit.rs | 96.43 % | 100.00 % | 96.00 % |
| router/crud/list.rs | 91.13 % | 86.67 % | 87.89 % |
| router/history.rs | 90.21 % | 95.83 % | 95.12 % |
| router/mod.rs | 97.14 % | 100.00 % | 98.13 % |
| router/path/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| router/permission/rank.rs | 86.21 % | 100.00 % | 86.96 % |
| router/permission/rights.rs | 89.11 % | 97.67 % | 93.32 % |
| router/permission/transfer.rs | 86.45 % | 92.31 % | 88.80 % |
| template/assemble.rs | 97.05 % | 97.87 % | 96.83 % |
| template/block.rs | 100.00 % | 100.00 % | 100.00 % |
| template/page.rs | 100.00 % | 100.00 % | 100.00 % |
| template/stylesheet.rs | 99.78 % | 100.00 % | 100.00 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 88.31 % | 87.10 % | 89.95 % |
| builder/mod.rs | 76.55 % | 77.78 % | 77.08 % |
| error_build.rs | 85.93 % | 100.00 % | 95.65 % |
| runique_app.rs | 5.80 % | 14.29 % | 7.89 % |
| staging/admin_staging.rs | 92.63 % | 91.84 % | 92.93 % |
| staging/core_staging.rs | 80.26 % | 90.00 % | 83.05 % |
| staging/cors_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 89.27 % | 96.00 % | 89.18 % |
| staging/middleware_staging/mod.rs | 80.54 % | 72.00 % | 82.29 % |
| staging/permissions_policy_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/static_staging.rs | 72.22 % | 75.00 % | 76.19 % |
| staging/trusted_proxies_config.rs | 97.38 % | 100.00 % | 99.10 % |
| templates.rs | 95.59 % | 96.30 % | 95.67 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| create_superuser.rs | 63.75 % | 73.68 % | 65.69 % |
| current_user.rs | 100.00 % | 100.00 % | 100.00 % |
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/change.rs | 99.16 % | 96.92 % | 98.42 % |
| gate/check.rs | 97.80 % | 98.08 % | 98.70 % |
| gate/decide.rs | 98.98 % | 100.00 % | 99.11 % |
| gate/membership.rs | 93.16 % | 100.00 % | 97.44 % |
| gate/proof.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/rank.rs | 96.78 % | 100.00 % | 98.17 % |
| gate/refusal.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/rights_grid.rs | 95.28 % | 100.00 % | 99.13 % |
| gate/save.rs | 97.77 % | 98.54 % | 98.81 % |
| gate/test_support.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/transfer.rs | 97.25 % | 100.00 % | 99.33 % |
| gate/write.rs | 94.95 % | 95.65 % | 96.16 % |
| guard.rs | 88.80 % | 96.15 % | 91.72 % |
| history.rs | 96.13 % | 100.00 % | 94.96 % |
| meta.rs | 100.00 % | 100.00 % | 100.00 % |
| migrations_table.rs | 88.69 % | 50.00 % | 89.28 % |
| password.rs | 85.75 % | 98.08 % | 87.21 % |
| permissions/groupe.rs | 81.40 % | 50.00 % | 75.00 % |
| permissions/groupes_droits.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/mod.rs | 96.33 % | 100.00 % | 96.16 % |
| permissions/rang.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/rights.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/users_groupes.rs | 100.00 % | 100.00 % | 100.00 % |
| session.rs | 73.60 % | 67.39 % | 71.15 % |
| user.rs | 88.25 % | 86.49 % | 92.89 % |
| user_trait.rs | 100.00 % | 100.00 % | 100.00 % |

---

## bin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| runique.rs | 0.00 % | 0.00 % | 0.00 % |

---

## cli

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| create_superuser.rs | 0.00 % | 0.00 % | 0.00 % |
| makemigration.rs | 0.00 % | 0.00 % | 0.00 % |
| migrate.rs | 31.25 % | 33.33 % | 33.33 % |
| new_project.rs | 0.00 % | 0.00 % | 0.00 % |
| test_runner.rs | 73.96 % | 86.30 % | 77.78 % |

---

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| security.rs | 98.51 % | 93.75 % | 99.08 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 97.45 % | 92.59 % | 98.04 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 92.32 % | 87.65 % | 90.89 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 93.33 % | 96.88 % | 91.10 % |
| tera/static_tera.rs | 97.60 % | 97.06 % | 98.22 % |
| tera/url.rs | 95.83 % | 100.00 % | 97.87 % |

---

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| adb.rs | 73.91 % | 70.59 % | 73.81 % |
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 88.18 % | 100.00 % | 90.24 % |
| engine.rs | 100.00 % | 100.00 % | 100.00 % |
| runique_db.rs | 70.37 % | 60.00 % | 65.22 % |
| sql_dialect.rs | 93.08 % | 92.59 % | 94.25 % |
| write.rs | 100.00 % | 100.00 % | 100.00 % |

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 92.47 % | 86.67 % | 95.00 % |

---

## errors

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| error.rs | 83.52 % | 96.43 % | 91.46 % |

---

## flash

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| flash_manager.rs | 100.00 % | 100.00 % | 100.00 % |
| flash_struct.rs | 100.00 % | 100.00 % | 100.00 % |

---

## forms

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| base.rs | 97.43 % | 96.92 % | 96.89 % |
| column_value.rs | 93.17 % | 82.14 % | 95.43 % |
| extractor.rs | 98.49 % | 96.67 % | 96.12 % |
| field.rs | 76.22 % | 82.35 % | 77.14 % |
| fields/binary.rs | 96.28 % | 93.75 % | 93.99 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 90.55 % | 92.00 % | 86.87 % |
| fields/datetime.rs | 79.20 % | 87.30 % | 80.13 % |
| fields/file.rs | 97.13 % | 94.59 % | 96.54 % |
| fields/hidden.rs | 88.65 % | 84.00 % | 85.96 % |
| fields/media.rs | 94.30 % | 93.07 % | 93.07 % |
| fields/number.rs | 93.43 % | 97.78 % | 93.46 % |
| fields/special.rs | 84.64 % | 85.94 % | 81.58 % |
| fields/text.rs | 94.53 % | 94.87 % | 95.86 % |
| form.rs | 85.94 % | 89.16 % | 89.28 % |
| form_data.rs | 85.29 % | 66.67 % | 85.71 % |
| generic.rs | 91.06 % | 100.00 % | 97.83 % |
| mod.rs | 100.00 % | 100.00 % | 100.00 % |
| model_form/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| options/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/aegis.rs | 87.07 % | 83.33 % | 87.96 % |
| renderer.rs | 84.15 % | 84.62 % | 91.43 % |
| save.rs | 89.36 % | 80.00 % | 84.00 % |
| validation_form.rs | 92.05 % | 92.31 % | 93.12 % |
| validator.rs | 86.40 % | 100.00 % | 89.47 % |

---

## macros

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| bdd/list.rs | 89.08 % | 77.78 % | 78.87 % |
| bdd/objects.rs | 84.20 % | 89.47 % | 89.88 % |
| bdd/query.rs | 83.89 % | 86.27 % | 88.53 % |
| context/flash.rs | 100.00 % | 100.00 % | 100.00 % |
| context/helper.rs | 100.00 % | 100.00 % | 100.00 % |
| context/impl_error.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/enum_kind.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/impl_form.rs | 100.00 % | 100.00 % | 100.00 % |
| routeur/register_url.rs | 87.02 % | 64.71 % | 89.66 % |
| routeur/router_ext.rs | 100.00 % | 100.00 % | 100.00 % |
| routeur/routes.rs | 95.79 % | 91.67 % | 94.67 % |

---

## middleware

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| config.rs | 100.00 % | 100.00 % | 100.00 % |
| dev/cache.rs | 100.00 % | 100.00 % | 100.00 % |
| errors/error.rs | 90.11 % | 92.68 % | 90.56 % |
| security/allowed_hosts.rs | 95.90 % | 94.74 % | 96.97 % |
| security/anti_bot.rs | 96.77 % | 100.00 % | 97.56 % |
| security/csp.rs | 95.13 % | 90.00 % | 98.10 % |
| security/csrf.rs | 94.54 % | 88.89 % | 97.10 % |
| security/open_redirect.rs | 97.49 % | 100.00 % | 96.77 % |
| security/permissions_policy.rs | 100.00 % | 100.00 % | 100.00 % |
| security/rate_limit.rs | 92.43 % | 90.62 % | 92.89 % |
| security/trusted_proxies.rs | 99.50 % | 100.00 % | 99.56 % |
| session/cleaning_store.rs | 90.84 % | 94.23 % | 91.29 % |
| session/session_db.rs | 91.67 % | 96.00 % | 93.26 % |

---

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| changes/destructive_changes.rs | 97.92 % | 96.97 % | 97.89 % |
| changes/name_length_limit.rs | 100.00 % | 100.00 % | 100.00 % |
| changes/ordering.rs | 98.91 % | 100.00 % | 100.00 % |
| cli_migration/init_migrate.rs | 91.28 % | 100.00 % | 91.60 % |
| cli_migration/lib_file.rs | 95.34 % | 100.00 % | 96.30 % |
| cli_migration/migrate.rs | 31.51 % | 44.44 % | 32.56 % |
| cli_migration/mod.rs | 90.52 % | 50.00 % | 96.00 % |
| cli_migration/plan.rs | 78.59 % | 91.67 % | 79.15 % |
| cli_migration/report.rs | 64.18 % | 100.00 % | 71.43 % |
| cli_migration/scan.rs | 92.49 % | 77.78 % | 93.81 % |
| cli_migration/snapshot.rs | 92.39 % | 100.00 % | 100.00 % |
| diff.rs | 95.90 % | 100.00 % | 97.70 % |
| generate/alter.rs | 94.26 % | 100.00 % | 97.45 % |
| generate/columns.rs | 100.00 % | 100.00 % | 100.00 % |
| generate/constraints.rs | 100.00 % | 100.00 % | 100.00 % |
| generate/create.rs | 100.00 % | 100.00 % | 100.00 % |
| generate/enums.rs | 100.00 % | 100.00 % | 100.00 % |
| generate/mod.rs | 99.32 % | 100.00 % | 99.34 % |
| helpers.rs | 100.00 % | 100.00 % | 100.00 % |
| parse/builder/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| parse/builder/to_schema.rs | 99.11 % | 100.00 % | 98.81 % |
| parse/col_types.rs | 90.88 % | 100.00 % | 88.24 % |
| parse/extend.rs | 91.89 % | 75.00 % | 95.45 % |
| parse/seaorm.rs | 86.80 % | 78.57 % | 89.43 % |
| parse/syn_extract.rs | 88.43 % | 100.00 % | 88.04 % |
| paths.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/column.rs | 73.35 % | 85.00 % | 84.84 % |
| schema/foreign_key.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/hooks.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/index.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/mod.rs | 95.83 % | 95.56 % | 95.58 % |
| schema/primary_key.rs | 100.00 % | 100.00 % | 100.00 % |
| tests_pipeline.rs | 99.94 % | 99.08 % | 99.91 % |
| types.rs | 100.00 % | 100.00 % | 100.00 % |

---

## new_project

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| mod.rs | 75.65 % | 89.47 % | 84.23 % |

---

## runique_test

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| logic/builder_test.rs | 88.00 % | 85.37 % | 88.63 % |
| logic/expect_test.rs | 80.95 % | 100.00 % | 92.86 % |
| logic/struct_test.rs | 97.40 % | 100.00 % | 98.10 % |
| logic/transaction_test.rs | 91.67 % | 81.25 % | 94.20 % |

---

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| aliases/helpers.rs | 69.23 % | 66.67 % | 66.67 % |
| config/env.rs | 98.55 % | 96.55 % | 97.70 % |
| config/integrity.rs | 95.45 % | 100.00 % | 100.00 % |
| config/runique_log/admin.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/auth.rs | 85.71 % | 80.00 % | 85.00 % |
| config/runique_log/builder.rs | 90.32 % | 85.71 % | 90.00 % |
| config/runique_log/db.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/errors.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/forms.rs | 90.32 % | 85.71 % | 90.00 % |
| config/runique_log/mailer.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/middleware.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/migration.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/mod.rs | 87.22 % | 77.59 % | 86.80 % |
| config/runique_log/output.rs | 81.25 % | 87.50 % | 87.84 % |
| config/runique_log/session.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/templates.rs | 100.00 % | 100.00 % | 100.00 % |
| config/trace_ext.rs | 90.60 % | 100.00 % | 93.67 % |
| config/url_params.rs | 100.00 % | 100.00 % | 100.00 % |
| constante/parse.rs | 100.00 % | 100.00 % | 100.00 % |
| constante/regex_template.rs | 100.00 % | 100.00 % | 100.00 % |
| constante/template.rs | 100.00 % | 100.00 % | 100.00 % |
| crypto/csp_nonce.rs | 100.00 % | 100.00 % | 100.00 % |
| crypto/csrf.rs | 100.00 % | 100.00 % | 100.00 % |
| crypto/session_cipher.rs | 98.84 % | 100.00 % | 100.00 % |
| forms/parse_boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_html.rs | 94.01 % | 95.80 % | 91.45 % |
| forms/sanitizer.rs | 98.92 % | 100.00 % | 99.35 % |
| init_error/init.rs | 100.00 % | 100.00 % | 100.00 % |
| mailer/mod.rs | 83.52 % | 86.54 % | 87.01 % |
| password/mod.rs | 88.05 % | 89.29 % | 91.88 % |
| private_file.rs | 95.89 % | 100.00 % | 100.00 % |
| reset_token/entity.rs | 92.86 % | 100.00 % | 100.00 % |
| reset_token/mod.rs | 87.13 % | 100.00 % | 86.60 % |
| resolve_ogimage/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| trad/switch_lang.rs | 87.11 % | 64.86 % | 85.06 % |
