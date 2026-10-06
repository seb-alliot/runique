# Couverture de tests — package `runique`

Snapshot du **2026-10-06** · commande : `cargo llvm-cov --package runique --features all-databases`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **81.20 %** | **83.91 %** | **82.27 %** |

Évolution depuis le 2026-09-24 : régions 71.95 % → 81.20 %, fonctions 75.83 % → 83.91 %, lignes 73.26 % → 82.27 %.

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| admin_main/action.rs | 98.29 % | 100.00 % | 99.51 % |
| admin_main/gate.rs | 94.46 % | 89.58 % | 95.12 % |
| admin_main/handle_bulk.rs | 69.88 % | 77.78 % | 71.30 % |
| admin_main/handle_crud.rs | 76.48 % | 71.15 % | 83.57 % |
| admin_main/handle_inline.rs | 81.82 % | 100.00 % | 80.72 % |
| admin_main/handle_list.rs | 58.41 % | 52.00 % | 65.95 % |
| admin_main/handle_password.rs | 56.89 % | 84.62 % | 56.61 % |
| admin_main/mod.rs | 89.76 % | 90.83 % | 90.12 % |
| builtin/droit.rs | 78.37 % | 73.33 % | 78.53 % |
| builtin/groupe.rs | 74.07 % | 83.33 % | 82.27 % |
| builtin/mod.rs | 98.44 % | 100.00 % | 100.00 % |
| builtin/user.rs | 76.43 % | 82.61 % | 79.71 % |
| config/config_admin.rs | 73.28 % | 64.71 % | 76.47 % |
| daemon/generator.rs | 41.88 % | 24.39 % | 37.06 % |
| daemon/parser.rs | 92.22 % | 98.39 % | 97.90 % |
| daemon/watcher.rs | 0.00 % | 0.00 % | 0.00 % |
| forms/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| helper/fk_resolve.rs | 54.94 % | 56.25 % | 61.68 % |
| helper/m2m.rs | 96.15 % | 100.00 % | 100.00 % |
| helper/resource_entry.rs | 68.70 % | 65.00 % | 71.17 % |
| helper/sql_dialect.rs | 61.76 % | 75.00 % | 72.22 % |
| helper/template.rs | 54.55 % | 60.87 % | 70.00 % |
| history.rs | 98.08 % | 100.00 % | 98.41 % |
| middleware/admin_middleware.rs | 89.08 % | 85.71 % | 92.31 % |
| mod.rs | 36.36 % | 50.00 % | 60.00 % |
| registry.rs | 99.10 % | 100.00 % | 100.00 % |
| resource.rs | 56.30 % | 54.55 % | 65.52 % |
| router/admin_router.rs | 73.08 % | 67.12 % | 77.01 % |
| table_admin/migrations_table.rs | 49.18 % | 22.73 % | 55.31 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 84.76 % | 75.00 % | 85.27 % |
| builder/mod.rs | 78.69 % | 77.78 % | 77.42 % |
| error_build.rs | 84.44 % | 100.00 % | 94.57 % |
| runique_app.rs | 6.06 % | 14.29 % | 8.11 % |
| staging/admin_staging.rs | 92.27 % | 95.65 % | 92.67 % |
| staging/core_staging.rs | 80.26 % | 90.00 % | 83.05 % |
| staging/cors_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 85.65 % | 92.00 % | 85.38 % |
| staging/middleware_staging/mod.rs | 78.26 % | 70.83 % | 82.10 % |
| staging/permissions_policy_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/static_staging.rs | 72.22 % | 75.00 % | 76.19 % |
| staging/trusted_proxies_config.rs | 96.05 % | 100.00 % | 98.88 % |
| templates.rs | 83.27 % | 85.71 % | 82.31 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| form.rs | 0.00 % | 0.00 % | 0.00 % |
| guard.rs | 85.31 % | 93.75 % | 90.32 % |
| password.rs | 72.19 % | 92.11 % | 70.48 % |
| permissions/groupe.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/groupes_droits.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/mod.rs | 86.67 % | 90.00 % | 88.64 % |
| permissions/users_groupes.rs | 100.00 % | 100.00 % | 100.00 % |
| session.rs | 88.09 % | 96.97 % | 89.40 % |
| user.rs | 89.75 % | 87.50 % | 93.23 % |
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
| cli_admin.rs | 33.13 % | 37.50 % | 33.67 % |
| makemigration.rs | 85.14 % | 90.11 % | 85.47 % |
| migrate.rs | 79.31 % | 100.00 % | 78.95 % |
| new_project.rs | 27.63 % | 37.50 % | 14.48 % |
| start.rs | 23.08 % | 37.50 % | 24.53 % |
| test_runner.rs | 73.96 % | 86.30 % | 77.78 % |

