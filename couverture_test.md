# Couverture de tests — Runique

> Mis a jour le 2026-03-01
> `cargo llvm-cov --package runique --test mod --ignore-filename-regex "admin"`
> **1 157 tests** — 0 echec

## Resume global

| Metrique   | Couvert | Total  | %      | Precedent |
|------------|---------|--------|--------|-----------|
| Regions    | 7 493   | 13 221 | 56.67% | 51.32%    |
| Fonctions  | 858     | 1 277  | 67.19% | 59.35%    |
| Lignes     | 5 481   | 8 875  | 61.76% | 55.14%    |

---

## Detailed Coverage by File

### App Module

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| app\builder.rs | 205 | 35 | **82.93%** | 19 | 3 | **84.21%** | 129 | 18 | **86.05%** |
| app\error_build.rs | 127 | 19 | **85.04%** | 15 | 0 | **100.00%** | 92 | 5 | **94.57%** |
| app\runique_app.rs | 41 | 41 | 0.00% | 4 | 4 | 0.00% | 23 | 23 | 0.00% |
| app\staging\core_staging.rs | 65 | 21 | 67.69% | 9 | 2 | 77.78% | 59 | 14 | 76.27% |
| app\staging\middleware_staging.rs | 257 | 67 | 73.93% | 29 | 6 | 79.31% | 186 | 39 | 79.03% |
| app\staging\static_staging.rs | 27 | 0 | **100.00%** | 8 | 0 | **100.00%** | 27 | 0 | **100.00%** |
| app\templates.rs | 181 | 70 | 61.33% | 9 | 5 | 44.44% | 117 | 38 | 67.52% |

### Bin & Config

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| bin\runique.rs | 339 | 339 | 0.00% | 9 | 9 | 0.00% | 180 | 180 | 0.00% |
| config\app.rs | 19 | 0 | 100.00% | 3 | 0 | 100.00% | 15 | 0 | 100.00% |
| config\router.rs | 22 | 0 | **100.00%** | 4 | 0 | **100.00%** | 16 | 0 | **100.00%** |
| config\security.rs | 44 | 0 | 100.00% | 8 | 0 | 100.00% | 31 | 0 | 100.00% |
| config\server.rs | 24 | 0 | 100.00% | 4 | 0 | 100.00% | 14 | 0 | 100.00% |
| config\settings.rs | 39 | 0 | 100.00% | 2 | 0 | 100.00% | 24 | 0 | 100.00% |
| config\static_files.rs | 80 | 0 | 100.00% | 15 | 0 | 100.00% | 51 | 0 | 100.00% |

### Context Module

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| context\request\extractor.rs | 60 | 60 | 0.00% | 4 | 4 | 0.00% | 44 | 44 | 0.00% |
| context\request_extensions.rs | 119 | 77 | 35.29% | 10 | 6 | 40.00% | 78 | 43 | 44.87% |
| context\template.rs | 200 | 200 | 0.00% | 18 | 18 | 0.00% | 135 | 135 | 0.00% |
| context\tera\csp.rs | 11 | 0 | **100.00%** | 1 | 0 | **100.00%** | 8 | 0 | **100.00%** |
| context\tera\form.rs | 269 | 63 | 76.58% | 38 | 10 | 73.68% | 161 | 35 | 78.26% |
| context\tera\static_tera.rs | 84 | 6 | **92.86%** | 7 | 2 | **71.43%** | 44 | 2 | **95.45%** |
| context\tera\url.rs | 57 | 2 | 96.49% | 7 | 0 | 100.00% | 32 | 1 | 96.88% |

### Database & Engine

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| db\config.rs | 214 | 214 | 0.00% | 22 | 22 | 0.00% | 153 | 153 | 0.00% |
| engine\core.rs | 57 | 39 | 31.58% | 2 | 1 | 50.00% | 42 | 28 | 33.33% |

