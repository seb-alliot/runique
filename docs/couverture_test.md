## Rapport de Couverture de Code

> Mis à jour le 2026-04-26

**Résumé des résultats :**

- ✅ **~1930+ tests passés**

| Métrique      | Total  | Manqué | Couverture | Précédent | Évolution |
| ------------- | ------ | ------ | ---------- | --------- | --------- |
| **Regions**   | 19,951 | 4,739  | **76.25%** | 75.12%    | +1.13%    |
| **Functions** | 1,778  | 321    | **81.95%** | 81.10%    | +0.85%    |
| **Lines**     | 12,976 | 2,796  | **78.45%** | 77.37%    | +1.08%    |
| **Branches**  | 0      | 0      | -          | -         | -         |

> Commande : `cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only`

### Détail par Fichier

#### App

| Fichier                                        | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ---------------------------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| app\builder\build.rs                           | 265     | 28          | 89.43%   | 13        | 2           | 84.62%   | 149   | 11          | 92.62%   |
| app\builder\mod.rs                             | 93      | 28          | 69.89%   | 16        | 5           | 68.75%   | 79    | 27          | 65.82%   |
| app\error_build.rs                             | 119     | 18          | 84.87%   | 15        | 0           | 100.00%  | 94    | 6           | 93.62%   |
| app\runique_app.rs                             | 47      | 43          | 8.51%    | 7         | 6           | 14.29%   | 32    | 29          | 9.38%    |
| app\staging\core_staging.rs                    | 64      | 20          | 68.75%   | 9         | 2           | 77.78%   | 59    | 14          | 76.27%   |
| app\staging\csp_config.rs                      | 119     | 0           | 100.00%  | 18        | 0           | 100.00%  | 70    | 0           | 100.00%  |
| app\staging\host_config.rs                     | 19      | 0           | 100.00%  | 3         | 0           | 100.00%  | 12    | 0           | 100.00%  |
| app\staging\middleware_staging\applicator.rs   | 292     | 70          | 76.03%   | 19        | 2           | 89.47%   | 179   | 53          | 70.39%   |
| app\staging\middleware_staging\mod.rs          | 133     | 40          | 69.92%   | 22        | 7           | 68.18%   | 132   | 33          | 75.00%   |
| app\staging\static_staging.rs                  | 35      | 8           | 77.14%   | 10        | 2           | 80.00%   | 39    | 8           | 79.49%   |
| app\templates.rs                               | 181     | 46          | 74.59%   | 10        | 6           | 40.00%   | 99    | 29          | 70.71%   |

#### Auth

| Fichier            | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------------ | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| auth\form.rs       | 14      | 0           | 100.00%  | 1         | 0           | 100.00%  | 10    | 0           | 100.00%  |
| auth\guard.rs      | 175     | 53          | 69.71%   | 21        | 5           | 76.19%   | 112   | 28          | 75.00%   |
| auth\password.rs   | 365     | 276         | 24.38%   | 23        | 11          | 52.17%   | 250   | 184         | 26.40%   |
| auth\session.rs    | 386     | 104         | 73.06%   | 36        | 5           | 86.11%   | 244   | 51          | 79.10%   |
| auth\user.rs       | 82      | 8           | 90.24%   | 14        | 2           | 85.71%   | 78    | 6           | 92.31%   |
| auth\user_trait.rs | 11      | 0           | 100.00%  | 2         | 0           | 100.00%  | 6     | 0           | 100.00%  |

#### Config

| Fichier                | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ---------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| config\app.rs          | 18      | 0           | 100.00%  | 2         | 0           | 100.00%  | 12    | 0           | 100.00%  |
| config\router.rs       | 22      | 0           | 100.00%  | 4         | 0           | 100.00%  | 16    | 0           | 100.00%  |
| config\security.rs     | 59      | 6           | 89.83%   | 10        | 3           | 70.00%   | 37    | 3           | 91.89%   |
| config\server.rs       | 28      | 0           | 100.00%  | 4         | 0           | 100.00%  | 21    | 0           | 100.00%  |
| config\static_files.rs | 99      | 4           | 95.96%   | 17        | 2           | 88.24%   | 66    | 2           | 96.97%   |