---

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| security.rs | 98.72 % | 94.74 % | 99.18 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 96.27 % | 87.50 % | 97.48 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 79.37 % | 67.65 % | 80.53 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 82.87 % | 89.66 % | 81.40 % |
| tera/static_tera.rs | 96.00 % | 94.74 % | 97.06 % |
| tera/url.rs | 91.84 % | 85.71 % | 93.88 % |

---

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| adb.rs | 69.23 % | 66.67 % | 69.44 % |
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 87.10 % | 100.00 % | 89.47 % |
| engine.rs | 96.77 % | 100.00 % | 75.00 % |

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 86.90 % | 76.92 % | 86.54 % |

---

## errors

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| error.rs | 83.91 % | 100.00 % | 91.79 % |

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
| base.rs | 92.24 % | 86.36 % | 88.69 % |
| extractor.rs | 96.63 % | 91.30 % | 95.00 % |
| field.rs | 73.64 % | 70.18 % | 73.02 % |
| fields/binary.rs | 90.76 % | 81.82 % | 87.96 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 91.15 % | 89.36 % | 87.43 % |
| fields/datetime.rs | 74.19 % | 88.14 % | 77.34 % |
| fields/file.rs | 95.28 % | 96.55 % | 96.13 % |
| fields/hidden.rs | 62.82 % | 60.00 % | 61.54 % |
| fields/number.rs | 85.84 % | 94.12 % | 89.07 % |
| fields/special.rs | 87.94 % | 89.83 % | 87.71 % |
| fields/text.rs | 79.25 % | 77.14 % | 83.33 % |
| form.rs | 64.93 % | 64.44 % | 67.09 % |
| form_data.rs | 50.82 % | 50.00 % | 59.46 % |
| generic.rs | 84.91 % | 86.67 % | 86.36 % |
| model_form/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| options/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/aegis.rs | 63.00 % | 62.50 % | 65.75 % |
| prisme/rules.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/sentinel.rs | 97.22 % | 100.00 % | 95.38 % |
| renderer.rs | 81.38 % | 90.00 % | 88.54 % |
| validation_form.rs | 73.33 % | 60.00 % | 68.42 % |
| validator.rs | 85.60 % | 100.00 % | 88.16 % |

---

## macros

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| bdd/list.rs | 89.08 % | 77.78 % | 78.87 % |
| bdd/objects.rs | 78.13 % | 89.47 % | 87.06 % |
| bdd/query.rs | 76.13 % | 81.25 % | 79.83 % |
| context/flash.rs | 100.00 % | 100.00 % | 100.00 % |
| context/helper.rs | 83.33 % | 71.43 % | 79.31 % |
| context/impl_error.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/enum_kind.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/impl_form.rs | 65.38 % | 57.14 % | 60.87 % |
| routeur/register_url.rs | 86.11 % | 58.33 % | 89.13 % |
| routeur/router_ext.rs | 96.20 % | 100.00 % | 98.28 % |

---

## middleware

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| config.rs | 100.00 % | 100.00 % | 100.00 % |
| dev/cache.rs | 100.00 % | 100.00 % | 100.00 % |
| errors/error.rs | 70.81 % | 80.95 % | 73.46 % |
| security/allowed_hosts.rs | 76.95 % | 68.42 % | 68.94 % |
| security/anti_bot.rs | 59.68 % | 75.00 % | 53.66 % |
| security/csp.rs | 94.87 % | 90.00 % | 98.10 % |
| security/csrf.rs | 80.75 % | 80.00 % | 85.48 % |
| security/open_redirect.rs | 95.88 % | 100.00 % | 95.41 % |
| security/permissions_policy.rs | 100.00 % | 100.00 % | 100.00 % |
| security/rate_limit.rs | 87.54 % | 87.50 % | 89.24 % |
| security/trusted_proxies.rs | 93.70 % | 100.00 % | 93.27 % |
| session/cleaning_store.rs | 92.99 % | 95.12 % | 95.05 % |
| session/session_db.rs | 92.50 % | 100.00 % | 91.73 % |

