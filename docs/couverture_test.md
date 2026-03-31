Voici ton rapport avec la mise en page corrigée pour plus de lisibilité et de cohérence des tableaux, sans aucune modification du contenu :

---

## Rapport de Couverture de Code

> Mis à jour le 2026-03-31

**Résumé des résultats :**

* ✅ **1833 tests passés** en 49.89s

| Métrique      | Total  | Manqué | Couverture | Précédent | Évolution |
| ------------- | ------ | ------ | ---------- | --------- | --------- |
| **Regions**   | 17,846 | 5,512  | **69.11%** | 75.38%    | -6.27%    |
| **Functions** | 1,670  | 391    | **76.59%** | 82.83%    | -6.24%    |
| **Lines**     | 11,894 | 3,387  | **71.52%** | 78.35%    | -6.83%    |
| **Branches**  | 0      | 0      | -          | -         | -         |

---

### Détail par Fichier

#### App

| Fichier                           | Regions Cover | Functions Cover | Lines Cover |
| --------------------------------- | ------------- | --------------- | ----------- |
| app\builder.rs                    | 286           | 53              | 81.47%      |
| app\error_build.rs                | 127           | 19              | 85.04%      |
| app\staging\core_staging.rs       | 65            | 21              | 67.69%      |
| app\staging\csp_config.rs         | 119           | 0               | 100.00%     |
| app\staging\host_config.rs        | 19            | 0               | 100.00%     |
| app\staging\middleware_staging.rs | 410           | 115             | 71.95%      |
| app\staging\static_staging.rs     | 27            | 0               | 100.00%     |
| app\templates.rs                  | 166           | 37              | 77.71%      |

#### Bin

**Cli difficile à tester**

| Fichier        | Regions Cover | Functions Cover | Lines Cover |
| -------------- | ------------- | --------------- | ----------- |
| bin\runique.rs | 145           | 145             | 0.00%       |

#### Config

| Fichier                | Regions Cover | Functions Cover | Lines Cover |
| ---------------------- | ------------- | --------------- | ----------- |
| config\app.rs          | 19            | 0               | 100.00%     |
| config\router.rs       | 22            | 0               | 100.00%     |
| config\security.rs     | 37            | 0               | 100.00%     |
| config\server.rs       | 29            | 0               | 100.00%     |
| config\settings.rs     | 44            | 0               | 100.00%     |
| config\static_files.rs | 96            | 4               | 95.83%      |

#### Context

| Fichier                       | Regions Cover | Functions Cover | Lines Cover |
| ----------------------------- | ------------- | --------------- | ----------- |
| context\request\extractor.rs  | 30            | 2               | 93.33%      |
| context\request\extensions.rs | 119           | 20              | 83.19%      |
| context\template.rs           | 255           | 82              | 67.84%      |
| context\tera\form.rs          | 273           | 63              | 76.92%      |
| context\tera\static_tera.rs   | 125           | 34              | 72.80%      |
| context\tera\url.rs           | 100           | 42              | 58.00%      |

#### DB

| Fichier      | Regions Cover | Functions Cover | Lines Cover |
| ------------ | ------------- | --------------- | ----------- |
| db\config.rs | 384           | 72              | 81.25%      |

#### Engine

| Fichier        | Regions Cover | Functions Cover | Lines Cover |
| -------------- | ------------- | --------------- | ----------- |
| engine\core.rs | 65            | 46              | 29.23%      |

#### Errors

| Fichier         | Regions Cover | Functions Cover | Lines Cover |
| --------------- | ------------- | --------------- | ----------- |
| errors\error.rs | 423           | 100             | 76.36%      |

#### Flash

| Fichier                | Regions Cover | Functions Cover | Lines Cover |
| ---------------------- | ------------- | --------------- | ----------- |
| flash\flash_manager.rs | 71            | 0               | 100.00%     |
| flash\flash_struct.rs  | 27            | 0               | 100.00%     |

#### Forms

| Fichier                      | Regions Cover | Functions Cover | Lines Cover |
| ---------------------------- | ------------- | --------------- | ----------- |
| forms\base.rs                | 174           | 6               | 96.55%      |
| forms\extractor.rs           | 125           | 10              | 92.00%      |
| forms\field.rs               | 151           | 124             | 17.88%      |
| forms\fields\boolean.rs      | 81            | 38              | 53.09%      |
| forms\fields\choice.rs       | 348           | 87              | 75.00%      |
| forms\fields\datetime.rs     | 623           | 130             | 79.13%      |
| forms\fields\file.rs         | 555           | 376             | 32.25%      |
| forms\fields\hidden.rs       | 83            | 27              | 67.47%      |
| forms\fields\number.rs       | 270           | 38              | 85.93%      |
| forms\fields\special.rs      | 518           | 143             | 72.39%      |
| forms\fields\text.rs         | 286           | 33              | 88.46%      |
| forms\form.rs                | 673           | 254             | 62.26%      |
| forms\generic.rs             | 92            | 25              | 72.83%      |
| forms\model_form\mod.rs      | 13            | 7               | 46.15%      |
| forms\options\bool_choice.rs | 3             | 0               | 100.00%     |
| forms\prisme\aeigs.rs        | 110           | 47              | 57.27%      |
| forms\prisme\csrf_gate.rs    | 51            | 2               | 96.08%      |
| forms\prisme\rules.rs        | 71            | 0               | 100.00%     |
| forms\prisme\sentinel.rs     | 16            | 0               | 100.00%     |
| forms\renderer.rs            | 116           | 13              | 88.79%      |
| forms\validator.rs           | 97            | 1               | 98.97%      |

---

#### TOTAL

| Total Regions | Total Functions | Total Lines | Couverture Regions | Couverture Functions | Couverture Lines |
| ------------- | --------------- | ----------- | ------------------ | -------------------- | ---------------- |
| 17,846        | 1,670           | 11,894      | 69.11%             | 76.59%               | 71.52%           |

---