#### Context

| Fichier                        | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------------------------ | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| context\request\extractor.rs   | 30      | 2           | 93.33%   | 2         | 0           | 100.00%  | 28    | 0           | 100.00%  |
| context\request_extensions.rs  | 119     | 20          | 83.19%   | 10        | 0           | 100.00%  | 78    | 5           | 93.59%   |
| context\template.rs            | 313     | 81          | 74.12%   | 31        | 10          | 67.74%   | 201   | 46          | 77.11%   |
| context\tera\form.rs           | 265     | 59          | 77.74%   | 38        | 10          | 73.68%   | 161   | 32          | 80.12%   |
| context\tera\static_tera.rs    | 122     | 34          | 72.13%   | 8         | 3           | 62.50%   | 63    | 16          | 74.60%   |
| context\tera\url.rs            | 98      | 7           | 92.86%   | 9         | 1           | 88.89%   | 53    | 3           | 94.34%   |

#### Database

| Fichier          | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ---------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| db\builder.rs    | 24      | 0           | 100.00%  | 6         | 0           | 100.00%  | 24    | 0           | 100.00%  |
| db\config.rs     | 248     | 63          | 74.60%   | 16        | 0           | 100.00%  | 163   | 36          | 77.91%   |
| db\engine.rs     | 31      | 3           | 90.32%   | 3         | 0           | 100.00%  | 32    | 10          | 68.75%   |

#### Engine

| Fichier         | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| --------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| engine\core.rs  | 71      | 12          | 83.10%   | 4         | 2           | 50.00%   | 50    | 6           | 88.00%   |

#### Errors

| Fichier           | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ----------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| errors\error.rs   | 412     | 89          | 78.40%   | 28        | 5           | 82.14%   | 288   | 34          | 88.19%   |

#### Flash

| Fichier                   | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| flash\flash_manager.rs    | 71      | 0           | 100.00%  | 14        | 0           | 100.00%  | 49    | 0           | 100.00%  |
| flash\flash_struct.rs     | 27      | 0           | 100.00%  | 6         | 0           | 100.00%  | 37    | 0           | 100.00%  |

#### Forms

| Fichier                      | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ---------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| forms\base.rs                | 174     | 6           | 96.55%   | 34        | 2           | 94.12%   | 130   | 6           | 95.38%   |
| forms\extractor.rs           | 94      | 23          | 75.53%   | 10        | 4           | 60.00%   | 70    | 15          | 78.57%   |
| forms\field.rs               | 178     | 18          | 89.89%   | 27        | 2           | 92.59%   | 93    | 5           | 94.62%   |
| forms\fields\boolean.rs      | 81      | 32          | 60.49%   | 11        | 2           | 81.82%   | 54    | 18          | 66.67%   |
| forms\fields\choice.rs       | 348     | 87          | 75.00%   | 45        | 9           | 80.00%   | 239   | 56          | 76.57%   |
| forms\fields\datetime.rs     | 615     | 130         | 78.86%   | 53        | 8           | 84.91%   | 426   | 78          | 81.69%   |
| forms\fields\file.rs         | 553     | 206         | 62.75%   | 50        | 7           | 86.00%   | 334   | 102         | 69.46%   |
| forms\fields\hidden.rs       | 82      | 27          | 67.07%   | 9         | 2           | 77.78%   | 56    | 16          | 71.43%   |
| forms\fields\number.rs       | 269     | 38          | 85.87%   | 19        | 2           | 89.47%   | 203   | 23          | 88.67%   |
| forms\fields\special.rs      | 518     | 138         | 73.36%   | 58        | 12          | 79.31%   | 366   | 87          | 76.23%   |
| forms\fields\text.rs         | 300     | 42          | 86.00%   | 30        | 5           | 83.33%   | 189   | 20          | 89.42%   |
| forms\form.rs                | 473     | 87          | 81.61%   | 27        | 5           | 81.48%   | 236   | 35          | 85.17%   |
| forms\generic.rs             | 92      | 16          | 82.61%   | 26        | 4           | 84.62%   | 78    | 12          | 84.62%   |
| forms\model_form\mod.rs      | 13      | 7           | 46.15%   | 3         | 1           | 66.67%   | 9     | 3           | 66.67%   |
| forms\options\bool_choice.rs | 3       | 0           | 100.00%  | 1         | 0           | 100.00%  | 3     | 0           | 100.00%  |
| forms\prisme\aegis.rs        | 100     | 37          | 63.00%   | 8         | 3           | 62.50%   | 73    | 25          | 65.75%   |
| forms\prisme\rules.rs        | 71      | 0           | 100.00%  | 11        | 0           | 100.00%  | 77    | 0           | 100.00%  |
| forms\prisme\sentinel.rs     | 16      | 0           | 100.00%  | 1         | 0           | 100.00%  | 7     | 0           | 100.00%  |
| forms\renderer.rs            | 112     | 10          | 91.07%   | 9         | 1           | 88.89%   | 73    | 2           | 97.26%   |
| forms\validator.rs           | 94      | 1           | 98.94%   | 9         | 0           | 100.00%  | 56    | 1           | 98.21%   |

