## Rapport de Couverture de Code

> Mis à jour le 2026-03-31

**Résumé des résultats :**

- ✅ **1833 tests passés** en 49.89s
  
| Métrique      | Total  | Manqué | Couverture | Précédent | Évolution |
| ------------- | ------ | ------ | ---------- | --------- | --------- |
| **Regions**   | 17,846 | 5,512  | **69.11%** | 75.38%    | -6.27%    |
| **Functions** | 1,670  | 391    | **76.59%** | 82.83%    | -6.24%    |
| **Lines**     | 11,894 | 3,387  | **71.52%** | 78.35%    | -6.83%    |
| **Branches**  | 0      | 0      | -          | -         | -         |

### Détail par Fichier

#### App

| Fichier                           | Regions Cover | Functions Cover | Lines Cover |
| --------------------------------- | ------------- | --------------- | ----------- |
| app\builder.rs                    | 286           | 53              | 81.47%      | 25 | 6 | 76.00%  | 181 | 37 | 79.56%  | 0 | 0 | - |
| app\error_build.rs                | 127           | 19              | 85.04%      | 15 | 0 | 100.00% | 92  | 5  | 94.57%  | 0 | 0 | - |
| app\staging\core_staging.rs       | 65            | 21              | 67.69%      | 9  | 2 | 77.78%  | 59  | 14 | 76.27%  | 0 | 0 | - |
| app\staging\csp_config.rs         | 119           | 0               | 100.00%     | 18 | 0 | 100.00% | 70  | 0  | 100.00% | 0 | 0 | - |
| app\staging\host_config.rs        | 19            | 0               | 100.00%     | 3  | 0 | 100.00% | 12  | 0  | 100.00% | 0 | 0 | - |
| app\staging\middleware_staging.rs | 410           | 115             | 71.95%      | 40 | 9 | 77.50%  | 364 | 90 | 75.27%  | 0 | 0 | - |
| app\staging\static_staging.rs     | 27            | 0               | 100.00%     | 8  | 0 | 100.00% | 27  | 0  | 100.00% | 0 | 0 | - |
| app\templates.rs                  | 166           | 37              | 77.71%      | 10 | 5 | 50.00%  | 88  | 21 | 76.14%  | 0 | 0 | - |

#### Bin

**Cli** Difficile a tester

| Fichier        | Regions Cover | Functions Cover | Lines Cover |
| -------------- | ------------- | --------------- | ----------- |
| bin\runique.rs | 145           | 145             | 0.00%       | 10 | 10 | 0.00% | 103 | 103 | 0.00% | 0 | 0 | - |

#### Config

| Fichier                | Regions Cover | Functions Cover | Lines Cover |
| ---------------------- | ------------- | --------------- | ----------- |
| config\app.rs          | 19            | 0               | 100.00%     | 2  | 0 | 100.00% | 13 | 0 | 100.00% | 0 | 0 | - |
| config\router.rs       | 22            | 0               | 100.00%     | 4  | 0 | 100.00% | 16 | 0 | 100.00% | 0 | 0 | - |
| config\security.rs     | 37            | 0               | 100.00%     | 7  | 0 | 100.00% | 26 | 0 | 100.00% | 0 | 0 | - |
| config\server.rs       | 29            | 0               | 100.00%     | 4  | 0 | 100.00% | 21 | 0 | 100.00% | 0 | 0 | - |
| config\settings.rs     | 44            | 0               | 100.00%     | 2  | 0 | 100.00% | 23 | 0 | 100.00% | 0 | 0 | - |
| config\static_files.rs | 96            | 4               | 95.83%      | 17 | 2 | 88.24%  | 63 | 2 | 96.83%  | 0 | 0 | - |

#### Context

| Fichier                       | Regions Cover | Functions Cover | Lines Cover |
| ----------------------------- | ------------- | --------------- | ----------- |
| context\request\extractor.rs  | 30            | 2               | 93.33%      | 2  | 0  | 100.00% | 28  | 0  | 100.00% | 0 | 0 | - |
| context\request\extensions.rs | 119           | 20              | 83.19%      | 10 | 0  | 100.00% | 78  | 5  | 93.59%  | 0 | 0 | - |
| context\template.rs           | 255           | 82              | 67.84%      | 27 | 10 | 62.96%  | 175 | 47 | 73.14%  | 0 | 0 | - |
| context\tera\form.rs          | 273           | 63              | 76.92%      | 38 | 10 | 73.68%  | 165 | 35 | 78.79%  | 0 | 0 | - |
| context\tera\static_tera.rs   | 125           | 34              | 72.80%      | 8  | 3  | 62.50%  | 65  | 16 | 75.38%  | 0 | 0 | - |
| context\tera\url.rs           | 100           | 42              | 58.00%      | 9  | 2  | 77.78%  | 54  | 21 | 61.11%  | 0 | 0 | - |