### Errors & Flash

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| errors\error.rs | 383 | 246 | 35.77% | 31 | 19 | 38.71% | 283 | 147 | 48.06% |
| flash\flash_manager.rs | 71 | 0 | 100.00% | 14 | 0 | 100.00% | 49 | 0 | 100.00% |
| flash\flash_struct.rs | 27 | 0 | 100.00% | 6 | 0 | 100.00% | 37 | 0 | 100.00% |

### Forms Module

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| forms\base.rs | 174 | 16 | **90.80%** | 34 | 5 | **85.29%** | 130 | 13 | **90.00%** |
| forms\extractor.rs | 89 | 89 | 0.00% | 10 | 10 | 0.00% | 65 | 65 | 0.00% |
| forms\field.rs | 32 | 5 | **84.38%** | 8 | 1 | **87.50%** | 22 | 3 | **86.36%** |
| forms\fields\boolean.rs | 89 | 33 | 62.92% | 12 | 3 | 75.00% | 59 | 16 | 72.88% |
| forms\fields\choice.rs | 332 | 96 | 71.08% | 45 | 13 | 71.11% | 221 | 57 | 74.21% |
| forms\fields\datetime.rs | 587 | 359 | 38.84% | 53 | 30 | 43.40% | 410 | 233 | 43.17% |
| forms\fields\file.rs | 370 | 215 | 41.89% | 35 | 14 | 60.00% | 238 | 131 | 44.96% |
| forms\fields\hidden.rs | 66 | 13 | 80.30% | 9 | 2 | 77.78% | 48 | 7 | 85.42% |
| forms\fields\number.rs | 263 | 124 | 52.85% | 19 | 5 | 73.68% | 201 | 85 | 57.71% |
| forms\fields\special.rs | 489 | 115 | 76.48% | 58 | 13 | 77.59% | 341 | 64 | 81.23% |
| forms\fields\text.rs | 285 | 19 | **93.33%** | 30 | 3 | **90.00%** | 179 | 8 | **95.53%** |
| forms\form.rs | 601 | 231 | **61.56%** | 58 | 5 | **91.38%** | 301 | 87 | **71.10%** |
| forms\generic.rs | 92 | 54 | 41.30% | 26 | 15 | 42.31% | 78 | 45 | 42.31% |
| forms\model_form\mod.rs | 13 | 13 | 0.00% | 3 | 3 | 0.00% | 9 | 9 | 0.00% |
| forms\options\bool_choice.rs | 3 | 0 | **100.00%** | 1 | 0 | **100.00%** | 3 | 0 | **100.00%** |
| forms\prisme\aegis.rs | 59 | 19 | **67.80%** | 6 | 2 | **66.67%** | 41 | 7 | **82.93%** |
| forms\prisme\csrf_gate.rs | 32 | 1 | **96.88%** | 4 | 0 | **100.00%** | 22 | 0 | **100.00%** |
| forms\prisme\rules.rs | 69 | 0 | 100.00% | 11 | 0 | 100.00% | 69 | 0 | 100.00% |
| forms\prisme\sentinel.rs | 16 | 0 | 100.00% | 1 | 0 | 100.00% | 7 | 0 | 100.00% |
| forms\renderer.rs | 113 | 11 | **90.27%** | 9 | 1 | **88.89%** | 73 | 2 | **97.26%** |
| forms\validator.rs | 93 | 6 | **93.55%** | 9 | 0 | **100.00%** | 56 | 4 | **92.86%** |

### Macros

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| macros\bdd\objects.rs | 99 | 99 | 0.00% | 14 | 14 | 0.00% | 75 | 75 | 0.00% |
| macros\bdd\query.rs | 123 | 123 | 0.00% | 17 | 17 | 0.00% | 86 | 86 | 0.00% |
| macros\context\helper.rs | 31 | 3 | 90.32% | 6 | 1 | 83.33% | 26 | 3 | 88.46% |
| macros\context\impl_error.rs | 8 | 8 | 0.00% | 2 | 2 | 0.00% | 2 | 2 | 0.00% |
| macros\forms\enum_kind.rs | 3 | 0 | 100.00% | 1 | 0 | 100.00% | 3 | 0 | 100.00% |
| macros\forms\impl_form.rs | 9 | 0 | **100.00%** | 3 | 0 | **100.00%** | 9 | 0 | **100.00%** |
| macros\routeur\register_url.rs | 63 | 5 | **92.06%** | 7 | 0 | **100.00%** | 36 | 2 | **94.44%** |