#### Macros

| Fichier                         | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| macros\bdd\objects.rs           | 375     | 82          | 78.13%   | 38        | 4           | 89.47%   | 239   | 33          | 86.19%   |
| macros\bdd\query.rs             | 492     | 111         | 77.44%   | 46        | 8           | 82.61%   | 297   | 61          | 79.46%   |
| macros\context\flash.rs         | 92      | 0           | 100.00%  | 19        | 0           | 100.00%  | 65    | 0           | 100.00%  |
| macros\context\helper.rs        | 31      | 3           | 90.32%   | 6         | 1           | 83.33%   | 26    | 3           | 88.46%   |
| macros\context\impl_error.rs    | 8       | 0           | 100.00%  | 2         | 0           | 100.00%  | 2     | 0           | 100.00%  |
| macros\forms\enum_kind.rs       | 3       | 0           | 100.00%  | 1         | 0           | 100.00%  | 3     | 0           | 100.00%  |
| macros\forms\impl_form.rs       | 9       | 0           | 100.00%  | 3         | 0           | 100.00%  | 9     | 0           | 100.00%  |
| macros\routeur\register_url.rs  | 72      | 10          | 86.11%   | 12        | 5           | 58.33%   | 50    | 5           | 90.00%   |
| macros\routeur\router_ext.rs    | 71      | 0           | 100.00%  | 3         | 0           | 100.00%  | 52    | 0           | 100.00%  |

#### Middleware

| Fichier                                    | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------------------------------------ | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| middleware\config.rs                       | 45      | 3           | 93.33%   | 12        | 1           | 91.67%   | 77    | 1           | 98.70%   |
| middleware\dev\cache.rs                    | 46      | 28          | 39.13%   | 5         | 2           | 60.00%   | 31    | 19          | 38.71%   |
| middleware\errors\error.rs                 | 376     | 93          | 75.27%   | 30        | 6           | 80.00%   | 233   | 50          | 78.54%   |
| middleware\security\allowed_hosts.rs       | 265     | 58          | 78.11%   | 19        | 6           | 68.42%   | 126   | 35          | 72.22%   |
| middleware\security\csp.rs                 | 397     | 20          | 94.96%   | 21        | 2           | 90.48%   | 221   | 4           | 98.19%   |
| middleware\security\csrf.rs                | 187     | 50          | 73.26%   | 12        | 3           | 75.00%   | 119   | 27          | 77.31%   |
| middleware\security\rate_limit.rs          | 137     | 29          | 78.83%   | 19        | 3           | 84.21%   | 94    | 14          | 85.11%   |
| middleware\session\cleaning_store.rs       | 188     | 60          | 68.09%   | 32        | 6           | 81.25%   | 137   | 35          | 74.45%   |
| middleware\session\session_db.rs           | 141     | 32          | 77.30%   | 18        | 2           | 88.89%   | 100   | 14          | 86.00%   |
| middleware\session\session_parametre.rs    | 7       | 0           | 100.00%  | 2         | 0           | 100.00%  | 10    | 0           | 100.00%  |

