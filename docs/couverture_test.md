# Couverture de tests — package `runique`

Snapshot du **2026-09-24** · commande : `cargo llvm-cov --package runique --features all-databases`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **71.95 %** | **75.83 %** | **73.26 %** |

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| admin_main/action.rs | 98.29 % | 100.00 % | 99.51 % |
| admin_main/handle_bulk.rs | 29.19 % | 30.56 % | 32.88 % |
| admin_main/handle_crud.rs | 57.25 % | 50.98 % | 68.70 % |
| admin_main/handle_inline.rs | 81.82 % | 100.00 % | 80.95 % |
| admin_main/handle_list.rs | 52.89 % | 44.00 % | 62.68 % |
| admin_main/handle_password.rs | 58.46 % | 85.71 % | 56.65 % |
| admin_main/mod.rs | 71.26 % | 65.52 % | 70.05 % |
| builtin/droit.rs | 75.56 % | 69.57 % | 76.21 % |
| builtin/groupe.rs | 63.93 % | 76.92 % | 77.22 % |
| builtin/mod.rs | 27.27 % | 50.00 % | 33.33 % |
| builtin/user.rs | 65.14 % | 72.50 % | 67.72 % |
| config/config_admin.rs | 56.85 % | 38.89 % | 57.27 % |
| daemon/generator.rs | 0.00 % | 0.00 % | 0.00 % |
| daemon/parser.rs | 92.22 % | 98.39 % | 97.90 % |
| daemon/watcher.rs | 0.00 % | 0.00 % | 0.00 % |
| forms/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| helper/fk_resolve.rs | 4.11 % | 13.33 % | 9.18 % |
| helper/resource_entry.rs | 49.62 % | 55.00 % | 54.05 % |
| helper/roles.rs | 40.00 % | 100.00 % | 36.36 % |
| helper/sql_dialect.rs | 51.72 % | 66.67 % | 60.00 % |
| helper/template.rs | 46.46 % | 52.17 % | 64.17 % |
| history.rs | 85.58 % | 100.00 % | 80.95 % |
| middleware/admin_middleware.rs | 66.67 % | 66.67 % | 80.00 % |
| mod.rs | 36.36 % | 50.00 % | 60.00 % |
| registry.rs | 58.56 % | 78.95 % | 61.64 % |
| resource.rs | 18.57 % | 13.16 % | 29.06 % |
| router/admin_router.rs | 72.64 % | 65.75 % | 76.33 % |
| table_admin/migrations_table.rs | 46.83 % | 25.00 % | 52.96 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 78.54 % | 75.00 % | 77.55 % |
| builder/mod.rs | 71.43 % | 70.59 % | 67.47 % |
| error_build.rs | 83.97 % | 100.00 % | 94.57 % |
| runique_app.rs | 6.25 % | 14.29 % | 8.11 % |
| staging/admin_staging.rs | 61.90 % | 65.22 % | 67.67 % |
| staging/core_staging.rs | 62.50 % | 70.00 % | 71.88 % |
| staging/cors_config.rs | 85.71 % | 83.33 % | 80.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 75.25 % | 82.61 % | 76.35 % |
| staging/middleware_staging/mod.rs | 64.37 % | 59.26 % | 67.84 % |
| staging/permissions_policy_config.rs | 94.44 % | 87.50 % | 91.67 % |
| staging/static_staging.rs | 50.77 % | 66.67 % | 52.38 % |
| staging/trusted_proxies_config.rs | 87.64 % | 87.50 % | 92.86 % |
| templates.rs | 77.73 % | 72.73 % | 76.74 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| guard.rs | 73.33 % | 72.00 % | 71.43 % |
| password.rs | 18.67 % | 43.24 % | 21.28 % |
| permissions/groupe.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/groupes_droits.rs | 14.29 % | 33.33 % | 16.67 % |
| permissions/mod.rs | 95.74 % | 100.00 % | 97.53 % |
| permissions/users_groupes.rs | 72.73 % | 50.00 % | 66.67 % |
| session.rs | 79.95 % | 85.37 % | 81.34 % |
| user.rs | 67.27 % | 63.16 % | 74.23 % |
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
| cli_admin.rs | 0.00 % | 0.00 % | 0.00 % |
| makemigration.rs | 80.89 % | 91.04 % | 81.86 % |
| migrate.rs | 67.58 % | 84.21 % | 68.56 % |
| new_project.rs | 0.00 % | 0.00 % | 0.00 % |
| start.rs | 0.00 % | 0.00 % | 0.00 % |

---

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| router.rs | 100.00 % | 100.00 % | 100.00 % |
| security.rs | 91.46 % | 70.83 % | 94.74 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 96.27 % | 87.50 % | 97.48 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 78.25 % | 57.58 % | 79.13 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 81.02 % | 82.76 % | 79.84 % |
| tera/static_tera.rs | 93.14 % | 84.21 % | 95.10 % |
| tera/url.rs | 91.84 % | 85.71 % | 93.88 % |

---

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 78.00 % | 100.00 % | 83.23 % |
| engine.rs | 96.77 % | 100.00 % | 75.00 % |

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 67.86 % | 44.44 % | 71.05 % |

---

