# Couverture de tests — package `runique`

Snapshot du **2026-10-29** · commande : `cargo llvm-cov --package runique --features all-databases,admin,test-utils`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **91.19 %** | **92.51 %** | **92.21 %** |

Évolution depuis le 2026-10-22 : régions 87.34 % → 91.19 %, fonctions 89.81 % → 92.51 %, lignes 88.65 % → 92.21 %.

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/admin_builder.rs | 97.28 % | 98.55 % | 97.30 % |
| builder/bulk_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/confirm_form.rs | 100.00 % | 100.00 % | 100.00 % |
| builder/form.rs | 98.46 % | 94.29 % | 98.96 % |
| builder/list.rs | 99.10 % | 100.00 % | 99.25 % |
| builder/registered.rs | 96.40 % | 94.64 % | 96.61 % |
| builder/site.rs | 97.93 % | 100.00 % | 99.10 % |
| config/config_admin.rs | 97.90 % | 94.74 % | 97.32 % |
| middleware/admin_middleware.rs | 92.53 % | 96.61 % | 92.97 % |
| router/account/memberships.rs | 77.38 % | 87.50 % | 82.35 % |
| router/account/reset_password.rs | 94.59 % | 100.00 % | 93.81 % |
| router/admin_router.rs | 64.03 % | 74.36 % | 72.45 % |
| router/context.rs | 66.67 % | 66.67 % | 70.59 % |
| router/crud/bulk.rs | 89.39 % | 100.00 % | 94.44 % |
| router/crud/create.rs | 91.00 % | 100.00 % | 93.33 % |
| router/crud/delete.rs | 95.77 % | 100.00 % | 95.35 % |
| router/crud/detail.rs | 91.21 % | 100.00 % | 83.23 % |
| router/crud/edit.rs | 96.43 % | 100.00 % | 96.00 % |
| router/crud/list.rs | 91.89 % | 100.00 % | 84.56 % |
| router/history.rs | 90.24 % | 95.83 % | 95.12 % |
| router/mod.rs | 97.55 % | 100.00 % | 98.96 % |
| router/path/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| router/permission/rank.rs | 86.44 % | 100.00 % | 87.14 % |
| router/permission/rights.rs | 89.18 % | 97.67 % | 93.35 % |
| router/permission/transfer.rs | 86.57 % | 92.31 % | 88.89 % |
| template/assemble.rs | 99.03 % | 100.00 % | 99.41 % |
| template/block.rs | 100.00 % | 100.00 % | 100.00 % |
| template/page.rs | 71.23 % | 69.23 % | 77.03 % |
| template/stylesheet.rs | 99.73 % | 100.00 % | 100.00 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 87.68 % | 88.46 % | 88.86 % |
| builder/mod.rs | 76.22 % | 77.78 % | 76.60 % |
| error_build.rs | 85.93 % | 100.00 % | 95.65 % |
| runique_app.rs | 5.80 % | 14.29 % | 7.89 % |
| staging/admin_staging.rs | 94.15 % | 93.75 % | 93.96 % |
| staging/core_staging.rs | 80.26 % | 90.00 % | 83.05 % |
| staging/cors_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 88.78 % | 96.00 % | 88.54 % |
| staging/middleware_staging/mod.rs | 80.00 % | 72.00 % | 81.91 % |
| staging/permissions_policy_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/static_staging.rs | 72.22 % | 75.00 % | 76.19 % |
| staging/trusted_proxies_config.rs | 97.38 % | 100.00 % | 99.10 % |
| templates.rs | 94.31 % | 95.45 % | 94.50 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| current_user.rs | 100.00 % | 100.00 % | 100.00 % |
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/change.rs | 98.69 % | 96.00 % | 97.62 % |
| gate/check.rs | 97.80 % | 98.08 % | 98.70 % |
| gate/decide.rs | 98.98 % | 100.00 % | 99.11 % |
| gate/membership.rs | 93.16 % | 100.00 % | 97.44 % |
| gate/proof.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/rank.rs | 96.78 % | 100.00 % | 98.17 % |
| gate/refusal.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/rights_grid.rs | 95.28 % | 100.00 % | 99.13 % |
| gate/save.rs | 97.77 % | 98.53 % | 98.80 % |
| gate/test_support.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/transfer.rs | 97.09 % | 100.00 % | 99.29 % |
| gate/write.rs | 98.35 % | 100.00 % | 99.17 % |
| guard.rs | 88.67 % | 96.15 % | 91.67 % |
| history.rs | 96.13 % | 100.00 % | 94.96 % |
| meta.rs | 100.00 % | 100.00 % | 100.00 % |
| migrations_table.rs | 81.65 % | 46.67 % | 80.85 % |
| password.rs | 83.65 % | 97.73 % | 82.21 % |
| permissions/groupe.rs | 81.40 % | 50.00 % | 75.00 % |
| permissions/groupes_droits.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/mod.rs | 96.15 % | 100.00 % | 96.16 % |
| permissions/rang.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/rights.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/users_groupes.rs | 100.00 % | 100.00 % | 100.00 % |
| session.rs | 77.73 % | 85.71 % | 79.11 % |
| user.rs | 90.87 % | 87.10 % | 93.53 % |
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
| makemigration.rs | 93.14 % | 94.12 % | 94.04 % |
| migrate.rs | 37.14 % | 100.00 % | 45.83 % |
| new_project.rs | 74.61 % | 88.89 % | 90.59 % |
| test_runner.rs | 73.96 % | 86.30 % | 77.78 % |