#### Migration

| Fichier                            | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ---------------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| migration\column\mod.rs            | 553     | 69          | 87.52%   | 54        | 0           | 100.00%  | 361   | 23          | 93.63%   |
| migration\foreign_key\mod.rs       | 42      | 0           | 100.00%  | 6         | 0           | 100.00%  | 39    | 0           | 100.00%  |
| migration\hooks\mod.rs             | 45      | 0           | 100.00%  | 7         | 0           | 100.00%  | 30    | 0           | 100.00%  |
| migration\index\mod.rs             | 44      | 0           | 100.00%  | 6         | 0           | 100.00%  | 29    | 0           | 100.00%  |
| migration\primary_key\mod.rs       | 42      | 1           | 97.62%   | 7         | 0           | 100.00%  | 38    | 0           | 100.00%  |
| migration\relation\mod.rs          | 18      | 0           | 100.00%  | 4         | 0           | 100.00%  | 31    | 0           | 100.00%  |
| migration\schema\mod.rs            | 370     | 27          | 92.70%   | 28        | 0           | 100.00%  | 246   | 15          | 93.90%   |
| migration\utils\convertisseur.rs   | 21      | 1           | 95.24%   | 3         | 0           | 100.00%  | 14    | 1           | 92.86%   |
| migration\utils\diff.rs            | 239     | 1           | 99.58%   | 21        | 0           | 100.00%  | 138   | 1           | 99.28%   |
| migration\utils\generators.rs      | 666     | 62          | 90.69%   | 31        | 0           | 100.00%  | 538   | 47          | 91.26%   |
| migration\utils\helpers.rs         | 661     | 183         | 72.31%   | 20        | 0           | 100.00%  | 350   | 99          | 71.71%   |
| migration\utils\parser_builder.rs  | 754     | 165         | 78.12%   | 17        | 0           | 100.00%  | 420   | 80          | 80.95%   |
| migration\utils\parser_extend.rs   | 254     | 76          | 70.08%   | 8         | 0           | 100.00%  | 134   | 24          | 82.09%   |
| migration\utils\parser_seaorm.rs   | 424     | 145         | 65.80%   | 19        | 8           | 57.89%   | 275   | 89          | 67.64%   |
| migration\utils\paths.rs           | 342     | 6           | 98.25%   | 49        | 2           | 95.92%   | 194   | 6           | 96.91%   |
| migration\utils\types.rs           | 23      | 0           | 100.00%  | 1         | 0           | 100.00%  | 13    | 0           | 100.00%  |

#### Utils