### Middleware

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| middleware\auth\auth_session.rs | 197 | 12 | 93.91% | 26 | 0 | 100.00% | 135 | 0 | 100.00% |
| middleware\auth\default_auth.rs | 8 | 2 | 75.00% | 3 | 1 | 66.67% | 8 | 2 | 75.00% |
| middleware\auth\form\login.rs | 14 | 0 | **100.00%** | 1 | 0 | **100.00%** | 10 | 0 | **100.00%** |
| middleware\auth\user.rs | 46 | 4 | 91.30% | 13 | 2 | 84.62% | 40 | 4 | 90.00% |
| middleware\auth\user_trait.rs | 11 | 0 | 100.00% | 2 | 0 | 100.00% | 6 | 0 | 100.00% |
| middleware\config.rs | 48 | 3 | 93.75% | 12 | 1 | 91.67% | 67 | 1 | 98.51% |
| middleware\dev\cache.rs | 47 | 28 | 40.43% | 5 | 2 | 60.00% | 32 | 19 | 40.62% |
| middleware\errors\error.rs | 132 | 42 | 68.18% | 15 | 5 | 66.67% | 245 | 20 | 91.84% |
| middleware\security\allowed_hosts.rs | 110 | 21 | 80.91% | 14 | 3 | 78.57% | 82 | 16 | 80.49% |
| middleware\security\csp.rs | 355 | 108 | 69.58% | 18 | 6 | 66.67% | 191 | 72 | 62.30% |
| middleware\security\csrf.rs | 124 | 18 | 85.48% | 8 | 1 | 87.50% | 79 | 10 | 87.34% |
| middleware\session\session_parametre.rs | 7 | 0 | 100.00% | 2 | 0 | 100.00% | 10 | 0 | 100.00% |

### Migration Module

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| migration\column\mod.rs | 545 | 222 | 59.27% | 54 | 11 | 79.63% | 355 | 105 | 70.42% |
| migration\foreign_key\mod.rs | 42 | 0 | 100.00% | 6 | 0 | 100.00% | 39 | 0 | 100.00% |
| migration\hooks\mod.rs | 45 | 0 | 100.00% | 7 | 0 | 100.00% | 30 | 0 | 100.00% |
| migration\index\mod.rs | 45 | 0 | 100.00% | 6 | 0 | 100.00% | 29 | 0 | 100.00% |
| migration\makemigrations.rs | 394 | 350 | 11.17% | 26 | 20 | 23.08% | 199 | 175 | 12.06% |
| migration\migrate.rs | 654 | 654 | 0.00% | 38 | 38 | 0.00% | 374 | 374 | 0.00% |
| migration\primary_key\mod.rs | 42 | 1 | 97.62% | 7 | 0 | 100.00% | 38 | 0 | 100.00% |
| migration\relation\mod.rs | 18 | 0 | 100.00% | 4 | 0 | 100.00% | 31 | 0 | 100.00% |
| migration\schema\mod.rs | 360 | 110 | 69.44% | 24 | 3 | 87.50% | 237 | 80 | 66.24% |
| migration\utils\convertisseur.rs | 21 | 1 | 95.24% | 3 | 0 | 100.00% | 14 | 1 | 92.86% |
| migration\utils\diff.rs | 149 | 24 | 83.89% | 20 | 8 | 60.00% | 117 | 18 | 84.62% |
| migration\utils\generators.rs | 440 | 154 | 65.00% | 25 | 3 | 88.00% | 387 | 142 | 63.31% |
| migration\utils\helpers.rs | 652 | 213 | 67.33% | 19 | 0 | 100.00% | 343 | 114 | 66.76% |
| migration\utils\parser_builder.rs | 310 | 88 | 71.61% | 11 | 0 | 100.00% | 165 | 22 | 86.67% |
| migration\utils\parser_seaorm.rs | 299 | 186 | 37.79% | 15 | 8 | 46.67% | 196 | 107 | 45.41% |
| migration\utils\paths.rs | 56 | 20 | 64.29% | 14 | 5 | 64.29% | 46 | 17 | 63.04% |
| migration\utils\types.rs | 17 | 0 | 100.00% | 1 | 0 | 100.00% | 10 | 0 | 100.00% |

