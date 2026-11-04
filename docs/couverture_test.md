# Couverture de tests — package `runique`

Snapshot du **2026-11-04** · commande : `cargo llvm-cov --package runique --features all-databases,admin,test-utils`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **91.38 %** | **92.40 %** | **92.33 %** |

Évolution depuis le 2026-10-29 : régions 91.19 % → 91.38 %, fonctions 92.51 % → 92.40 %, lignes 92.21 % → 92.33 %.

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/admin_builder.rs | 97.30 % | 98.56 % | 97.31 % |
| builder/bulk_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/confirm_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/form.rs | 98.46 % | 94.29 % | 98.96 % |
| builder/list.rs | 99.10 % | 100.00 % | 99.25 % |
| builder/registered.rs | 96.39 % | 94.64 % | 96.59 % |
| builder/site.rs | 97.93 % | 100.00 % | 99.10 % |
| config/config_admin.rs | 97.90 % | 94.74 % | 97.32 % |
| middleware/admin_middleware.rs | 92.53 % | 96.61 % | 92.97 % |
| router/account/memberships.rs | 76.83 % | 87.50 % | 82.00 % |
| router/account/reset_password.rs | 94.59 % | 100.00 % | 93.81 % |
| router/admin_router.rs | 64.40 % | 74.36 % | 72.62 % |
| router/context.rs | 66.67 % | 66.67 % | 70.59 % |
| router/crud/bulk.rs | 89.39 % | 100.00 % | 94.44 % |
| router/crud/create.rs | 91.00 % | 100.00 % | 93.33 % |
| router/crud/delete.rs | 95.77 % | 100.00 % | 95.35 % |
| router/crud/detail.rs | 91.21 % | 100.00 % | 83.23 % |
| router/crud/edit.rs | 96.43 % | 100.00 % | 96.00 % |
| router/crud/list.rs | 91.92 % | 100.00 % | 84.67 % |
| router/history.rs | 90.24 % | 95.83 % | 95.12 % |
| router/mod.rs | 97.55 % | 100.00 % | 98.96 % |
| router/path/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| router/permission/rank.rs | 86.21 % | 100.00 % | 86.96 % |
| router/permission/rights.rs | 89.11 % | 97.67 % | 93.32 % |
| router/permission/transfer.rs | 86.45 % | 92.31 % | 88.80 % |
| template/assemble.rs | 99.03 % | 100.00 % | 99.41 % |
| template/block.rs | 100.00 % | 100.00 % | 100.00 % |
| template/page.rs | 100.00 % | 100.00 % | 100.00 % |
| template/stylesheet.rs | 99.78 % | 100.00 % | 100.00 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 88.16 % | 89.66 % | 89.58 % |
| builder/mod.rs | 76.55 % | 77.78 % | 77.08 % |
| error_build.rs | 85.93 % | 100.00 % | 95.65 % |
| runique_app.rs | 5.80 % | 14.29 % | 7.89 % |
| staging/admin_staging.rs | 94.15 % | 93.75 % | 93.96 % |
| staging/core_staging.rs | 80.26 % | 90.00 % | 83.05 % |
| staging/cors_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 88.81 % | 96.00 % | 88.63 % |
| staging/middleware_staging/mod.rs | 80.00 % | 72.00 % | 81.91 % |
| staging/permissions_policy_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/static_staging.rs | 72.22 % | 75.00 % | 76.19 % |
| staging/trusted_proxies_config.rs | 97.38 % | 100.00 % | 99.10 % |
| templates.rs | 94.38 % | 95.45 % | 94.52 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| current_user.rs | 100.00 % | 100.00 % | 100.00 % |
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/change.rs | 98.80 % | 96.15 % | 97.79 % |
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
| guard.rs | 88.67 % | 96.15 % | 91.67 % |
| history.rs | 96.13 % | 100.00 % | 94.96 % |
| meta.rs | 100.00 % | 100.00 % | 100.00 % |
| migrations_table.rs | 88.63 % | 50.00 % | 89.13 % |
| password.rs | 85.55 % | 97.87 % | 86.83 % |
| permissions/groupe.rs | 81.40 % | 50.00 % | 75.00 % |
| permissions/groupes_droits.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/mod.rs | 96.15 % | 100.00 % | 96.16 % |
| permissions/rang.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/rights.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/users_groupes.rs | 100.00 % | 100.00 % | 100.00 % |
| session.rs | 73.64 % | 80.00 % | 75.14 % |
| user.rs | 87.38 % | 85.29 % | 92.04 % |
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
| cli_admin.rs | 63.75 % | 73.68 % | 65.69 % |
| makemigration.rs | 92.23 % | 94.62 % | 93.13 % |
| migrate.rs | 31.51 % | 44.44 % | 32.56 % |
| migration_crate.rs | 91.28 % | 100.00 % | 91.60 % |
| new_project.rs | 69.38 % | 83.33 % | 81.64 % |
| test_runner.rs | 73.96 % | 86.30 % | 77.78 % |

---

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| security.rs | 98.51 % | 93.75 % | 99.08 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 95.95 % | 92.00 % | 97.64 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 92.18 % | 87.65 % | 90.79 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 93.33 % | 96.88 % | 91.10 % |
| tera/static_tera.rs | 96.43 % | 95.65 % | 97.27 % |
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
| sql_dialect.rs | 89.19 % | 88.89 % | 90.20 % |
| write.rs | 50.00 % | 50.00 % | 50.00 % |

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 73.03 % | 66.67 % | 72.88 % |