#### DB

| Fichier      | Regions Cover | Functions Cover | Lines Cover |
| ------------ | ------------- | --------------- | ----------- |
| db\config.rs | 384           | 72              | 81.25%      | 24 | 0 | 100.00% | 214 | 46 | 78.50% | 0 | 0 | - |

#### Engine

| Fichier        | Regions Cover | Functions Cover | Lines Cover |
| -------------- | ------------- | --------------- | ----------- |
| engine\core.rs | 65            | 46              | 29.23%      | 2 | 1 | 50.00% | 49 | 34 | 30.61% | 0 | 0 | - |

#### Errors

| Fichier         | Regions Cover | Functions Cover | Lines Cover |
| --------------- | ------------- | --------------- | ----------- |
| errors\error.rs | 423           | 100             | 76.36%      | 28 | 5 | 82.14% | 289 | 35 | 87.89% | 0 | 0 | - |

#### Flash

| Fichier                | Regions Cover | Functions Cover | Lines Cover |
| ---------------------- | ------------- | --------------- | ----------- |
| flash\flash_manager.rs | 71            | 0               | 100.00%     | 14 | 0 | 100.00% | 49 | 0 | 100.00% | 0 | 0 | - |
| flash\flash_struct.rs  | 27            | 0               | 100.00%     | 6  | 0 | 100.00% | 37 | 0 | 100.00% | 0 | 0 | - |

#### Forms

| Fichier                      | Regions Cover | Functions Cover | Lines Cover |
| ---------------------------- | ------------- | --------------- | ----------- |
| forms\base.rs                | 174           | 6               | 96.55%      | 34 | 2  | 94.12%  | 130 | 6   | 95.38%  | 0 | 0 | - |
| forms\extractor.rs           | 125           | 10              | 92.00%      | 12 | 2  | 83.33%  | 84  | 4   | 95.24%  | 0 | 0 | - |
| forms\field.rs               | 151           | 124             | 17.88%      | 25 | 18 | 28.00%  | 85  | 66  | 22.35%  | 0 | 0 | - |
| forms\fields\boolean.rs      | 81            | 38              | 53.09%      | 11 | 3  | 72.73%  | 54  | 22  | 59.26%  | 0 | 0 | - |
| forms\fields\choice.rs       | 348           | 87              | 75.00%      | 45 | 9  | 80.00%  | 239 | 56  | 76.57%  | 0 | 0 | - |
| forms\fields\datetime.rs     | 623           | 130             | 79.13%      | 53 | 8  | 84.91%  | 434 | 78  | 82.03%  | 0 | 0 | - |
| forms\fields\file.rs         | 555           | 376             | 32.25%      | 50 | 26 | 48.00%  | 337 | 211 | 37.39%  | 0 | 0 | - |
| forms\fields\hidden.rs       | 83            | 27              | 67.47%      | 9  | 2  | 77.78%  | 57  | 16  | 71.93%  | 0 | 0 | - |
| forms\fields\number.rs       | 270           | 38              | 85.93%      | 19 | 2  | 89.47%  | 205 | 23  | 88.78%  | 0 | 0 | - |
| forms\fields\special.rs      | 518           | 143             | 72.39%      | 98 | 13 | 86.73%  | 366 | 91  | 75.14%  | 0 | 0 | - |
| forms\fields\text.rs         | 286           | 33              | 88.46%      | 29 | 4  | 86.21%  | 183 | 15  | 91.80%  | 0 | 0 | - |
| forms\form.rs                | 673           | 254             | 62.26%      | 62 | 6  | 90.32%  | 338 | 92  | 72.78%  | 0 | 0 | - |
| forms\generic.rs             | 92            | 25              | 72.83%      | 26 | 7  | 73.08%  | 78  | 21  | 73.08%  | 0 | 0 | - |
| forms\model_form\mod.rs      | 13            | 7               | 46.15%      | 3  | 1  | 66.67%  | 9   | 3   | 66.67%  | 0 | 0 | - |
| forms\options\bool_choice.rs | 3             | 0               | 100.00%     | 1  | 0  | 100.00% | 3   | 0   | 100.00% | 0 | 0 | - |
| forms\prisme\aeigs.rs        | 110           | 47              | 57.27%      | 10 | 4  | 60.00%  | 77  | 29  | 62.34%  | 0 | 0 | - |
| forms\prisme\csrf_gate.rs    | 51            | 2               | 96.08%      | 5  | 0  | 100.00% | 33  | 1   | 96.97%  | 0 | 0 | - |
| forms\prisme\rules.rs        | 71            | 0               | 100.00%     | 11 | 0  | 100.00% | 77  | 0   | 100.00% | 0 | 0 | - |
| forms\prisme\sentinel.rs     | 16            | 0               | 100.00%     | 1  | 0  | 100.00% | 7   | 0   | 100.00% | 0 | 0 | - |
| forms\renderer.rs            | 116           | 13              | 88.79%      | 9  | 1  | 88.89%  | 73  | 2   | 97.26%  | 0 | 0 | - |
| forms\validator.rs           | 97            | 1               | 98.97%      | 9  | 0  | 100.00% | 56  | 1   | 98.21%  | 0 | 0 | - |

