# Couverture de tests — Runique

> Généré le 2026-03-01 · `cargo llvm-cov --package runique --ignore-filename-regex "admin" --output-format markdown`

## Résumé global

| Métrique   | Couvert | Total | %      |
|------------|---------|-------|--------|
| Régions    | 7269    | 14165 | 51.32% |
| Fonctions  | 809     | 1363  | 59.35% |
| Lignes     | 5163    | 9363  | 55.14% |

---

## Detailed Coverage by File

### App Module

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| app\builder.rs | 205 | 205 | 0.00% | 19 | 19 | 0.00% | 129 | 129 | 0.00% |
| app\error_build.rs | 127 | 127 | 0.00% | 15 | 15 | 0.00% | 92 | 92 | 0.00% |
| app\runique_app.rs | 41 | 41 | 0.00% | 4 | 4 | 0.00% | 23 | 23 | 0.00% |
| app\staging\core_staging.rs | 65 | 65 | 0.00% | 9 | 9 | 0.00% | 59 | 59 | 0.00% |
| app\staging\middleware_staging.rs | 247 | 247 | 0.00% | 26 | 26 | 0.00% | 177 | 177 | 0.00% |
| app\staging\static_staging.rs | 24 | 24 | 0.00% | 7 | 7 | 0.00% | 24 | 24 | 0.00% |
| app\templates.rs | 181 | 181 | 0.00% | 9 | 9 | 0.00% | 117 | 117 | 0.00% |

### Bin & Config

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| bin\runique.rs | 339 | 339 | 0.00% | 9 | 9 | 0.00% | 180 | 180 | 0.00% |
| config\app.rs | 19 | 0 | 100.00% | 3 | 0 | 100.00% | 15 | 0 | 100.00% |
| config\router.rs | 22 | 22 | 0.00% | 4 | 4 | 0.00% | 16 | 16 | 0.00% |
| config\security.rs | 44 | 0 | 100.00% | 8 | 0 | 100.00% | 31 | 0 | 100.00% |
| config\server.rs | 24 | 0 | 100.00% | 4 | 0 | 100.00% | 14 | 0 | 100.00% |
| config\settings.rs | 39 | 0 | 100.00% | 2 | 0 | 100.00% | 24 | 0 | 100.00% |
| config\static_files.rs | 80 | 0 | 100.00% | 15 | 0 | 100.00% | 51 | 0 | 100.00% |

### Context Module

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| context\request\extractor.rs | 60 | 60 | 0.00% | 4 | 4 | 0.00% | 44 | 44 | 0.00% |
| context\request_extensions.rs | 119 | 77 | 35.29% | 10 | 6 | 40.00% | 78 | 43 | 44.87% |
| context\template.rs | 200 | 200 | 0.00% | 18 | 18 | 0.00% | 135 | 135 | 0.00% |
| context\tera\csp.rs | 11 | 11 | 0.00% | 1 | 1 | 0.00% | 8 | 8 | 0.00% |
| context\tera\form.rs | 269 | 63 | 76.58% | 38 | 10 | 73.68% | 161 | 35 | 78.26% |
| context\tera\static_tera.rs | 84 | 84 | 0.00% | 7 | 7 | 0.00% | 44 | 44 | 0.00% |
| context\tera\url.rs | 57 | 2 | 96.49% | 7 | 0 | 100.00% | 32 | 1 | 96.88% |

### Database & Engine

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| db\config.rs | 244 | 175 | 28.28% | 25 | 20 | 20.00% | 171 | 133 | 22.22% |
| engine\core.rs | 57 | 39 | 31.58% | 2 | 1 | 50.00% | 42 | 28 | 33.33% |

### Errors & Flash

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| errors\error.rs | 383 | 246 | 35.77% | 31 | 19 | 38.71% | 283 | 147 | 48.06% |
| flash\flash_manager.rs | 71 | 0 | 100.00% | 14 | 0 | 100.00% | 49 | 0 | 100.00% |
| flash\flash_struct.rs | 27 | 0 | 100.00% | 6 | 0 | 100.00% | 37 | 0 | 100.00% |

