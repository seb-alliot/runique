## Rapport de Couverture de Code

> Mis à jour le 2026-04-21

**Résumé des résultats :**

- ✅ **1701 tests passés**

| Métrique      | Total  | Manqué | Couverture | Précédent | Évolution |
| ------------- | ------ | ------ | ---------- | --------- | --------- |
| **Regions**   | 19,557 | 7,018  | **64.12%** | 69.11%    | -4.99%    |
| **Functions** | 1,737  | 514    | **70.41%** | 76.59%    | -6.18%    |
| **Lines**     | 12,722 | 4,307  | **66.15%** | 71.52%    | -5.37%    |
| **Branches**  | 0      | 0      | -          | -         | -         |

> Note : La baisse est attendue — ~9000 lignes de code ajoutées depuis le dernier audit (sessions DB, permissions admin, group_action, password reset, mailer, etc.) sans tests associés.

> Commande : `cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only`

### Détail par Fichier

#### App

| Fichier                           | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| --------------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| app\builder.rs                    | 330     | 56          | 83.03%   | 27        | 6           | 77.78%   | 210   | 37          | 82.38%   |
| app\error_build.rs                | 119     | 18          | 84.87%   | 15        | 0           | 100.00%  | 94    | 6           | 93.62%   |
| app\runique_app.rs                | 47      | 43          | 8.51%    | 7         | 6           | 14.29%   | 32    | 29          | 9.38%    |
| app\staging\core_staging.rs       | 64      | 20          | 68.75%   | 9         | 2           | 77.78%   | 59    | 14          | 76.27%   |
| app\staging\csp_config.rs         | 119     | 0           | 100.00%  | 18        | 0           | 100.00%  | 70    | 0           | 100.00%  |
| app\staging\host_config.rs        | 19      | 0           | 100.00%  | 3         | 0           | 100.00%  | 12    | 0           | 100.00%  |
| app\staging\middleware_staging.rs | 425     | 110         | 74.12%   | 41        | 9           | 78.05%   | 309   | 88          | 71.52%   |
| app\staging\static_staging.rs     | 27      | 0           | 100.00%  | 8         | 0           | 100.00%  | 27    | 0           | 100.00%  |
| app\templates.rs                  | 181     | 46          | 74.59%   | 10        | 6           | 40.00%   | 99    | 29          | 70.71%   |

#### Auth

| Fichier            | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------------ | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| auth\form.rs       | 14      | 0           | 100.00%  | 1         | 0           | 100.00%  | 10    | 0           | 100.00%  |
| auth\guard.rs      | 175     | 61          | 65.14%   | 21        | 6           | 71.43%   | 112   | 33          | 70.54%   |
| auth\password.rs   | 359     | 359         | 0.00%    | 23        | 23          | 0.00%    | 248   | 248         | 0.00%    |
| auth\session.rs    | 400     | 182         | 54.50%   | 33        | 10          | 69.70%   | 268   | 106         | 60.45%   |
| auth\user.rs       | 82      | 61          | 25.61%   | 14        | 7           | 50.00%   | 78    | 57          | 26.92%   |
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
| context\tera\url.rs            | 98      | 41          | 58.16%   | 9         | 2           | 77.78%   | 53    | 21          | 60.38%   |

#### Database

| Fichier      | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------ | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| db\config.rs | 303     | 66          | 78.22%   | 25        | 0           | 100.00%  | 219   | 46          | 79.00%   |

#### Forms

| Fichier                    | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| -------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| forms\base.rs              | 174     | 9           | 94.83%   | 34        | 3           | 91.18%   | 130   | 9           | 93.08%   |
| forms\extractor.rs         | 94      | 23          | 75.53%   | 10        | 4           | 60.00%   | 70    | 15          | 78.57%   |
| forms\field.rs             | 178     | 134         | 24.72%   | 27        | 18          | 33.33%   | 93    | 65          | 30.11%   |
| forms\fields\boolean.rs    | 81      | 59          | 27.16%   | 11        | 6           | 45.45%   | 54    | 35          | 35.19%   |
| forms\fields\choice.rs     | 348     | 87          | 75.00%   | 45        | 9           | 80.00%   | 239   | 56          | 76.57%   |
| forms\fields\datetime.rs   | 615     | 130         | 78.86%   | 53        | 8           | 84.91%   | 426   | 78          | 81.69%   |
| forms\fields\file.rs       | 553     | 553         | 0.00%    | 50        | 50          | 0.00%    | 334   | 334         | 0.00%    |
| forms\fields\hidden.rs     | 82      | 27          | 67.07%   | 9         | 2           | 77.78%   | 56    | 16          | 71.43%   |
| forms\fields\number.rs     | 269     | 38          | 85.87%   | 19        | 2           | 89.47%   | 203   | 23          | 88.67%   |
| forms\fields\special.rs    | 518     | 152         | 70.66%   | 58        | 14          | 75.86%   | 366   | 97          | 73.50%   |
| forms\fields\text.rs       | 300     | 42          | 86.00%   | 30        | 5           | 83.33%   | 189   | 20          | 89.42%   |
| forms\form.rs              | 473     | 339         | 28.33%   | 27        | 15          | 44.44%   | 236   | 151         | 36.02%   |
| forms\generic.rs           | 92      | 25          | 72.83%   | 26        | 7           | 73.08%   | 78    | 21          | 73.08%   |
| forms\model_form\mod.rs    | 13      | 7           | 46.15%   | 3         | 1           | 66.67%   | 9     | 3           | 66.67%   |
| forms\prisme\aegis.rs      | 100     | 37          | 63.00%   | 8         | 3           | 62.50%   | 73    | 25          | 65.75%   |
| forms\prisme\rules.rs      | 71      | 0           | 100.00%  | 11        | 0           | 100.00%  | 77    | 0           | 100.00%  |
| forms\prisme\sentinel.rs   | 16      | 0           | 100.00%  | 1         | 0           | 100.00%  | 7     | 0           | 100.00%  |
| forms\renderer.rs          | 112     | 10          | 91.07%   | 9         | 1           | 88.89%   | 73    | 2           | 97.26%   |
| forms\validator.rs         | 94      | 1           | 98.94%   | 9         | 0           | 100.00%  | 56    | 1           | 98.21%   |