#### Macros

| Fichier                       | Regions Cover | Functions Cover | Lines Cover |
| ----------------------------- | ------------- | --------------- | ----------- |
| macros\bdd\objects.rs         | 243           | 100             | 58.85%      | 26 | 11 | 57.69%  | 169 | 71 | 57.99%  | 0 | 0 | - |
| macros\bdd\query.rs           | 300           | 127             | 57.67%      | 31 | 12 | 61.29%  | 201 | 86 | 57.21%  | 0 | 0 | - |
| macros\context\flash.rs       | 96            | 0               | 100.00%     | 19 | 0  | 100.00% | 65  | 0  | 100.00% | 0 | 0 | - |
| macros\context\helper.rs      | 31            | 3               | 90.32%      | 6  | 1  | 83.33%  | 26  | 3  | 88.46%  | 0 | 0 | - |
| macros\context\impl_error.rs  | 8             | 0               | 100.00%     | 2  | 0  | 100.00% | 2   | 0  | 100.00% | 0 | 0 | - |
| macros\forms\enum_kind.rs     | 3             | 0               | 100.00%     | 1  | 0  | 100.00% | 3   | 0  | 100.00% | 0 | 0 | - |
| macros\forms\impl_form.rs     | 9             | 0               | 100.00%     | 3  | 0  | 100.00% | 9   | 0  | 100.00% | 0 | 0 | - |
| macros\router\register_url.rs | 73            | 10              | 86.30%      | 12 | 5  | 58.33%  | 50  | 5  | 90.00%  | 0 | 0 | - |
| macros\router\router_ext.rs   | 45            | 0               | 100.00%     | 2  | 0  | 100.00% | 34  | 0  | 100.00% | 0 | 0 | - |

#### Middleware

