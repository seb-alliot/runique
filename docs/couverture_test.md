# Couverture de tests — package `runique`

Snapshot du **2026-10-08** · commande : `cargo llvm-cov --package runique --features all-databases`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **87.25 %** | **88.72 %** | **88.81 %** |

Évolution depuis le 2026-10-06 : régions 81.20 % → 87.25 %, fonctions 83.91 % → 88.72 %, lignes 82.27 % → 88.81 %.

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| admin_main/action.rs | 98.29 % | 100.00 % | 99.51 % |
| admin_main/gate.rs | 96.63 % | 92.31 % | 96.13 % |
| admin_main/handle_bulk.rs | 79.81 % | 83.33 % | 78.24 % |
| admin_main/handle_crud.rs | 83.62 % | 74.07 % | 89.61 % |
| admin_main/handle_inline.rs | 84.55 % | 100.00 % | 81.93 % |
| admin_main/handle_list.rs | 86.73 % | 84.00 % | 90.32 % |
| admin_main/handle_password.rs | 56.89 % | 84.62 % | 56.61 % |
| admin_main/mod.rs | 90.32 % | 91.74 % | 90.42 % |
| builtin/droit.rs | 89.40 % | 84.44 % | 89.79 % |
| builtin/groupe.rs | 74.49 % | 83.33 % | 82.98 % |
| builtin/mod.rs | 98.63 % | 100.00 % | 100.00 % |
| builtin/user.rs | 83.19 % | 86.96 % | 86.67 % |
| config/config_admin.rs | 97.71 % | 94.12 % | 97.06 % |
| daemon/generator.rs | 93.89 % | 88.89 % | 94.86 % |
| daemon/parser.rs | 93.70 % | 98.39 % | 99.55 % |
| daemon/watcher.rs | 0.00 % | 0.00 % | 0.00 % |
| forms/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| helper/fk_resolve.rs | 87.04 % | 93.75 % | 91.59 % |
| helper/m2m.rs | 96.15 % | 100.00 % | 100.00 % |
| helper/resource_entry.rs | 90.08 % | 85.00 % | 89.19 % |
| helper/sql_dialect.rs | 61.76 % | 75.00 % | 72.22 % |
| helper/template.rs | 84.85 % | 86.96 % | 90.00 % |
| history.rs | 98.08 % | 100.00 % | 98.41 % |
| middleware/admin_middleware.rs | 89.08 % | 85.71 % | 92.31 % |
| mod.rs | 36.36 % | 50.00 % | 60.00 % |
| registry.rs | 99.10 % | 100.00 % | 100.00 % |
| resource.rs | 60.50 % | 59.09 % | 68.97 % |
| router/admin_router.rs | 85.66 % | 87.67 % | 88.82 % |
| table_admin/migrations_table.rs | 96.72 % | 81.82 % | 97.43 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 84.76 % | 75.00 % | 85.27 % |
| builder/mod.rs | 78.69 % | 77.78 % | 77.42 % |
| error_build.rs | 84.44 % | 100.00 % | 94.57 % |
| runique_app.rs | 6.06 % | 14.29 % | 8.11 % |
| staging/admin_staging.rs | 96.13 % | 100.00 % | 95.33 % |
| staging/core_staging.rs | 80.26 % | 90.00 % | 83.05 % |
| staging/cors_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 88.76 % | 96.00 % | 88.54 % |
| staging/middleware_staging/mod.rs | 80.00 % | 72.00 % | 83.71 % |
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
| password.rs | 72.05 % | 92.11 % | 70.18 % |
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
| cli_admin.rs | 61.27 % | 71.43 % | 62.39 % |
| makemigration.rs | 92.65 % | 93.64 % | 93.48 % |
| migrate.rs | 79.31 % | 100.00 % | 78.95 % |
| new_project.rs | 27.63 % | 37.50 % | 14.48 % |
| start.rs | 23.08 % | 37.50 % | 24.53 % |
| test_runner.rs | 73.96 % | 86.30 % | 77.78 % |