---

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| column/mod.rs | 78.49 % | 95.52 % | 86.16 % |
| foreign_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| hooks/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| index/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| primary_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/mod.rs | 91.18 % | 90.00 % | 91.07 % |
| utils/diff.rs | 95.57 % | 100.00 % | 97.11 % |
| utils/generators.rs | 98.54 % | 100.00 % | 99.30 % |
| utils/helpers.rs | 73.62 % | 100.00 % | 72.08 % |
| utils/parser_builder/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| utils/parser_builder/to_schema.rs | 98.48 % | 100.00 % | 97.98 % |
| utils/parser_extend.rs | 91.89 % | 75.00 % | 95.45 % |
| utils/parser_seaorm.rs | 70.21 % | 68.00 % | 71.38 % |
| utils/paths.rs | 94.29 % | 89.47 % | 91.04 % |
| utils/tests_pipeline.rs | 99.93 % | 98.98 % | 99.90 % |
| utils/types.rs | 100.00 % | 100.00 % | 100.00 % |

---

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| aliases/helpers.rs | 69.23 % | 66.67 % | 66.67 % |
| config/env.rs | 93.48 % | 87.50 % | 91.04 % |
| config/integrity.rs | 95.45 % | 100.00 % | 100.00 % |
| config/runique_log/admin.rs | 33.33 % | 37.50 % | 34.29 % |
| config/runique_log/auth.rs | 88.46 % | 83.33 % | 88.00 % |
| config/runique_log/builder.rs | 90.32 % | 85.71 % | 90.00 % |
| config/runique_log/db.rs | 50.00 % | 50.00 % | 57.14 % |
| config/runique_log/errors.rs | 50.00 % | 50.00 % | 57.14 % |
| config/runique_log/forms.rs | 90.32 % | 85.71 % | 90.00 % |
| config/runique_log/mailer.rs | 36.36 % | 33.33 % | 40.00 % |
| config/runique_log/middleware.rs | 34.78 % | 40.00 % | 35.56 % |
| config/runique_log/migration.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/mod.rs | 83.05 % | 72.55 % | 82.81 % |
| config/runique_log/output.rs | 55.36 % | 68.75 % | 67.57 % |
| config/runique_log/session.rs | 38.10 % | 40.00 % | 40.00 % |
| config/runique_log/templates.rs | 100.00 % | 100.00 % | 100.00 % |
| config/trace_ext.rs | 90.60 % | 100.00 % | 93.67 % |
| config/url_params.rs | 100.00 % | 100.00 % | 100.00 % |
| constante/parse.rs | 100.00 % | 100.00 % | 100.00 % |
| constante/regex_template.rs | 100.00 % | 100.00 % | 100.00 % |
| crypto/csp_nonce.rs | 100.00 % | 100.00 % | 100.00 % |
| crypto/csrf.rs | 99.21 % | 100.00 % | 100.00 % |
| forms/parse_boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_html.rs | 77.41 % | 77.78 % | 65.82 % |
| forms/sanitizer.rs | 95.45 % | 94.12 % | 93.84 % |
| init_error/init.rs | 100.00 % | 100.00 % | 100.00 % |
| mailer/mod.rs | 83.04 % | 86.00 % | 86.35 % |
| password/mod.rs | 86.70 % | 88.24 % | 90.25 % |
| reset_token/entity.rs | 0.00 % | 0.00 % | 0.00 % |
| reset_token/mod.rs | 98.12 % | 100.00 % | 99.07 % |
| resolve_ogimage/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| trad/switch_lang.rs | 91.10 % | 66.67 % | 88.68 % |