| Fichier                                 | Regions Cover | Functions Cover | Lines Cover |
| --------------------------------------- | ------------- | --------------- | ----------- |
| middleware\auth\auth_session.rs         | 269           | 62              | 76.95%      | 34 | 6  | 82.35%  | 190 | 39  | 79.47%  | 0 | 0 | - |
| middleware\auth\default_auth.rs         | 8             | 2               | 75.00%      | 3  | 1  | 66.67%  | 8   | 2   | 75.00%  | 0 | 0 | - |
| middleware\auth\form_login.rs           | 14            | 0               | 100.00%     | 1  | 0  | 100.00% | 10  | 0   | 100.00% | 0 | 0 | - |
| middleware\auth\login_guard.rs          | 132           | 47              | 64.39%      | 14 | 4  | 71.43%  | 83  | 23  | 72.29%  | 0 | 0 | - |
| middleware\auth\reset.rs                | 351           | 351             | 0.00%       | 23 | 23 | 0.00%   | 250 | 250 | 0.00%   | 0 | 0 | - |
| middleware\auth\user.rs                 | 61            | 6               | 90.16%      | 17 | 3  | 82.35%  | 55  | 9   | 83.64%  | 0 | 0 | - |
| middleware\auth\user_trait.rs           | 11            | 0               | 100.00%     | 2  | 0  | 100.00% | 6   | 0   | 100.00% | 0 | 0 | - |
| middleware\config.rs                    | 45            | 3               | 93.33%      | 12 | 1  | 91.67%  | 72  | 1   | 98.61%  | 0 | 0 | - |
| middleware\dev\cache.rs                 | 46            | 28              | 39.13%      | 5  | 2  | 60.00%  | 31  | 19  | 38.71%  | 0 | 0 | - |
| middleware\errors\error.rs              | 293           | 111             | 62.12%      | 19 | 7  | 63.16%  | 369 | 95  | 74.25%  | 0 | 0 | - |
| middleware\rate_limit.rs                | 137           | 29              | 78.83%      | 19 | 3  | 84.21%  | 94  | 14  | 85.11%  | 0 | 0 | - |
| middleware\security\allowed_hosts.rs    | 231           | 25              | 89.18%      | 16 | 3  | 81.25%  | 107 | 16  | 85.05%  | 0 | 0 | - |
| middleware\security\csp.rs              | 407           | 19              | 95.33%      | 19 | 1  | 94.74%  | 232 | 2   | 99.14%  | 0 | 0 | - |
| middleware\security\csrf.rs             | 189           | 51              | 73.02%      | 12 | 3  | 75.00%  | 120 | 27  | 77.50%  | 0 | 0 | - |
| middleware\session\cleaning_store.rs    | 172           | 57              | 66.86%      | 30 | 5  | 83.33%  | 127 | 35  | 72.44%  | 0 | 0 | - |
| middleware\session\session_parametre.rs | 7             | 0               | 100.00%     | 2  | 0  | 100.00% | 10  | 0   | 100.00% | 0 | 0 | - |

#### Migration

| Fichier                           | Regions Cover | Functions Cover | Lines Cover |
| --------------------------------- | ------------- | --------------- | ----------- |
| migration\column\mod.rs           | 550           | 66              | 88.00%      | 54 | 0  | 100.00% | 358 | 21  | 94.13%  | 0 | 0 | - |
| migration\foreign_key\mod.rs      | 42            | 0               | 100.00%     | 6  | 0  | 100.00% | 39  | 0   | 100.00% | 0 | 0 | - |
| migration\hooks\mod.rs            | 45            | 0               | 100.00%     | 7  | 0  | 100.00% | 30  | 0   | 100.00% | 0 | 0 | - |
| migration\index\mod.rs            | 45            | 0               | 100.00%     | 6  | 0  | 100.00% | 29  | 0   | 100.00% | 0 | 0 | - |
| migration\makemigrations.rs       | 598           | 108             | 81.94%      | 29 | 6  | 79.31%  | 324 | 61  | 81.17%  | 0 | 0 | - |
| migration\migrate.rs              | 714           | 552             | 22.69%      | 38 | 23 | 39.47%  | 391 | 297 | 24.04%  | 0 | 0 | - |
| migration\primary_key\mod.rs      | 42            | 1               | 97.62%      | 7  | 0  | 100.00% | 38  | 0   | 100.00% | 0 | 0 | - |
| migration\relation\mod.rs         | 18            | 0               | 100.00%     | 4  | 0  | 100.00% | 31  | 0   | 100.00% | 0 | 0 | - |
| migration\schema\mod.rs           | 376           | 126             | 66.49%      | 28 | 7  | 75.00%  | 247 | 90  | 63.56%  | 0 | 0 | - |
| migration\utils\convertisseur.rs  | 21            | 1               | 95.24%      | 3  | 0  | 100.00% | 14  | 1   | 92.86%  | 0 | 0 | - |
| migration\utils\diff.rs           | 150           | 0               | 100.00%     | 20 | 0  | 100.00% | 118 | 0   | 100.00% | 0 | 0 | - |
| migration\utils\generators.rs     | 505           | 181             | 64.16%      | 27 | 3  | 88.89%  | 431 | 154 | 64.27%  | 0 | 0 | - |
| migration\utils\helpers.rs        | 651           | 187             | 71.27%      | 19 | 0  | 100.00% | 343 | 102 | 70.26%  | 0 | 0 | - |
| migration\utils\runner_builder.rs | 513           | 87              | 83.04%      | 14 | 0  | 100.00% | 308 | 45  | 85.39%  | 0 | 0 | - |
| migration\utils\runner_season.rs  | 307           | 102             | 66.78%      | 15 | 7  | 53.33%  | 204 | 61  | 70.10%  | 0 | 0 | - |
| migration\utils\runner.rs         | 356           | 0               | 100.00%     | 47 | 0  | 100.00% | 192 | 0   | 100.00% | 0 | 0 | - |
| migration\utils\sql.rs            | 17            | 0               | 100.00%     | 1  | 0  | 100.00% | 10  | 0   | 100.00% | 0 | 0 | - |

