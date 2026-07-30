# Couverture de tests — package `runique`

Snapshot du **2026-07-31** · commande : `cargo llvm-cov --package runique --summary-only`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **65.02 %** | **70.43 %** | **66.14 %** |

---

## admin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| admin_main/action.rs | 97.95 % | 100.00 % | 99.51 % |
| admin_main/handle_bulk.rs | 13.15 % | 11.11 % | 14.29 % |
| admin_main/handle_crud.rs | 34.92 % | 40.00 % | 43.76 % |
| admin_main/handle_inline.rs | 81.82 % | 100.00 % | 80.95 % |
| admin_main/handle_list.rs | 52.89 % | 44.00 % | 62.68 % |
| admin_main/handle_password.rs | 0.00 % | 0.00 % | 0.00 % |
| admin_main/mod.rs | 58.53 % | 60.92 % | 58.33 % |
| builtin/droit.rs | 54.33 % | 50.00 % | 52.21 % |
| builtin/groupe.rs | 46.18 % | 59.26 % | 50.87 % |
| builtin/mod.rs | 27.27 % | 50.00 % | 33.33 % |
| builtin/user.rs | 42.76 % | 46.34 % | 43.54 % |
| config/config_admin.rs | 60.96 % | 44.44 % | 60.91 % |
| daemon/generator.rs | 0.00 % | 0.00 % | 0.00 % |
| daemon/parser.rs | 0.00 % | 0.00 % | 0.00 % |
| daemon/watcher.rs | 0.00 % | 0.00 % | 0.00 % |
| forms/mod.rs | 98.50 % | 80.00 % | 95.92 % |
| helper/fk_resolve.rs | 5.17 % | 16.67 % | 11.69 % |
| helper/resource_entry.rs | 49.62 % | 55.00 % | 54.05 % |
| helper/roles.rs | 0.00 % | 0.00 % | 0.00 % |
| helper/template.rs | 46.46 % | 52.17 % | 64.17 % |
| history.rs | 43.27 % | 83.33 % | 46.03 % |
| middleware/admin_middleware.rs | 46.67 % | 66.67 % | 57.14 % |
| mod.rs | 36.36 % | 50.00 % | 60.00 % |
| permissions/groupe.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/groupes_droits.rs | 0.00 % | 0.00 % | 0.00 % |
| permissions/mod.rs | 54.26 % | 70.00 % | 62.96 % |
| permissions/users_groupes.rs | 36.36 % | 25.00 % | 33.33 % |
| registry.rs | 58.56 % | 78.95 % | 61.64 % |
| resource.rs | 18.02 % | 12.50 % | 27.91 % |
| router/admin_router.rs | 72.19 % | 65.75 % | 76.12 % |
| table_admin/migrations_table.rs | 2.40 % | 8.33 % | 1.32 % |
| trad/mod.rs | 100.00 % | 100.00 % | 100.00 % |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 78.32 % | 75.00 % | 77.64 % |
| builder/mod.rs | 65.00 % | 64.71 % | 62.65 % |
| error_build.rs | 84.87 % | 100.00 % | 93.62 % |
| runique_app.rs | 6.25 % | 14.29 % | 8.11 % |
| staging/admin_staging.rs | 58.71 % | 61.90 % | 64.75 % |
| staging/core_staging.rs | 62.50 % | 70.00 % | 71.88 % |
| staging/cors_config.rs | 0.00 % | 0.00 % | 0.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 77.30 % | 83.33 % | 79.24 % |
| staging/middleware_staging/mod.rs | 59.32 % | 55.56 % | 63.69 % |
| staging/permissions_policy_config.rs | 0.00 % | 0.00 % | 0.00 % |
| staging/static_staging.rs | 64.15 % | 72.73 % | 67.31 % |
| staging/trusted_proxies_config.rs | 0.00 % | 0.00 % | 0.00 % |
| templates.rs | 77.73 % | 72.73 % | 76.74 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| guard.rs | 64.62 % | 64.00 % | 65.08 % |
| password.rs | 17.95 % | 40.00 % | 20.30 % |
| session.rs | 76.37 % | 85.37 % | 78.17 % |
| user.rs | 67.27 % | 63.16 % | 74.23 % |
| user_trait.rs | 100.00 % | 100.00 % | 100.00 % |