### Forms Module

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| forms\base.rs | 174 | 109 | 37.36% | 34 | 22 | 35.29% | 130 | 78 | 40.00% |
| forms\extractor.rs | 89 | 89 | 0.00% | 10 | 10 | 0.00% | 65 | 65 | 0.00% |
| forms\field.rs | 32 | 32 | 0.00% | 8 | 8 | 0.00% | 22 | 22 | 0.00% |
| forms\fields\boolean.rs | 89 | 33 | 62.92% | 12 | 3 | 75.00% | 59 | 16 | 72.88% |
| forms\fields\choice.rs | 332 | 96 | 71.08% | 45 | 13 | 71.11% | 221 | 57 | 74.21% |
| forms\fields\datetime.rs | 587 | 359 | 38.84% | 53 | 30 | 43.40% | 410 | 233 | 43.17% |
| forms\fields\file.rs | 370 | 215 | 41.89% | 35 | 14 | 60.00% | 238 | 131 | 44.96% |
| forms\fields\hidden.rs | 66 | 13 | 80.30% | 9 | 2 | 77.78% | 48 | 7 | 85.42% |
| forms\fields\number.rs | 263 | 124 | 52.85% | 19 | 5 | 73.68% | 201 | 85 | 57.71% |
| forms\fields\special.rs | 489 | 115 | 76.48% | 58 | 13 | 77.59% | 341 | 64 | 81.23% |
| forms\fields\text.rs | 285 | 63 | 77.89% | 30 | 7 | 76.67% | 179 | 34 | 81.01% |
| forms\form.rs | 601 | 465 | 22.63% | 58 | 37 | 36.21% | 301 | 219 | 27.24% |
| forms\generic.rs | 92 | 73 | 20.65% | 26 | 21 | 19.23% | 78 | 63 | 19.23% |
| forms\model_form\mod.rs | 13 | 13 | 0.00% | 3 | 3 | 0.00% | 9 | 9 | 0.00% |
| forms\options\bool_choice.rs | 3 | 3 | 0.00% | 1 | 1 | 0.00% | 3 | 3 | 0.00% |
| forms\prisme\aegis.rs | 59 | 59 | 0.00% | 6 | 6 | 0.00% | 41 | 41 | 0.00% |
| forms\prisme\csrf_gate.rs | 32 | 32 | 0.00% | 4 | 4 | 0.00% | 22 | 22 | 0.00% |
| forms\prisme\rules.rs | 69 | 0 | 100.00% | 11 | 0 | 100.00% | 69 | 0 | 100.00% |
| forms\prisme\sentinel.rs | 16 | 0 | 100.00% | 1 | 0 | 100.00% | 7 | 0 | 100.00% |
| forms\renderer.rs | 113 | 44 | 61.06% | 9 | 3 | 66.67% | 73 | 22 | 69.86% |
| forms\validator.rs | 93 | 46 | 50.54% | 9 | 3 | 66.67% | 56 | 30 | 46.43% |

### Macros

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| macros\bdd\objects.rs | 234 | 89 | 61.97% | 24 | 9 | 62.50% | 154 | 57 | 62.99% |
| macros\bdd\query.rs | 259 | 84 | 67.57% | 25 | 6 | 76.00% | 164 | 50 | 69.51% |
| macros\context\flash.rs | 96 | 0 | 100.00% | 19 | 0 | 100.00% | 65 | 0 | 100.00% |
| macros\context\helper.rs | 31 | 3 | 90.32% | 6 | 1 | 83.33% | 26 | 3 | 88.46% |
| macros\context\impl_error.rs | 8 | 8 | 0.00% | 2 | 2 | 0.00% | 2 | 2 | 0.00% |
| macros\forms\enum_kind.rs | 3 | 0 | 100.00% | 1 | 0 | 100.00% | 3 | 0 | 100.00% |
| macros\forms\impl_form.rs | 9 | 9 | 0.00% | 3 | 3 | 0.00% | 9 | 9 | 0.00% |
| macros\routeur\register_url.rs | 63 | 16 | 74.60% | 7 | 1 | 85.71% | 36 | 7 | 80.56% |

### Middleware

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| middleware\auth\auth_session.rs | 197 | 12 | 93.91% | 26 | 0 | 100.00% | 135 | 0 | 100.00% |
| middleware\auth\default_auth.rs | 8 | 2 | 75.00% | 3 | 1 | 66.67% | 8 | 2 | 75.00% |
| middleware\auth\form\login.rs | 14 | 14 | 0.00% | 1 | 1 | 0.00% | 10 | 10 | 0.00% |
| middleware\auth\user.rs | 46 | 4 | 91.30% | 13 | 2 | 84.62% | 40 | 4 | 90.00% |
| middleware\auth\user_trait.rs | 11 | 0 | 100.00% | 2 | 0 | 100.00% | 6 | 0 | 100.00% |
| middleware\config.rs | 48 | 3 | 93.75% | 12 | 1 | 91.67% | 67 | 1 | 98.51% |
| middleware\dev\cache.rs | 47 | 28 | 40.43% | 5 | 2 | 60.00% | 32 | 19 | 40.62% |
| middleware\errors\error.rs | 132 | 42 | 68.18% | 15 | 5 | 66.67% | 245 | 20 | 91.84% |
| middleware\security\allowed_hosts.rs | 256 | 23 | 91.02% | 21 | 4 | 80.95% | 133 | 17 | 87.22% |
| middleware\security\csp.rs | 355 | 108 | 69.58% | 18 | 6 | 66.67% | 191 | 72 | 62.30% |
| middleware\security\csrf.rs | 124 | 18 | 85.48% | 8 | 1 | 87.50% | 79 | 10 | 87.34% |
| middleware\session\session_parametre.rs | 7 | 0 | 100.00% | 2 | 0 | 100.00% | 10 | 0 | 100.00% |