| Fichier                          | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| -------------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| utils\aliases\helpers.rs         | 13      | 4           | 69.23%   | 3         | 1           | 66.67%   | 9     | 3           | 66.67%   |
| utils\cli\makemigration.rs       | 957     | 379         | 60.40%   | 49        | 17          | 65.31%   | 538   | 210         | 60.97%   |
| utils\cli\migrate.rs             | 688     | 541         | 21.37%   | 38        | 23          | 39.47%   | 384   | 292         | 23.96%   |
| utils\cli\new_project.rs         | 135     | 135         | 0.00%    | 7         | 7           | 0.00%    | 142   | 142         | 0.00%    |
| utils\cli\start.rs               | 87      | 87          | 0.00%    | 7         | 7           | 0.00%    | 47    | 47          | 0.00%    |
| utils\config\env.rs              | 73      | 21          | 71.23%   | 10        | 3           | 70.00%   | 42    | 13          | 69.05%   |
| utils\config\integrity.rs        | 44      | 2           | 95.45%   | 2         | 0           | 100.00%  | 21    | 0           | 100.00%  |
| utils\config\runique_log.rs      | 90      | 12          | 86.67%   | 17        | 0           | 100.00%  | 79    | 11          | 86.08%   |
| utils\config\url_params.rs       | 13      | 0           | 100.00%  | 3         | 0           | 100.00%  | 10    | 0           | 100.00%  |
| utils\constante\parse.rs         | 50      | 0           | 100.00%  | 7         | 0           | 100.00%  | 44    | 0           | 100.00%  |
| utils\constante\regex_template.rs| 19      | 0           | 100.00%  | 5         | 0           | 100.00%  | 10    | 0           | 100.00%  |
| utils\forms\parse_boolean.rs     | 10      | 0           | 100.00%  | 2         | 0           | 100.00%  | 6     | 0           | 100.00%  |
| utils\forms\parse_html.rs        | 208     | 51          | 75.48%   | 12        | 6           | 50.00%   | 143   | 71          | 50.35%   |
| utils\forms\sanitizer.rs         | 220     | 22          | 90.00%   | 14        | 2           | 85.71%   | 125   | 14          | 88.80%   |
| utils\init_error\init.rs         | 4       | 0           | 100.00%  | 1         | 0           | 100.00%  | 3     | 0           | 100.00%  |
| utils\mailer\mod.rs              | 188     | 188         | 0.00%    | 25        | 25          | 0.00%    | 125   | 125         | 0.00%    |
| utils\middleware\csp_nonce.rs    | 53      | 0           | 100.00%  | 6         | 0           | 100.00%  | 30    | 0           | 100.00%  |
| utils\middleware\csrf.rs         | 139     | 0           | 100.00%  | 11        | 0           | 100.00%  | 72    | 0           | 100.00%  |
| utils\password\mod.rs            | 343     | 89          | 74.05%   | 38        | 9           | 76.32%   | 229   | 48          | 79.04%   |
| utils\reset_token\mod.rs         | 212     | 0           | 100.00%  | 8         | 0           | 100.00%  | 85    | 0           | 100.00%  |
| utils\resolve_ogimage\mod.rs     | 30      | 13          | 56.67%   | 5         | 4           | 20.00%   | 25    | 12          | 52.00%   |
| utils\trad\switch_lang.rs        | 228     | 13          | 94.30%   | 31        | 9           | 70.97%   | 152   | 11          | 92.76%   |

### Modules à faible couverture — priorité tests

| Module                              | L. Manquées | L. Cover | Raison probable                        |
| ----------------------------------- | ----------- | -------- | -------------------------------------- |
| utils\cli\new_project.rs            | 142         | 0.00%    | CLI — génération fichiers              |
| utils\cli\start.rs                  | 47          | 0.00%    | CLI — démarrage processus              |
| utils\mailer\mod.rs                 | 125         | 0.00%    | SMTP externe                           |
| app\runique_app.rs                  | 29          | 9.38%    | Nécessite setup complet app            |
| utils\cli\migrate.rs                | 292         | 23.96%   | Nécessite DB + fichiers migrations     |
| auth\password.rs                    | 184         | 26.40%   | Password reset — nécessite DB          |
| middleware\dev\cache.rs             | 19          | 38.71%   | Cache dev — difficile à isoler         |
| utils\forms\parse_html.rs           | 71          | 50.35%   | Parsing multipart — nécessite runtime  |
| utils\resolve_ogimage\mod.rs        | 12          | 52.00%   | Résolution OG image — HTTP externe     |
| migration\utils\parser_seaorm.rs    | 89          | 67.64%   | Parser SeaORM — branches complexes     |
| forms\prisme\aegis.rs               | 25          | 65.75%   | Extraction body — nécessite Axum       |
| forms\model_form\mod.rs             | 3           | 66.67%   | Petit fichier — compléter              |
| forms\fields\boolean.rs             | 18          | 66.67%   | render() nécessite Tera                |
| utils\cli\makemigration.rs          | 210         | 60.97%   | CLI — branches fichiers                |