---

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| security.rs | 98.51 % | 93.75 % | 99.08 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 97.56 % | 91.67 % | 98.33 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 90.86 % | 87.69 % | 90.49 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 93.33 % | 96.88 % | 91.10 % |
| tera/static_tera.rs | 96.11 % | 95.00 % | 97.09 % |
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

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 91.67 % | 84.62 % | 92.31 % |

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
| base.rs | 96.59 % | 93.44 % | 96.00 % |
| column_value.rs | 92.86 % | 82.14 % | 95.43 % |
| extractor.rs | 98.45 % | 96.55 % | 95.90 % |
| field.rs | 68.00 % | 71.88 % | 69.05 % |
| fields/binary.rs | 94.42 % | 88.89 % | 90.34 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 90.55 % | 92.00 % | 86.87 % |
| fields/datetime.rs | 79.20 % | 87.30 % | 80.13 % |
| fields/file.rs | 96.14 % | 97.58 % | 96.90 % |
| fields/hidden.rs | 88.65 % | 84.00 % | 85.96 % |
| fields/number.rs | 92.36 % | 97.56 % | 92.14 % |
| fields/special.rs | 82.90 % | 84.38 % | 79.32 % |
| fields/text.rs | 92.59 % | 92.11 % | 94.94 % |
| form.rs | 81.39 % | 87.50 % | 86.76 % |
| form_data.rs | 85.29 % | 66.67 % | 85.71 % |
| generic.rs | 90.17 % | 100.00 % | 97.73 % |
| model_form/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| options/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/aegis.rs | 85.59 % | 80.00 % | 86.21 % |
| renderer.rs | 84.15 % | 84.62 % | 91.43 % |
| save.rs | 89.36 % | 80.00 % | 84.00 % |
| validation_form.rs | 87.03 % | 85.71 % | 88.54 % |
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
| routeur/routes.rs | 96.30 % | 88.89 % | 95.31 % |

---

## middleware

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| config.rs | 100.00 % | 100.00 % | 100.00 % |
| dev/cache.rs | 100.00 % | 100.00 % | 100.00 % |
| errors/error.rs | 90.52 % | 93.48 % | 91.67 % |
| security/allowed_hosts.rs | 91.42 % | 89.47 % | 90.91 % |
| security/anti_bot.rs | 96.77 % | 100.00 % | 97.56 % |
| security/csp.rs | 95.13 % | 90.00 % | 98.10 % |
| security/csrf.rs | 80.75 % | 80.00 % | 85.48 % |
| security/open_redirect.rs | 97.49 % | 100.00 % | 96.77 % |
| security/permissions_policy.rs | 100.00 % | 100.00 % | 100.00 % |
| security/rate_limit.rs | 92.43 % | 90.62 % | 92.89 % |
| security/trusted_proxies.rs | 99.45 % | 100.00 % | 99.52 % |
| session/cleaning_store.rs | 92.99 % | 95.12 % | 95.05 % |
| session/session_db.rs | 92.31 % | 100.00 % | 91.13 % |

---

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| column/mod.rs | 79.83 % | 97.14 % | 88.27 % |
| foreign_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| hooks/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| index/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| primary_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/mod.rs | 95.48 % | 95.45 % | 95.08 % |
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
| logic/builder_test.rs | 88.00 % | 85.37 % | 88.63 % |
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
| forms/parse_boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_html.rs | 85.79 % | 85.37 % | 81.77 % |
| forms/sanitizer.rs | 98.88 % | 100.00 % | 99.33 % |
| init_error/init.rs | 100.00 % | 100.00 % | 100.00 % |
| mailer/mod.rs | 83.52 % | 86.54 % | 87.01 % |
| password/mod.rs | 86.70 % | 88.24 % | 90.25 % |
| reset_token/entity.rs | 0.00 % | 0.00 % | 0.00 % |
| reset_token/mod.rs | 98.12 % | 100.00 % | 99.07 % |
| resolve_ogimage/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| trad/switch_lang.rs | 87.11 % | 64.86 % | 85.06 % |