---

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| security.rs | 98.51 % | 93.75 % | 99.08 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 96.27 % | 87.50 % | 97.48 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 82.19 % | 72.22 % | 82.77 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 82.87 % | 89.66 % | 81.40 % |
| tera/static_tera.rs | 96.00 % | 94.74 % | 97.06 % |
| tera/url.rs | 91.00 % | 85.71 % | 93.88 % |

---

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| adb.rs | 69.23 % | 66.67 % | 69.44 % |
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 88.18 % | 100.00 % | 90.24 % |
| engine.rs | 100.00 % | 100.00 % | 100.00 % |

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 91.67 % | 84.62 % | 92.31 % |

---

## errors

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| error.rs | 84.46 % | 100.00 % | 92.17 % |

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
| fields/file.rs | 95.89 % | 97.41 % | 96.58 % |
| fields/hidden.rs | 62.82 % | 60.00 % | 61.54 % |
| fields/number.rs | 93.15 % | 97.06 % | 95.08 % |
| fields/special.rs | 88.89 % | 89.83 % | 87.71 % |
| fields/text.rs | 86.32 % | 88.57 % | 88.64 % |
| form.rs | 66.71 % | 66.67 % | 69.87 % |
| form_data.rs | 86.89 % | 70.00 % | 86.49 % |
| generic.rs | 87.74 % | 90.00 % | 89.77 % |
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
| security/open_redirect.rs | 96.02 % | 100.00 % | 95.65 % |
| security/permissions_policy.rs | 100.00 % | 100.00 % | 100.00 % |
| security/rate_limit.rs | 90.04 % | 87.50 % | 90.51 % |
| security/trusted_proxies.rs | 99.45 % | 100.00 % | 99.52 % |
| session/cleaning_store.rs | 92.99 % | 95.12 % | 95.05 % |
| session/session_db.rs | 92.50 % | 100.00 % | 91.73 % |

---

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| column/mod.rs | 78.65 % | 95.59 % | 86.31 % |
| foreign_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| hooks/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| index/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| primary_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| schema/mod.rs | 91.18 % | 90.00 % | 91.07 % |
| utils/diff.rs | 95.57 % | 100.00 % | 97.11 % |
| utils/generators.rs | 98.42 % | 100.00 % | 99.13 % |
| utils/helpers.rs | 91.21 % | 100.00 % | 89.74 % |
| utils/parser_builder/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| utils/parser_builder/to_schema.rs | 99.09 % | 100.00 % | 98.79 % |
| utils/parser_extend.rs | 91.89 % | 75.00 % | 95.45 % |
| utils/parser_seaorm.rs | 86.88 % | 76.00 % | 89.39 % |
| utils/paths.rs | 100.00 % | 100.00 % | 100.00 % |
| utils/tests_pipeline.rs | 99.93 % | 98.98 % | 99.90 % |
| utils/types.rs | 100.00 % | 100.00 % | 100.00 % |

---

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| aliases/helpers.rs | 69.23 % | 66.67 % | 66.67 % |
| config/env.rs | 96.12 % | 89.47 % | 94.90 % |
| config/integrity.rs | 95.45 % | 100.00 % | 100.00 % |
| config/runique_log/admin.rs | 100.00 % | 100.00 % | 100.00 % |
| config/runique_log/auth.rs | 88.46 % | 83.33 % | 88.00 % |
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
| crypto/csrf.rs | 99.21 % | 100.00 % | 100.00 % |
| forms/parse_boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_html.rs | 82.49 % | 77.78 % | 74.68 % |
| forms/sanitizer.rs | 98.88 % | 100.00 % | 99.33 % |
| init_error/init.rs | 100.00 % | 100.00 % | 100.00 % |
| mailer/mod.rs | 83.04 % | 86.00 % | 86.35 % |
| password/mod.rs | 86.70 % | 88.24 % | 90.25 % |
| reset_token/entity.rs | 0.00 % | 0.00 % | 0.00 % |
| reset_token/mod.rs | 98.12 % | 100.00 % | 99.07 % |
| resolve_ogimage/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| trad/switch_lang.rs | 90.72 % | 66.67 % | 87.58 % |