---

## bin

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| runique.rs | 0.00 % | 0.00 % | 0.00 % |

---

## config

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| app.rs | 100.00 % | 100.00 % | 100.00 % |
| router.rs | 100.00 % | 100.00 % | 100.00 % |
| security.rs | 91.81 % | 72.00 % | 95.00 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 96.34 % | 87.50 % | 97.48 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 69.71 % | 56.76 % | 71.61 % |
| tera/contrib.rs | 100.00 % | 100.00 % | 100.00 % |
| tera/form.rs | 81.02 % | 82.76 % | 79.84 % |
| tera/static_tera.rs | 93.14 % | 84.21 % | 95.10 % |
| tera/url.rs | 91.84 % | 85.71 % | 93.88 % |

---

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 75.60 % | 100.00 % | 79.04 % |
| engine.rs | 96.77 % | 100.00 % | 75.00 % |

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 74.51 % | 50.00 % | 79.17 % |

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
| base.rs | 92.75 % | 87.50 % | 88.68 % |
| extractor.rs | 92.25 % | 86.67 % | 89.09 % |
| field.rs | 69.38 % | 70.45 % | 70.18 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 93.15 % | 88.89 % | 90.17 % |
| fields/datetime.rs | 81.01 % | 85.45 % | 83.02 % |
| fields/file.rs | 66.84 % | 75.71 % | 70.19 % |
| fields/hidden.rs | 71.15 % | 60.00 % | 67.09 % |
| fields/number.rs | 88.51 % | 89.47 % | 90.00 % |
| fields/special.rs | 92.63 % | 91.38 % | 92.48 % |
| fields/text.rs | 78.12 % | 73.53 % | 83.00 % |
| form.rs | 66.39 % | 69.05 % | 69.44 % |
| generic.rs | 86.96 % | 88.46 % | 88.46 % |
| model_form/mod.rs | 46.15 % | 66.67 % | 66.67 % |
| options/bool_choice.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/aegis.rs | 63.00 % | 62.50 % | 65.75 % |
| prisme/rules.rs | 100.00 % | 100.00 % | 100.00 % |
| prisme/sentinel.rs | 100.00 % | 100.00 % | 100.00 % |
| renderer.rs | 81.38 % | 90.00 % | 88.54 % |
| validator.rs | 85.48 % | 100.00 % | 88.00 % |

---

## macros

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| bdd/objects.rs | 78.13 % | 89.47 % | 87.06 % |
| bdd/query.rs | 73.27 % | 76.00 % | 76.94 % |
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
| config.rs | 93.33 % | 91.67 % | 98.70 % |
| dev/cache.rs | 100.00 % | 100.00 % | 100.00 % |
| errors/error.rs | 67.30 % | 80.49 % | 70.40 % |
| security/allowed_hosts.rs | 76.95 % | 68.42 % | 68.94 % |
| security/anti_bot.rs | 59.68 % | 75.00 % | 53.66 % |
| security/csp.rs | 94.81 % | 90.48 % | 98.23 % |
| security/csrf.rs | 74.26 % | 75.00 % | 79.03 % |
| security/open_redirect.rs | 95.88 % | 100.00 % | 95.41 % |
| security/permissions_policy.rs | 86.11 % | 75.00 % | 86.36 % |
| security/rate_limit.rs | 80.78 % | 75.00 % | 83.54 % |
| security/trusted_proxies.rs | 92.05 % | 93.55 % | 90.87 % |
| session/cleaning_store.rs | 75.70 % | 85.00 % | 78.09 % |
| session/session_db.rs | 77.50 % | 88.89 % | 82.71 % |
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
| schema/mod.rs | 87.41 % | 87.50 % | 88.35 % |
| utils/convertisseur.rs | 95.24 % | 100.00 % | 92.86 % |
| utils/diff.rs | 95.81 % | 100.00 % | 97.65 % |
| utils/generators.rs | 92.63 % | 100.00 % | 93.64 % |
| utils/helpers.rs | 72.62 % | 100.00 % | 71.71 % |
| utils/parser_builder.rs | 76.95 % | 100.00 % | 79.58 % |
| utils/parser_extend.rs | 69.83 % | 100.00 % | 76.95 % |
| utils/parser_seaorm.rs | 68.97 % | 61.90 % | 70.55 % |
| utils/paths.rs | 98.25 % | 95.92 % | 96.91 % |
| utils/tests_pipeline.rs | 99.92 % | 98.89 % | 99.89 % |
| utils/types.rs | 100.00 % | 100.00 % | 100.00 % |