### Migration Module

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
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
| migration\utils\paths.rs | 356 | 0 | 100.00% | 47 | 0 | 100.00% | 192 | 0 | 100.00% |
| migration\utils\types.rs | 17 | 0 | 100.00% | 1 | 0 | 100.00% | 10 | 0 | 100.00% |

### Utils Module

| Fichier | Régions | Manquées | Cover | Fonctions | Manquées | Exec | Lignes | Manquées | Cover |
|---------|--------:|---------:|------:|----------:|---------:|-----:|-------:|---------:|------:|
| utils\aliases\helpers.rs | 13 | 4 | 69.23% | 3 | 1 | 66.67% | 9 | 3 | 66.67% |
| utils\config\autofield.rs | 18 | 0 | 100.00% | 4 | 0 | 100.00% | 16 | 0 | 100.00% |
| utils\config\lecture_env.rs | 7 | 0 | 100.00% | 1 | 0 | 100.00% | 3 | 0 | 100.00% |
| utils\constante\parse.rs | 50 | 0 | 100.00% | 7 | 0 | 100.00% | 44 | 0 | 100.00% |
| utils\forms\parse_html.rs | 99 | 99 | 0.00% | 9 | 9 | 0.00% | 52 | 52 | 0.00% |
| utils\forms\sanitizer.rs | 102 | 1 | 99.02% | 7 | 0 | 100.00% | 65 | 0 | 100.00% |
| utils\init_error\init.rs | 8 | 0 | 100.00% | 1 | 0 | 100.00% | 10 | 0 | 100.00% |
| utils\middleware\csp_nonce.rs | 54 | 0 | 100.00% | 6 | 0 | 100.00% | 30 | 0 | 100.00% |
| utils\middleware\csrf.rs | 139 | 0 | 100.00% | 11 | 0 | 100.00% | 72 | 0 | 100.00% |
| utils\password\mod.rs | 330 | 125 | 62.12% | 38 | 12 | 68.42% | 224 | 80 | 64.29% |
| utils\trad\switch_lang.rs | 102 | 10 | 90.20% | 12 | 3 | 75.00% | 58 | 8 | 86.21% |

---

## Fichiers à 0% (non testés)

| Fichier |
|---------|
| app\builder.rs |
| app\error_build.rs |
| app\runique_app.rs |
| app\staging\core_staging.rs |
| app\staging\middleware_staging.rs |
| app\staging\static_staging.rs |
| app\templates.rs |
| bin\runique.rs |
| config\router.rs |
| context\request\extractor.rs |
| context\template.rs |
| context\tera\csp.rs |
| context\tera\static_tera.rs |
| forms\extractor.rs |
| forms\field.rs |
| forms\model_form\mod.rs |
| forms\options\bool_choice.rs |
| forms\prisme\aegis.rs |
| forms\prisme\csrf_gate.rs |
| macros\context\impl_error.rs |
| macros\forms\impl_form.rs |
| middleware\auth\form\login.rs |
| migration\migrate.rs |
| utils\forms\parse_html.rs |

## Fichiers à 100%

| Fichier |
|---------|
| config\app.rs |
| config\security.rs |
| config\server.rs |
| config\settings.rs |
| config\static_files.rs |
| flash\flash_manager.rs |
| flash\flash_struct.rs |
| forms\prisme\rules.rs |
| forms\prisme\sentinel.rs |
| macros\context\flash.rs |
| macros\forms\enum_kind.rs |
| middleware\auth\auth_session.rs (lignes) |
| middleware\auth\user_trait.rs |
| middleware\session\session_parametre.rs |
| migration\foreign_key\mod.rs |
| migration\hooks\mod.rs |
| migration\index\mod.rs |
| migration\primary_key\mod.rs (lignes) |
| migration\relation\mod.rs |
| migration\utils\paths.rs |
| migration\utils\types.rs |
| utils\config\autofield.rs |
| utils\config\lecture_env.rs |
| utils\constante\parse.rs |
| utils\forms\sanitizer.rs (lignes) |
| utils\init_error\init.rs |
| utils\middleware\csp_nonce.rs |
| utils\middleware\csrf.rs |