#### Middleware

| Fichier                                   | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ----------------------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| middleware\config.rs                      | 45      | 3           | 93.33%   | 12        | 1           | 91.67%   | 77    | 1           | 98.70%   |
| middleware\security\allowed_hosts.rs      | 265     | 58          | 78.11%   | 19        | 6           | 68.42%   | 126   | 35          | 72.22%   |
| middleware\security\csp.rs                | 397     | 20          | 94.96%   | 21        | 2           | 90.48%   | 221   | 4           | 98.19%   |
| middleware\security\csrf.rs               | 187     | 50          | 73.26%   | 12        | 3           | 75.00%   | 119   | 27          | 77.31%   |
| middleware\security\rate_limit.rs         | 137     | 29          | 78.83%   | 19        | 3           | 84.21%   | 94    | 14          | 85.11%   |
| middleware\session\cleaning_store.rs      | 188     | 60          | 68.09%   | 32        | 6           | 81.25%   | 137   | 35          | 74.45%   |
| middleware\session\session_db.rs          | 141     | 138         | 2.13%    | 18        | 17          | 5.56%    | 100   | 97          | 3.00%    |

#### Migration

| Fichier                            | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ---------------------------------- | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| migration\column\mod.rs            | 553     | 69          | 87.52%   | 54        | 0           | 100.00%  | 361   | 23          | 93.63%   |
| migration\schema\mod.rs            | 370     | 122         | 67.03%   | 28        | 7           | 75.00%   | 246   | 89          | 63.82%   |
| migration\utils\generators.rs      | 666     | 245         | 63.21%   | 31        | 5           | 83.87%   | 538   | 206         | 61.71%   |
| migration\utils\helpers.rs         | 661     | 183         | 72.31%   | 20        | 0           | 100.00%  | 350   | 99          | 71.71%   |
| migration\utils\makemigration.rs   | 957     | 379         | 60.40%   | 49        | 17          | 65.31%   | 538   | 210         | 60.97%   |
| migration\utils\migrate.rs         | 688     | 541         | 21.37%   | 38        | 23          | 39.47%   | 384   | 292         | 23.96%   |
| migration\utils\parser_builder.rs  | 754     | 165         | 78.12%   | 17        | 0           | 100.00%  | 420   | 80          | 80.95%   |
| migration\utils\parser_extend.rs   | 254     | 225         | 11.42%   | 8         | 4           | 50.00%   | 134   | 110         | 17.91%   |
| migration\utils\parser_seaorm.rs   | 424     | 145         | 65.80%   | 19        | 8           | 57.89%   | 275   | 89          | 67.64%   |
| migration\utils\paths.rs           | 342     | 6           | 98.25%   | 49        | 2           | 95.92%   | 194   | 6           | 96.91%   |

#### Utils

| Fichier                        | Regions | R. Manquées | R. Cover | Functions | F. Manquées | F. Cover | Lines | L. Manquées | L. Cover |
| ------------------------------ | ------- | ----------- | -------- | --------- | ----------- | -------- | ----- | ----------- | -------- |
| utils\cli\new_project.rs       | 135     | 135         | 0.00%    | 7         | 7           | 0.00%    | 142   | 142         | 0.00%    |
| utils\cli\start.rs             | 87      | 87          | 0.00%    | 7         | 7           | 0.00%    | 47    | 47          | 0.00%    |
| utils\config\runique_log.rs    | 90      | 60          | 33.33%   | 17        | 12          | 29.41%   | 79    | 58          | 26.58%   |
| utils\forms\parse_boolean.rs   | 10      | 10          | 0.00%    | 2         | 2           | 0.00%    | 6     | 6           | 0.00%    |
| utils\forms\parse_html.rs      | 208     | 51          | 75.48%   | 12        | 6           | 50.00%   | 143   | 71          | 50.35%   |
| utils\forms\sanitizer.rs       | 220     | 22          | 90.00%   | 14        | 2           | 85.71%   | 125   | 14          | 88.80%   |
| utils\mailer\mod.rs            | 188     | 188         | 0.00%    | 25        | 25          | 0.00%    | 125   | 125         | 0.00%    |
| utils\reset_token\mod.rs       | 212     | 212         | 0.00%    | 8         | 8           | 0.00%    | 85    | 85          | 0.00%    |
| utils\password\mod.rs          | 343     | 137         | 60.06%   | 38        | 12          | 68.42%   | 229   | 82          | 64.19%   |

### Modules à 0% — priorité tests

| Module                        | Raison probable            |
| ----------------------------- | -------------------------- |
| auth\password.rs              | Password reset — nouveau   |
| forms\fields\file.rs          | Upload — difficile à tester|
| middleware\session\session_db | Sessions DB — nouveau      |
| utils\mailer\mod.rs           | Mailer — nouveau           |
| utils\reset_token\mod.rs      | Reset token — nouveau      |
| utils\cli\new_project.rs      | CLI — difficile à tester   |
| utils\cli\start.rs            | CLI — difficile à tester   |
| utils\forms\parse_boolean.rs  | Manque tests unitaires     |