### Utils Module

| Fichier | Regions | Manquees | Cover | Fonctions | Manquees | Exec | Lignes | Manquees | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| utils\aliases\helpers.rs | 13 | 0 | **100.00%** | 3 | 0 | **100.00%** | 9 | 0 | **100.00%** |
| utils\config\autofield.rs | 18 | 0 | 100.00% | 4 | 0 | 100.00% | 16 | 0 | 100.00% |
| utils\config\lecture_env.rs | 7 | 0 | 100.00% | 1 | 0 | 100.00% | 3 | 0 | 100.00% |
| utils\constante\parse.rs | 50 | 0 | 100.00% | 7 | 0 | 100.00% | 44 | 0 | 100.00% |
| utils\forms\parse_html.rs | 99 | 99 | 0.00% | 9 | 9 | 0.00% | 52 | 52 | 0.00% |
| utils\forms\sanitizer.rs | 58 | 1 | 98.28% | 4 | 0 | 100.00% | 36 | 0 | 100.00% |
| utils\init_error\init.rs | 8 | 0 | 100.00% | 1 | 0 | 100.00% | 10 | 0 | 100.00% |
| utils\middleware\csp_nonce.rs | 23 | 23 | 0.00% | 3 | 3 | 0.00% | 14 | 14 | 0.00% |
| utils\middleware\csrf.rs | 139 | 0 | 100.00% | 11 | 0 | 100.00% | 72 | 0 | 100.00% |
| utils\password\mod.rs | 330 | 125 | 62.12% | 38 | 12 | 68.42% | 224 | 80 | 64.29% |
| utils\trad\switch_lang.rs | 63 | 63 | 0.00% | 8 | 8 | 0.00% | 40 | 40 | 0.00% |

---

## Fichiers a 0%

| Fichier |
|---------|
| app\runique_app.rs |
| bin\runique.rs |
| context\request\extractor.rs |
| context\template.rs |
| db\config.rs |
| forms\extractor.rs |
| forms\model_form\mod.rs |
| macros\bdd\objects.rs |
| macros\bdd\query.rs |
| macros\context\impl_error.rs |
| migration\migrate.rs |
| utils\forms\parse_html.rs |
| utils\middleware\csp_nonce.rs |
| utils\trad\switch_lang.rs |

## Fichiers a 100%

| Fichier |
|---------|
| app\staging\static_staging.rs (*) |
| config\app.rs |
| config\router.rs (*) |
| config\security.rs |
| config\server.rs |
| config\settings.rs |
| config\static_files.rs |
| context\tera\csp.rs (*) |
| flash\flash_manager.rs |
| flash\flash_struct.rs |
| forms\options\bool_choice.rs (*) |
| forms\prisme\csrf_gate.rs lignes (*) |
| forms\prisme\rules.rs |
| forms\prisme\sentinel.rs |
| macros\context\flash.rs |
| macros\forms\enum_kind.rs |
| macros\forms\impl_form.rs (*) |
| middleware\auth\auth_session.rs lignes |
| middleware\auth\form\login.rs (*) |
| middleware\auth\user_trait.rs |
| middleware\session\session_parametre.rs |
| migration\foreign_key\mod.rs |
| migration\hooks\mod.rs |
| migration\index\mod.rs |
| migration\primary_key\mod.rs lignes |
| migration\relation\mod.rs |
| migration\utils\types.rs |
| utils\aliases\helpers.rs (*) |
| utils\config\autofield.rs |
| utils\config\lecture_env.rs |
| utils\constante\parse.rs |
| utils\forms\sanitizer.rs lignes |
| utils\init_error\init.rs |
| utils\middleware\csrf.rs |

(*) nouveau 100% depuis la derniere mise a jour
