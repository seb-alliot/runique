# Couverture de tests — package `runique`

Snapshot du **2026-10-22** · commande : `cargo llvm-cov --package runique --features all-databases,admin,test-utils`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **87.34 %** | **89.81 %** | **88.65 %** |

Évolution depuis le 2026-10-08 : régions 87.25 % → 87.34 %, fonctions 88.72 % → 89.81 %, lignes 88.81 % → 88.65 %.

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/admin_builder.rs | 97.89 % | 99.12 % | 97.58 % |
| builder/form.rs | 99.68 % | 100.00 % | 100.00 % |
| builder/list.rs | 99.08 % | 100.00 % | 99.23 % |
| builder/registered.rs | 96.97 % | 93.75 % | 95.77 % |
| builder/site.rs | 98.26 % | 100.00 % | 100.00 % |
| config/config_admin.rs | 97.92 % | 94.74 % | 97.32 % |
| config/template.rs | 54.55 % | 60.87 % | 70.00 % |
| gate/check.rs | 96.92 % | 98.08 % | 98.33 % |
| gate/decide.rs | 98.98 % | 100.00 % | 99.11 % |
| gate/proof.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/refusal.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/test_support.rs | 100.00 % | 100.00 % | 100.00 % |
| gate/write.rs | 96.97 % | 96.30 % | 97.40 % |
| history.rs | 0.00 % | 0.00 % | 0.00 % |
| meta.rs | 100.00 % | 100.00 % | 100.00 % |
| middleware/admin_middleware.rs | 18.03 % | 66.67 % | 21.88 % |
| router/admin_router.rs | 22.70 % | 13.79 % | 21.88 % |
| router/detail.rs | 0.00 % | 0.00 % | 0.00 % |
| router/list.rs | 0.00 % | 0.00 % | 0.00 % |
| router/mod.rs | 42.61 % | 58.33 % | 44.59 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 86.29 % | 85.00 % | 88.15 % |
| builder/mod.rs | 73.77 % | 72.22 % | 70.63 % |
| error_build.rs | 84.44 % | 100.00 % | 94.57 % |
| runique_app.rs | 5.80 % | 14.29 % | 7.89 % |
| staging/admin_staging.rs | 84.66 % | 95.65 % | 82.78 % |
| staging/core_staging.rs | 80.26 % | 90.00 % | 83.05 % |
| staging/cors_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 88.78 % | 96.00 % | 88.54 % |
| staging/middleware_staging/mod.rs | 80.00 % | 72.00 % | 81.91 % |
| staging/permissions_policy_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/static_staging.rs | 72.22 % | 75.00 % | 76.19 % |
| staging/trusted_proxies_config.rs | 97.38 % | 100.00 % | 99.10 % |
| templates.rs | 83.27 % | 85.71 % | 82.31 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| current_user.rs | 100.00 % | 100.00 % | 100.00 % |
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| guard.rs | 85.31 % | 93.75 % | 90.32 % |
| migrations_table.rs | 75.29 % | 46.43 % | 74.27 % |
| password.rs | 71.41 % | 92.11 % | 70.18 % |
| permissions/groupe.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/groupes_droits.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/mod.rs | 83.80 % | 82.35 % | 83.80 % |
| permissions/rang.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/rights.rs | 100.00 % | 100.00 % | 100.00 % |
| permissions/users_groupes.rs | 100.00 % | 100.00 % | 100.00 % |
| session.rs | 68.12 % | 80.95 % | 71.52 % |
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
| template.rs | 86.26 % | 77.78 % | 86.08 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 90.30 % | 87.88 % | 87.41 % |
| tera/static_tera.rs | 96.00 % | 94.74 % | 97.06 % |
| tera/url.rs | 91.00 % | 85.71 % | 93.88 % |

---

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| adb.rs | 73.91 % | 70.59 % | 73.81 % |
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 88.18 % | 100.00 % | 90.24 % |
| engine.rs | 100.00 % | 100.00 % | 100.00 % |
| runique_db.rs | 70.37 % | 60.00 % | 65.22 % |
| sql_dialect.rs | 85.88 % | 85.71 % | 87.01 % |

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
| base.rs | 94.61 % | 91.67 % | 94.62 % |
| column_value.rs | 86.47 % | 75.00 % | 89.33 % |
| extractor.rs | 98.16 % | 96.55 % | 95.92 % |
| field.rs | 66.97 % | 69.23 % | 67.61 % |
| fields/binary.rs | 94.42 % | 88.89 % | 90.34 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 90.55 % | 92.00 % | 86.87 % |
| fields/datetime.rs | 79.20 % | 87.30 % | 80.13 % |
| fields/file.rs | 95.66 % | 96.75 % | 96.11 % |
| fields/hidden.rs | 89.01 % | 88.24 % | 87.01 % |
| fields/number.rs | 90.37 % | 95.12 % | 91.70 % |
| fields/special.rs | 82.90 % | 84.38 % | 79.32 % |
| fields/text.rs | 91.36 % | 92.11 % | 94.30 % |
| form.rs | 78.38 % | 83.33 % | 83.29 % |
| form_data.rs | 85.29 % | 66.67 % | 85.71 % |
| generic.rs | 88.24 % | 97.30 % | 95.35 % |
| model_form/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| options/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/aegis.rs | 85.59 % | 80.00 % | 86.21 % |
| renderer.rs | 84.15 % | 84.62 % | 91.43 % |
| validation_form.rs | 100.00 % | 100.00 % | 100.00 % |
| validator.rs | 86.40 % | 100.00 % | 89.47 % |

---

## macros

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| bdd/list.rs | 89.08 % | 77.78 % | 78.87 % |
| bdd/objects.rs | 78.13 % | 89.47 % | 87.06 % |
| bdd/query.rs | 78.28 % | 85.42 % | 83.24 % |
| context/flash.rs | 100.00 % | 100.00 % | 100.00 % |
| context/helper.rs | 100.00 % | 100.00 % | 100.00 % |
| context/impl_error.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/enum_kind.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/impl_form.rs | 65.38 % | 57.14 % | 60.87 % |
| routeur/register_url.rs | 86.11 % | 58.33 % | 89.13 % |
| routeur/router_ext.rs | 100.00 % | 100.00 % | 100.00 % |

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
| security/rate_limit.rs | 90.04 % | 87.50 % | 90.51 % |
| security/trusted_proxies.rs | 99.45 % | 100.00 % | 99.52 % |
| session/cleaning_store.rs | 92.99 % | 95.12 % | 95.05 % |
| session/session_db.rs | 91.61 % | 100.00 % | 91.13 % |

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
| utils/diff.rs | 95.58 % | 100.00 % | 97.13 % |
| utils/generators.rs | 98.41 % | 100.00 % | 99.13 % |
| utils/helpers.rs | 91.30 % | 100.00 % | 89.83 % |
| utils/parser_builder/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| utils/parser_builder/to_schema.rs | 99.11 % | 100.00 % | 98.81 % |
| utils/parser_extend.rs | 91.89 % | 75.00 % | 95.45 % |
| utils/parser_seaorm.rs | 86.81 % | 78.57 % | 89.63 % |
| utils/paths.rs | 100.00 % | 100.00 % | 100.00 % |
| utils/tests_pipeline.rs | 99.93 % | 99.05 % | 99.90 % |
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
| trad/switch_lang.rs | 94.09 % | 72.73 % | 91.93 % |