#### Utils

| Fichier                           | Regions Cover | Functions Cover | Lines Cover |
| --------------------------------- | ------------- | --------------- | ----------- |
| utils\alliance\mod.rs             | 13            | 0               | 100.00%     | 3        | 0       | 100.00%    | 9         | 0        | 100.00%    | 0     | 0     | -     |
| utils\cli\mod.rs                  | 141           | 141             | 0.00%       | 7        | 7       | 0.00%      | 149       | 149      | 0.00%      | 0     | 0     | -     |
| utils\config\auto_field.rs        | 18            | 0               | 100.00%     | 4        | 0       | 100.00%    | 16        | 0        | 100.00%    | 0     | 0     | -     |
| utils\config\lecture_env.rs       | 7             | 0               | 100.00%     | 1        | 0       | 100.00%    | 3         | 0        | 100.00%    | 0     | 0     | -     |
| utils\constante\parse.rs          | 50            | 0               | 100.00%     | 7        | 0       | 100.00%    | 44        | 0        | 100.00%    | 0     | 0     | -     |
| utils\constante\regex_template.rs | 19            | 0               | 100.00%     | 5        | 0       | 100.00%    | 10        | 0        | 100.00%    | 0     | 0     | -     |
| utils\env.rs                      | 25            | 5               | 80.00%      | 6        | 1       | 83.33%     | 19        | 2        | 89.47%     | 0     | 0     | -     |
| utils\forms\parse_html.rs         | 203           | 51              | 74.88%      | 12       | 6       | 50.00%     | 143       | 71       | 50.35%     | 0     | 0     | -     |
| utils\forms\sanitizer.rs          | 102           | 1               | 99.02%      | 8        | 0       | 100.00%    | 68        | 0        | 100.00%    | 0     | 0     | -     |
| utils\init_error\init.rs          | 4             | 0               | 100.00%     | 1        | 0       | 100.00%    | 3         | 0        | 100.00%    | 0     | 0     | -     |
| utils\mailer\mod.rs               | 194           | 194             | 0.00%       | 25       | 25      | 0.00%      | 123       | 123      | 0.00%      | 0     | 0     | -     |
| utils\middleware\vsp_nonce.rs     | 54            | 0               | 100.00%     | 6        | 0       | 100.00%    | 30        | 0        | 100.00%    | 0     | 0     | -     |
| utils\middleware\csrf.rs          | 140           | 0               | 100.00%     | 11       | 0       | 100.00%    | 72        | 0        | 100.00%    | 0     | 0     | -     |
| utils\password\mod.rs             | 345           | 121             | 64.93%      | 38       | 10      | 73.68%     | 231       | 75       | 67.53%     | 0     | 0     | -     |
| utils\reset_token\mod.rs          | 210           | 210             | 0.00%       | 8        | 8       | 0.00%      | 86        | 86       | 0.00%      | 0     | 0     | -     |
| utils\runtime_log.rs              | 80            | 50              | 37.50%      | 15       | 10      | 33.33%     | 69        | 48       | 30.43%     | 0     | 0     | -     |
| utils\trad\switch_lang.rs         | 238           | 23              | 90.34%      | 31       | 9       | 70.97%     | 153       | 12       | 92.16%     | 0     | 0     | -     |
| utils\url_params.rs               | 13            | 13              | 0.00%       | 3        | 3       | 0.00%      | 10        | 10       | 0.00%      | 0     | 0     | -     |
| **TOTAL**                         | **17846**     | **5512**        | **69.11%**  | **1670** | **391** | **76.59%** | **11894** | **3387** | **71.52%** | **0** | **0** | **-** |