---

## errors

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| error.rs | 84.77 % | 100.00 % | 92.33 % |

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
| base.rs | 96.62 % | 93.75 % | 96.06 % |
| column_value.rs | 92.86 % | 82.14 % | 95.43 % |
| extractor.rs | 98.49 % | 96.67 % | 96.12 % |
| field.rs | 73.87 % | 80.00 % | 72.73 % |
| fields/binary.rs | 96.48 % | 94.00 % | 94.04 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 90.55 % | 92.00 % | 86.87 % |
| fields/datetime.rs | 79.20 % | 87.30 % | 80.13 % |
| fields/file.rs | 97.19 % | 94.89 % | 96.52 % |
| fields/hidden.rs | 88.65 % | 84.00 % | 85.96 % |
| fields/media.rs | 89.45 % | 83.75 % | 87.46 % |
| fields/number.rs | 90.82 % | 92.86 % | 91.74 % |
| fields/special.rs | 82.90 % | 84.38 % | 79.32 % |
| fields/text.rs | 92.59 % | 92.11 % | 94.94 % |
| form.rs | 85.61 % | 89.02 % | 88.98 % |
| form_data.rs | 85.29 % | 66.67 % | 85.71 % |
| generic.rs | 90.50 % | 100.00 % | 97.83 % |
| model_form/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| options/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/aegis.rs | 86.55 % | 80.00 % | 86.96 % |
| renderer.rs | 84.15 % | 84.62 % | 91.43 % |
| save.rs | 89.36 % | 80.00 % | 84.00 % |
| validation_form.rs | 88.37 % | 89.47 % | 88.80 % |
| validator.rs | 86.40 % | 100.00 % | 89.47 % |

---

## macros

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| bdd/list.rs | 89.08 % | 77.78 % | 78.87 % |
| bdd/objects.rs | 78.13 % | 89.47 % | 87.06 % |
| bdd/query.rs | 79.89 % | 86.27 % | 84.91 % |
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
| errors/error.rs | 91.19 % | 94.00 % | 92.37 % |
| security/allowed_hosts.rs | 91.42 % | 89.47 % | 90.91 % |
| security/anti_bot.rs | 96.77 % | 100.00 % | 97.56 % |
| security/csp.rs | 95.13 % | 90.00 % | 98.10 % |
| security/csrf.rs | 94.54 % | 88.89 % | 97.10 % |
| security/open_redirect.rs | 97.49 % | 100.00 % | 96.77 % |
| security/permissions_policy.rs | 100.00 % | 100.00 % | 100.00 % |
| security/rate_limit.rs | 92.43 % | 90.62 % | 92.89 % |
| security/trusted_proxies.rs | 99.45 % | 100.00 % | 99.52 % |
| session/cleaning_store.rs | 90.84 % | 94.23 % | 91.29 % |
| session/session_db.rs | 91.67 % | 96.00 % | 93.26 % |

---

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| column/mod.rs | 73.35 % | 85.00 % | 84.84 % |
| foreign_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| hooks/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| index/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| primary_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/mod.rs | 95.83 % | 95.56 % | 95.58 % |
| utils/diff.rs | 95.90 % | 100.00 % | 97.70 % |
| utils/generators.rs | 98.40 % | 100.00 % | 99.13 % |
| utils/helpers.rs | 91.30 % | 100.00 % | 89.83 % |
| utils/parser_builder/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| utils/parser_builder/to_schema.rs | 99.11 % | 100.00 % | 98.81 % |
| utils/parser_extend.rs | 91.89 % | 75.00 % | 95.45 % |
| utils/parser_seaorm.rs | 86.80 % | 78.57 % | 89.43 % |
| utils/paths.rs | 100.00 % | 100.00 % | 100.00 % |
| utils/tests_pipeline.rs | 99.94 % | 99.08 % | 99.91 % |
| utils/types.rs | 100.00 % | 100.00 % | 100.00 % |

---

## runique_test

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| logic/builder_test.rs | 88.24 % | 85.37 % | 88.63 % |
| logic/expect_test.rs | 80.95 % | 100.00 % | 92.86 % |
| logic/struct_test.rs | 97.40 % | 100.00 % | 98.10 % |
| logic/transaction_test.rs | 91.67 % | 81.25 % | 94.20 % |

---

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| aliases/helpers.rs | 69.23 % | 66.67 % | 66.67 % |
| config/env.rs | 96.12 % | 89.47 % | 94.90 % |
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
| crypto/session_cipher.rs | 91.89 % | 80.00 % | 93.18 % |
| forms/parse_boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_html.rs | 93.95 % | 94.96 % | 90.79 % |
| forms/sanitizer.rs | 98.88 % | 100.00 % | 99.33 % |
| init_error/init.rs | 100.00 % | 100.00 % | 100.00 % |
| mailer/mod.rs | 83.52 % | 86.54 % | 87.01 % |
| password/mod.rs | 86.80 % | 88.24 % | 90.29 % |
| reset_token/entity.rs | 0.00 % | 0.00 % | 0.00 % |
| reset_token/mod.rs | 87.13 % | 100.00 % | 86.60 % |
| resolve_ogimage/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| trad/switch_lang.rs | 87.11 % | 64.86 % | 85.06 % |