---

## utils

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| aliases/helpers.rs | 69.23 % | 66.67 % | 66.67 % |
| cli/cli_admin.rs | 0.00 % | 0.00 % | 0.00 % |
| cli/makemigration.rs | 77.10 % | 88.71 % | 78.93 % |
| cli/migrate.rs | 22.00 % | 39.47 % | 24.48 % |
| cli/new_project.rs | 0.00 % | 0.00 % | 0.00 % |
| cli/start.rs | 0.00 % | 0.00 % | 0.00 % |
| config/env.rs | 69.33 % | 70.00 % | 64.44 % |
| config/integrity.rs | 95.45 % | 100.00 % | 100.00 % |
| config/runique_log/admin.rs | 19.51 % | 22.22 % | 20.00 % |
| config/runique_log/auth.rs | 88.46 % | 83.33 % | 88.00 % |
| config/runique_log/builder.rs | 90.32 % | 85.71 % | 90.00 % |
| config/runique_log/db.rs | 50.00 % | 50.00 % | 57.14 % |
| config/runique_log/errors.rs | 50.00 % | 50.00 % | 57.14 % |
| config/runique_log/forms.rs | 90.32 % | 85.71 % | 90.00 % |
| config/runique_log/mailer.rs | 36.36 % | 33.33 % | 40.00 % |
| config/runique_log/middleware.rs | 34.78 % | 40.00 % | 35.56 % |
| config/runique_log/migration.rs | 0.00 % | 0.00 % | 0.00 % |
| config/runique_log/mod.rs | 83.05 % | 72.55 % | 82.81 % |
| config/runique_log/output.rs | 55.36 % | 68.75 % | 67.57 % |
| config/runique_log/session.rs | 38.10 % | 40.00 % | 40.00 % |
| config/runique_log/templates.rs | 0.00 % | 0.00 % | 0.00 % |
| config/trace_ext.rs | 90.60 % | 100.00 % | 93.67 % |
| config/url_params.rs | 100.00 % | 100.00 % | 100.00 % |
| constante/parse.rs | 100.00 % | 100.00 % | 100.00 % |
| constante/regex_template.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| forms/parse_html.rs | 77.92 % | 77.78 % | 65.82 % |
| forms/sanitizer.rs | 91.64 % | 88.89 % | 90.13 % |
| init_error/init.rs | 0.00 % | 0.00 % | 0.00 % |
| mailer/mod.rs | 29.62 % | 31.43 % | 31.13 % |
| middleware/csp_nonce.rs | 100.00 % | 100.00 % | 100.00 % |
| middleware/csrf.rs | 100.00 % | 100.00 % | 100.00 % |
| password/mod.rs | 74.09 % | 75.61 % | 78.69 % |
| reset_token/entity.rs | 0.00 % | 0.00 % | 0.00 % |
| reset_token/mod.rs | 98.12 % | 100.00 % | 99.11 % |
| resolve_ogimage/mod.rs | 66.67 % | 20.00 % | 78.26 % |
| trad/switch_lang.rs | 94.30 % | 70.97 % | 92.76 % |