```


## Analyse des Données Actuelles

---

### Points d'Attention (Couverture < 50%)

| Fichier                   | Regions | Functions | Lines  | Priorité   |
| ------------------------- | ------- | --------- | ------ | ---------- |
| `bin\runique.rs`          | 0.00%   | 0.00%     | 0.00%  | 🔴 Critique |
| `migration\migrate.rs`    | 22.69%  | 39.47%    | 24.04% | 🔴 Critique |
| `engine\core.rs`          | 29.23%  | 50.00%    | 30.61% | 🟠 Haute    |
| `forms\fields\file.rs`    | 32.25%  | 48.00%    | 37.39% | 🟠 Haute    |
| `middleware\dev\cache.rs` | 39.13%  | 60.00%    | 38.71% | 🟠 Haute    |
| `forms\model_form\mod.rs` | 46.15%  | 66.67%    | 66.67% | 🟡 Moyenne  |
| `forms\field.rs`          | 17.88%  | 28.00%    | 22.35% | 🔴 Critique |

**⚠️ Nouveaux fichiers critiques identifiés :**
- `forms\field.rs` est passé de 52.94% à **17.88%** (régression majeure)
- `forms\fields\file.rs` a baissé (40% → 32.25%)

---

### Fichiers 100% Couverts ✅

| Fichier                                   | Commentaire                    |
| ----------------------------------------- | ------------------------------ |
| `app\staging\csp_config.rs`               | ✅ Maintenu                     |
| `app\staging\host_config.rs`              | ✅ Maintenu                     |
| `app\staging\static_staging.rs`           | ✅ Maintenu                     |
| `config\app.rs`                           | ✅ Maintenu                     |
| `config\router.rs`                        | ✅ Maintenu                     |
| `config\security.rs`                      | ✅ Maintenu                     |
| `config\server.rs`                        | ✅ Maintenu                     |
| `config\settings.rs`                      | ✅ Maintenu                     |
| `flash\flash_manager.rs`                  | ✅ Maintenu                     |
| `flash\flash_struct.rs`                   | ✅ Maintenu                     |
| `forms\options\bool_choice.rs`            | ✅ Maintenu                     |
| `forms\prisme\rules.rs`                   | ✅ Maintenu                     |
| `forms\prisme\sentinel.rs`                | ✅ Maintenu                     |
| `forms\prisme\csrf_gate.rs`               | 🆕 **Nouveau** (96.08% → 100%)  |
| `forms\validator.rs`                      | ✅ Maintenu                     |
| `macros\context\flash.rs`                 | ✅ Maintenu                     |
| `macros\context\impl_error.rs`            | ✅ Maintenu                     |
| `macros\forms\enum_kind.rs`               | ✅ Maintenu                     |
| `macros\forms\impl_form.rs`               | ✅ Maintenu                     |
| `macros\router\router_ext.rs`             | 🆕 **Nouveau**                  |
| `middleware\auth\form_login.rs`           | 🆕 **Nouveau**                  |
| `middleware\auth\user_trait.rs`           | ✅ Maintenu                     |
| `middleware\session\session_parametre.rs` | ✅ Maintenu                     |
| `migration\foreign_key\mod.rs`            | ✅ Maintenu                     |
| `migration\hooks\mod.rs`                  | ✅ Maintenu                     |
| `migration\index\mod.rs`                  | ✅ Maintenu                     |
| `migration\primary_key\mod.rs`            | ✅ Maintenu                     |
| `migration\relation\mod.rs`               | ✅ Maintenu                     |
| `migration\utils\diff.rs`                 | ✅ Maintenu                     |
| `migration\utils\sql.rs`                  | 🆕 **Nouveau**                  |
| `utils\alliance\mod.rs`                   | 🆕 **Nouveau**                  |
| `utils\config\auto_field.rs`              | 🆕 **Nouveau**                  |
| `utils\config\lecture_env.rs`             | ✅ Maintenu                     |
| `utils\constante\parse.rs`                | ✅ Maintenu                     |
| `utils\constante\regex_template.rs`       | ✅ Maintenu                     |
| `utils\forms\sanitizer.rs`                | ✅ Maintenu                     |
| `utils\init_error\init.rs`                | 🆕 **Nouveau**                  |
| `utils\middleware\vsp_nonce.rs`           | 🆕 **Nouveau** (nom mis à jour) |
| `utils\middleware\csrf.rs`                | ✅ Maintenu                     |

---

### Progressions Notables (Évolution Globale)

| Métrique      | Précédent | Actuel     | Évolution |
| ------------- | --------- | ---------- | --------- |
| **Regions**   | 75.38%    | **69.11%** | 📉 -6.27%  |
| **Functions** | 82.83%    | **76.59%** | 📉 -6.24%  |
| **Lines**     | 78.35%    | **71.52%** | 📉 -6.83%  |

**⚠️ Régression globale significative** - La couverture a baissé de ~6-7% sur tous les axes.

---

### Fichiers avec Progression Positive

| Fichier                        | Avant  | Après (Functions) | Évolution |
| ------------------------------ | ------ | ----------------- | --------- |
| `middleware\security\csp.rs`   | 66.67% | **94.74%**        | 📈 +28%    |
| `forms\prisme\csrf_gate.rs`    | ~50%   | **100.00%**       | 📈 +50%    |
| `config\static_files.rs`       | 88.24% | **88.24%**        | → Stable  |
| `context\request\extractor.rs` | 93.33% | **100.00%**       | 📈 +6.67%  |

---

### Ce qui Reste à Couvrir (Priorités Actualisées)

#### 🔴 Priorité Critique — Effort Important

| Fichier                    | Functions | Bloquant                   | Note                                 |
| -------------------------- | --------- | -------------------------- | ------------------------------------ |
| `bin\runique.rs`           | 0.00%     | CLI complet                | **Aucun test** — Difficile à tester  |
| `migration\migrate.rs`     | 39.47%    | Commandes CLI sea-orm      | Régression depuis 41.03%             |
| `forms\field.rs`           | 28.00%    | Champs de formulaires      | **Régression majeure** (était à 58%) |
| `utils\cli\mod.rs`         | 0.00%     | Utilitaires CLI            | **Nouveau** — Aucun test             |
| `utils\mailer\mod.rs`      | 0.00%     | Envoi d'emails             | **Nouveau** — Aucun test             |
| `utils\reset_token\mod.rs` | 0.00%     | Tokens de réinitialisation | **Nouveau** — Aucun test             |
| `utils\url_params.rs`      | 0.00%     | Paramètres URL             | **Nouveau** — Aucun test             |
| `middleware\auth\reset.rs` | 0.00%     | Réinitialisation auth      | **Nouveau** — Aucun test             |

#### 🟠 Priorité Haute — Effort Moyen

| Fichier                      | Functions | Cible | Note                          |
| ---------------------------- | --------- | ----- | ----------------------------- |
| `engine\core.rs`             | 50.00%    | 80%   | Initialisation app complète   |
| `middleware\dev\cache.rs`    | 60.00%    | 85%   | Cache HTTP conditionnel       |
| `forms\fields\file.rs`       | 48.00%    | 80%   | Upload multipart (régression) |
| `forms\model_form\mod.rs`    | 66.67%    | 85%   | Formulaires de modèles        |
| `middleware\errors\error.rs` | 63.16%    | 85%   | Rendu erreurs HTTP            |
| `utils\runtime_log.rs`       | 33.33%    | 70%   | **Nouveau** — Logging runtime |

#### 🟡 Priorité Moyenne — Testable Unitairement

| Fichier                         | Functions | Cible | Évolution                |
| ------------------------------- | --------- | ----- | ------------------------ |
| `utils\password\mod.rs`         | 73.68%    | 90%   | Stable                   |
| `migration\schema\mod.rs`       | 75.00%    | 90%   | Stable                   |
| `migration\utils\generators.rs` | 88.89%    | 95%   | 📈 +0.89%                 |
| `forms\form.rs`                 | 90.32%    | 95%   | 📉 -1.35%                 |
| `utils\trad\switch_lang.rs`     | 70.97%    | 90%   | Stable                   |
| `forms\fields\special.rs`       | 86.73%    | 95%   | 📈 +9.14%                 |
| `context\template.rs`           | 62.96%    | 85%   | Régression depuis 80.95% |
| `forms\fields\boolean.rs`       | 72.73%    | 90%   | **Nouveau**              |
| `forms\fields\hidden.rs`        | 77.78%    | 90%   | **Nouveau**              |
| `forms\fields\url.rs`           | 77.78%    | 90%   | **Nouveau**              |

---
