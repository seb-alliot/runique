# Couverture de tests — package `runique`

Snapshot du **2026-06-17** · commande : `cargo llvm-cov --package runique --ignore-filename-regex "admin" --summary-only`

| | Régions | Fonctions | Lignes |
|---|---|---|---|
| **TOTAL** | **75.46 %** | **78.17 %** | **77.26 %** |

---

## app

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder/build.rs | 80.52 % | 68.75 % | 81.19 % |
| builder/mod.rs | 65.00 % | 64.71 % | 62.65 % |
| error_build.rs | 84.87 % | 100.00 % | 93.62 % |
| runique_app.rs | 6.25 % | 14.29 % | 8.11 % |
| staging/core_staging.rs | 62.50 % | 70.00 % | 71.88 % |
| staging/cors_config.rs | 0.00 % | 0.00 % | 0.00 % |
| staging/csp_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/host_config.rs | 100.00 % | 100.00 % | 100.00 % |
| staging/middleware_staging/applicator.rs | 68.38 % | 83.33 % | 65.96 % |
| staging/middleware_staging/mod.rs | 59.09 % | 55.56 % | 63.07 % |
| staging/permissions_policy_config.rs | 0.00 % | 0.00 % | 0.00 % |
| staging/static_staging.rs | 64.15 % | 72.73 % | 67.31 % |
| staging/trusted_proxies_config.rs | 0.00 % | 0.00 % | 0.00 % |
| templates.rs | 77.63 % | 66.67 % | 74.81 % |

---

## auth

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| form.rs | 100.00 % | 100.00 % | 100.00 % |
| guard.rs | 69.71 % | 76.19 % | 75.00 % |
| password.rs | 17.95 % | 40.00 % | 20.30 % |
| session.rs | 68.11 % | 81.08 % | 71.71 % |
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
| security.rs | 88.41 % | 66.67 % | 90.91 % |
| server.rs | 100.00 % | 100.00 % | 100.00 % |
| static_files.rs | 96.34 % | 87.50 % | 97.48 % |

---

## context

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| request/extractor.rs | 93.33 % | 100.00 % | 100.00 % |
| request_extensions.rs | 83.19 % | 100.00 % | 93.59 % |
| template.rs | 65.44 % | 54.05 % | 68.51 % |
| tera/form.rs | 74.72 % | 68.89 % | 76.97 % |
| tera/static_tera.rs | 56.07 % | 35.71 % | 58.14 % |
| tera/url.rs | 92.86 % | 88.89 % | 94.34 % |

---

## db

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| builder.rs | 100.00 % | 100.00 % | 100.00 % |
| config.rs | 74.40 % | 100.00 % | 76.65 % |
| engine.rs | 90.32 % | 100.00 % | 68.75 % |

---

## engine

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| core.rs | 71.74 % | 28.57 % | 76.19 % |

---

## errors

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| error.rs | 78.40 % | 82.14 % | 88.19 % |

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
| base.rs | 93.33 % | 88.89 % | 89.21 % |
| extractor.rs | 75.53 % | 60.00 % | 78.57 % |
| field.rs | 68.92 % | 68.29 % | 68.90 % |
| fields/boolean.rs | 100.00 % | 100.00 % | 100.00 % |
| fields/choice.rs | 93.39 % | 88.89 % | 90.38 % |
| fields/datetime.rs | 79.20 % | 85.45 % | 81.86 % |
| fields/file.rs | 58.64 % | 73.44 % | 63.38 % |
| fields/hidden.rs | 71.93 % | 60.00 % | 67.47 % |
| fields/number.rs | 85.87 % | 89.47 % | 88.67 % |
| fields/special.rs | 92.86 % | 91.38 % | 92.62 % |
| fields/text.rs | 78.15 % | 73.53 % | 83.17 % |
| form.rs | 61.95 % | 57.14 % | 64.27 % |
| generic.rs | 82.61 % | 84.62 % | 84.62 % |
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
| context/helper.rs | 82.35 % | 71.43 % | 79.31 % |
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
| dev/cache.rs | 69.57 % | 100.00 % | 77.42 % |
| errors/error.rs | 74.50 % | 81.58 % | 77.78 % |
| security/allowed_hosts.rs | 76.95 % | 68.42 % | 68.94 % |
| security/anti_bot.rs | 62.71 % | 75.00 % | 56.41 % |
| security/csp.rs | 94.76 % | 90.48 % | 98.21 % |
| security/csrf.rs | 74.36 % | 76.92 % | 78.57 % |
| security/open_redirect.rs | 95.88 % | 100.00 % | 95.41 % |
| security/permissions_policy.rs | 86.11 % | 75.00 % | 86.36 % |
| security/rate_limit.rs | 82.67 % | 82.76 % | 85.38 % |
| security/trusted_proxies.rs | 90.91 % | 92.31 % | 89.33 % |
| session/cleaning_store.rs | 60.32 % | 76.92 % | 66.86 % |
| session/session_db.rs | 79.39 % | 88.89 % | 85.95 % |
| session/session_parametre.rs | 100.00 % | 100.00 % | 100.00 % |

---

## migration

| Fichier | Régions | Fonctions | Lignes |
|---|---|---|---|
| column/mod.rs | 87.52 % | 100.00 % | 93.63 % |
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
| cli/makemigration.rs | 77.10 % | 88.71 % | 78.93 % |
| cli/migrate.rs | 21.27 % | 39.47 % | 23.71 % |
| cli/new_project.rs | 0.00 % | 0.00 % | 0.00 % |
| cli/start.rs | 0.00 % | 0.00 % | 0.00 % |
| config/env.rs | 69.33 % | 70.00 % | 64.44 % |
| config/integrity.rs | 95.45 % | 100.00 % | 100.00 % |
| config/runique_log/auth.rs | 85.71 % | 80.00 % | 85.00 % |
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
| forms/parse_html.rs | 70.95 % | 50.00 % | 50.32 % |
| forms/sanitizer.rs | 80.36 % | 77.78 % | 80.26 % |
| init_error/init.rs | 0.00 % | 0.00 % | 0.00 % |
| mailer/mod.rs | 31.53 % | 32.35 % | 32.84 % |
| middleware/csp_nonce.rs | 100.00 % | 100.00 % | 100.00 % |
| middleware/csrf.rs | 100.00 % | 100.00 % | 100.00 % |
| password/mod.rs | 74.09 % | 75.61 % | 78.69 % |
| reset_token/entity.rs | 0.00 % | 0.00 % | 0.00 % |
| reset_token/mod.rs | 98.86 % | 100.00 % | 100.00 % |
| resolve_ogimage/mod.rs | 53.33 % | 20.00 % | 52.17 % |
| trad/switch_lang.rs | 94.30 % | 70.97 % | 92.76 % |