## errors

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| error.rs | 78.43 % | 82.14 % | 86.64 % |

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
| base.rs | 92.72 % | 87.50 % | 88.61 % |
| extractor.rs | 92.99 % | 87.50 % | 92.19 % |
| field.rs | 69.59 % | 70.83 % | 70.17 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 90.42 % | 88.89 % | 86.71 % |
| fields/datetime.rs | 67.42 % | 85.45 % | 71.65 % |
| fields/file.rs | 69.32 % | 79.73 % | 73.30 % |
| fields/hidden.rs | 62.82 % | 60.00 % | 61.54 % |
| fields/number.rs | 78.72 % | 89.47 % | 84.85 % |
| fields/special.rs | 89.64 % | 91.38 % | 89.22 % |
| fields/text.rs | 76.35 % | 73.53 % | 81.10 % |
| form.rs | 64.94 % | 69.57 % | 68.66 % |
| generic.rs | 86.67 % | 88.46 % | 88.16 % |
| model_form/mod.rs | 46.15 % | 66.67 % | 66.67 % |
| options/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/aegis.rs | 63.00 % | 62.50 % | 65.75 % |
| prisme/rules.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/sentinel.rs | 100.00 % | 100.00 % | 100.00 % |
| renderer.rs | 81.38 % | 90.00 % | 88.54 % |
| validation_form.rs | 83.33 % | 80.00 % | 84.21 % |
| validator.rs | 85.60 % | 100.00 % | 88.16 % |

---

## macros

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| bdd/objects.rs | 78.13 % | 89.47 % | 87.06 % |
| bdd/query.rs | 75.05 % | 78.00 % | 78.24 % |
| context/flash.rs | 100.00 % | 100.00 % | 100.00 % |
| context/helper.rs | 83.33 % | 71.43 % | 79.31 % |
| context/impl_error.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/enum_kind.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/impl_form.rs | 100.00 % | 100.00 % | 100.00 % |
| routeur/register_url.rs | 86.11 % | 58.33 % | 90.00 % |
| routeur/router_ext.rs | 96.20 % | 100.00 % | 98.28 % |

---

## middleware

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| config.rs | 92.68 % | 90.91 % | 98.41 % |
| dev/cache.rs | 100.00 % | 100.00 % | 100.00 % |
| errors/error.rs | 67.30 % | 80.49 % | 70.40 % |
| security/allowed_hosts.rs | 76.95 % | 68.42 % | 68.94 % |
| security/anti_bot.rs | 59.68 % | 75.00 % | 53.66 % |
| security/csp.rs | 94.83 % | 89.47 % | 98.08 % |
| security/csrf.rs | 74.26 % | 75.00 % | 79.03 % |
| security/open_redirect.rs | 95.88 % | 100.00 % | 95.41 % |
| security/permissions_policy.rs | 100.00 % | 100.00 % | 100.00 % |
| security/rate_limit.rs | 86.83 % | 83.33 % | 88.61 % |
| security/trusted_proxies.rs | 93.15 % | 96.77 % | 92.79 % |
| session/cleaning_store.rs | 75.65 % | 82.93 % | 77.72 % |
| session/session_db.rs | 91.25 % | 100.00 % | 91.73 % |
| session/session_parametre.rs | 100.00 % | 100.00 % | 100.00 % |

---

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| column/mod.rs | 83.39 % | 96.43 % | 89.06 % |
| foreign_key/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| hooks/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| index/mod.rs | 100.00 % | 100.00 % | 100.00 % |
| primary_key/mod.rs | 97.62 % | 100.00 % | 100.00 % |
| relation/mod.rs | 85.71 % | 80.00 % | 91.18 % |
| schema/mod.rs | 87.91 % | 87.50 % | 88.72 % |
| utils/convertisseur.rs | 95.24 % | 100.00 % | 92.86 % |
| utils/diff.rs | 95.58 % | 100.00 % | 97.13 % |
| utils/generators.rs | 91.49 % | 100.00 % | 92.43 % |
| utils/helpers.rs | 70.43 % | 95.45 % | 69.67 % |
| utils/parser_builder/field.rs | 74.47 % | 100.00 % | 78.21 % |
| utils/parser_builder/mod.rs | 95.74 % | 100.00 % | 96.77 % |
| utils/parser_builder/model.rs | 74.64 % | 100.00 % | 79.17 % |
| utils/parser_builder/relation.rs | 63.25 % | 100.00 % | 63.53 % |
| utils/parser_builder/to_schema.rs | 94.90 % | 100.00 % | 95.35 % |
| utils/parser_builder/type_mapping.rs | 85.71 % | 100.00 % | 95.12 % |
| utils/parser_extend.rs | 67.92 % | 100.00 % | 76.54 % |
| utils/parser_seaorm.rs | 69.48 % | 65.22 % | 70.76 % |
| utils/paths.rs | 98.25 % | 95.92 % | 96.91 % |
| utils/tests_pipeline.rs | 99.92 % | 98.88 % | 99.89 % |
| utils/types.rs | 100.00 % | 100.00 % | 100.00 % |

---

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| aliases/helpers.rs | 69.23 % | 66.67 % | 66.67 % |
| config/env.rs | 86.67 % | 87.50 % | 90.62 % |
| config/integrity.rs | 95.45 % | 100.00 % | 100.00 % |
| config/runique_log/admin.rs | 39.02 % | 44.44 % | 40.00 % |
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
| crypto/csrf.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_html.rs | 77.16 % | 77.78 % | 65.40 % |
| forms/sanitizer.rs | 91.64 % | 88.89 % | 90.13 % |
| init_error/init.rs | 100.00 % | 100.00 % | 100.00 % |
| mailer/mod.rs | 30.89 % | 34.29 % | 32.55 % |
| password/mod.rs | 71.32 % | 72.00 % | 76.00 % |
| reset_token/entity.rs | 0.00 % | 0.00 % | 0.00 % |
| reset_token/mod.rs | 98.12 % | 100.00 % | 99.11 % |
| resolve_ogimage/mod.rs | 66.67 % | 20.00 % | 78.26 % |
| trad/switch_lang.rs | 94.30 % | 70.97 % | 92.76 % |
