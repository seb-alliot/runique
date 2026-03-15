## Rapport de Couverture de Code

> Mis à jour le 2026-03-15
> `cargo llvm-cov --tests --package runique --ignore-filename-regex "admin|bin/runique|runique_app" --summary-only`
> **~1 600 tests** — 0 échec

### Résumé Global

| Métrique | Total | Manqué | Couverture | Précédent |
|----------|-------|--------|------------|-----------|
| **Regions** | 16,833 | 4,145 | **75.38%** | 67.22% |
| **Functions** | 1,596 | 274 | **82.83%** | 76.66% |
| **Lines** | 10,864 | 2,352 | **78.35%** | 71.04% |
| **Branches** | 0 | 0 | - | - |

---

### Détail par Fichier

#### App

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `app\builder.rs` | 86.47% (179/207) | 89.47% (17/19) | 89.15% (115/129) |
| `app\error_build.rs` | 85.04% (108/127) | 100.00% (15/15) | 94.57% (87/92) |
| `app\runique_app.rs` | 8.89% (4/45) ⚠️ | 20.00% (1/5) ⚠️ | 11.54% (3/26) ⚠️ |
| `app\staging\core_staging.rs` | 67.69% (44/65) | 77.78% (7/9) | 76.27% (45/59) |
| `app\staging\csp_config.rs` | 100.00% (119/119) ✅ | 100.00% (18/18) ✅ | 100.00% (70/70) ✅ |
| `app\staging\middleware_staging.rs` | 73.79% (259/351) | 69.23% (27/39) | 76.72% (201/262) |
| `app\staging\static_staging.rs` | 100.00% (27/27) ✅ | 100.00% (8/8) ✅ | 100.00% (27/27) ✅ |
| `app\templates.rs` | 75.82% (116/153) | 44.44% (4/9) ⚠️ | 74.07% (60/81) |

#### Bin

**Cli** Difficile a tester

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `bin\runique.rs` | 0.00% (0/429) ❌ | 0.00% (0/13) ❌ | 0.00% (0/208) ❌ |

#### Config

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `config\app.rs` | 100.00% (19/19) ✅ | 100.00% (3/3) ✅ | 100.00% (15/15) ✅ |
| `config\router.rs` | 100.00% (22/22) ✅ | 100.00% (4/4) ✅ | 100.00% (16/16) ✅ |
| `config\security.rs` | 100.00% (44/44) ✅ | 100.00% (8/8) ✅ | 100.00% (31/31) ✅ |
| `config\server.rs` | 100.00% (24/24) ✅ | 100.00% (4/4) ✅ | 100.00% (14/14) ✅ |
| `config\settings.rs` | 100.00% (44/44) ✅ | 100.00% (2/2) ✅ | 100.00% (23/23) ✅ |
| `config\static_files.rs` | 100.00% (80/80) ✅ | 100.00% (15/15) ✅ | 100.00% (51/51) ✅ |

#### Context

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `context\request\extractor.rs` | 93.33% (28/30) | 100.00% (2/2) ✅ | 100.00% (28/28) ✅ |
| `context\request_extensions.rs` | 83.19% (99/119) | 100.00% (10/10) ✅ | 93.59% (73/78) |
| `context\template.rs` | 73.15% (158/216) | 80.95% (17/21) | 76.67% (115/150) |
| `context\tera\csp.rs` | 100.00% (11/11) ✅ | 100.00% (1/1) ✅ | 100.00% (8/8) ✅ |
| `context\tera\form.rs` | 76.58% (206/269) | 73.68% (28/38) | 78.26% (126/161) |
| `context\tera\static_tera.rs` | 89.66% (78/87) | 71.43% (5/7) | 89.13% (41/46) |
| `context\tera\url.rs` | 93.22% (55/59) | 87.50% (7/8) | 93.94% (31/33) |

#### DB

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `db\config.rs` | 78.04% (231/296) | 100.00% (24/24) ✅ | 80.68% (167/207) |

#### Engine

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `engine\core.rs` | 28.12% (18/64) ⚠️ | 50.00% (1/2) | 29.17% (14/48) ⚠️ |

#### Errors

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `errors\error.rs` | 75.52% (324/429) | 77.42% (24/31) | 88.54% (255/288) |

#### Flash

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `flash\flash_manager.rs` | 100.00% (71/71) ✅ | 100.00% (14/14) ✅ | 100.00% (49/49) ✅ |
| `flash\flash_struct.rs` | 100.00% (27/27) ✅ | 100.00% (6/6) ✅ | 100.00% (37/37) ✅ |

#### Forms

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `forms\base.rs` | 96.55% (168/174) | 94.12% (32/34) | 95.38% (124/130) |
| `forms\extractor.rs` | 90.00% (90/100) | 81.82% (9/11) | 94.29% (66/70) |
| `forms\field.rs` | 52.94% (27/51) ⚠️ | 58.33% (7/12) ⚠️ | 55.88% (19/34) ⚠️ |
| `forms\fields\boolean.rs` | 59.57% (56/94) ⚠️ | 75.00% (9/12) | 66.15% (43/65) |
| `forms\fields\choice.rs` | 75.00% (261/348) | 80.00% (36/45) | 76.57% (183/239) |
| `forms\fields\datetime.rs` | 79.13% (493/623) | 84.91% (45/53) | 82.03% (356/434) |
| `forms\fields\file.rs` | 40.00% (162/405) ⚠️ | 61.11% (22/36) | 44.92% (115/256) ⚠️ |
| `forms\fields\hidden.rs` | 75.68% (56/74) | 77.78% (7/9) | 75.93% (41/54) |
| `forms\fields\number.rs` | 85.93% (232/270) | 89.47% (17/19) | 88.78% (182/205) |
| `forms\fields\special.rs` | 72.39% (375/518) | 77.59% (45/58) | 75.14% (275/366) |
| `forms\fields\text.rs` | 92.54% (248/268) | 92.59% (25/27) | 95.95% (166/173) |
| `forms\form.rs` | 62.99% (405/643) | 91.67% (55/60) | 73.42% (232/316) |
| `forms\generic.rs` | 72.83% (67/92) | 73.08% (19/26) | 73.08% (57/78) |
| `forms\model_form\mod.rs` | 46.15% (6/13) ⚠️ | 66.67% (2/3) | 66.67% (6/9) |
| `forms\options\bool_choice.rs` | 100.00% (3/3) ✅ | 100.00% (1/1) ✅ | 100.00% (3/3) ✅ |
| `forms\prisme\aegis.rs` | 70.24% (59/84) | 71.43% (5/7) | 74.58% (44/59) |
| `forms\prisme\csrf_gate.rs` | 96.08% (49/51) | 100.00% (5/5) ✅ | 96.97% (32/33) |
| `forms\prisme\rules.rs` | 100.00% (71/71) ✅ | 100.00% (11/11) ✅ | 100.00% (77/77) ✅ |
| `forms\prisme\sentinel.rs` | 100.00% (16/16) ✅ | 100.00% (1/1) ✅ | 100.00% (7/7) ✅ |
| `forms\renderer.rs` | 88.79% (103/116) | 88.89% (8/9) | 97.26% (71/73) |
| `forms\test_forms.rs` | 99.10% (989/998) | 96.39% (80/83) | 97.99% (439/448) |
| `forms\validator.rs` | 98.96% (95/96) | 100.00% (9/9) ✅ | 98.15% (53/54) |

#### Macros

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `macros\bdd\objects.rs` | 61.97% (145/234) | 62.50% (15/24) | 62.99% (97/154) |
| `macros\bdd\query.rs` | 67.57% (175/259) | 76.00% (19/25) | 69.51% (114/164) |
| `macros\context\flash.rs` | 100.00% (96/96) ✅ | 100.00% (19/19) ✅ | 100.00% (65/65) ✅ |
| `macros\context\helper.rs` | 90.32% (28/31) | 83.33% (5/6) | 88.46% (23/26) |
| `macros\context\impl_error.rs` | 100.00% (8/8) ✅ | 100.00% (2/2) ✅ | 100.00% (2/2) ✅ |
| `macros\forms\enum_kind.rs` | 100.00% (3/3) ✅ | 100.00% (1/1) ✅ | 100.00% (3/3) ✅ |
| `macros\forms\impl_form.rs` | 100.00% (9/9) ✅ | 100.00% (3/3) ✅ | 100.00% (9/9) ✅ |
| `macros\routeur\register_url.rs` | 79.45% (58/73) | 58.33% (7/12) ⚠️ | 86.00% (43/50) |

#### Middleware

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `middleware\auth\auth_session.rs` | 75.88% (195/257) | 81.25% (26/32) | 78.65% (140/178) |
| `middleware\auth\default_auth.rs` | 75.00% (6/8) | 66.67% (2/3) | 75.00% (6/8) |
| `middleware\auth\form\login.rs` | 100.00% (14/14) ✅ | 100.00% (1/1) ✅ | 100.00% (10/10) ✅ |
| `middleware\auth\login_guard.rs` | 67.65% (92/136) | 75.00% (9/12) | 75.31% (61/81) |
| `middleware\auth\user.rs` | 91.30% (42/46) | 84.62% (11/13) | 90.00% (36/40) |
| `middleware\auth\user_trait.rs` | 100.00% (11/11) ✅ | 100.00% (2/2) ✅ | 100.00% (6/6) ✅ |
| `middleware\config.rs` | 93.62% (44/47) | 91.67% (11/12) | 98.61% (71/72) |
| `middleware\dev\cache.rs` | 40.43% (19/47) ⚠️ | 60.00% (3/5) | 40.62% (13/32) ⚠️ |
| `middleware\errors\error.rs` | 61.69% (182/295) | 60.00% (12/20) | 74.05% (274/370) |
| `middleware\rate_limit.rs` | 80.99% (98/121) | 88.89% (16/18) | 87.50% (70/80) |
| `middleware\security\allowed_hosts.rs` | 88.98% (226/254) | 80.95% (17/21) | 84.50% (109/129) |
| `middleware\security\csp.rs` | 95.64% (395/413) | 95.00% (19/20) | 98.68% (224/227) |
| `middleware\security\csrf.rs` | 73.02% (138/189) | 75.00% (9/12) | 77.69% (94/121) |
| `middleware\session\cleaning_store.rs` | 86.96% (100/115) | 95.65% (22/23) | 91.86% (79/86) |
| `middleware\session\session_parametre.rs` | 100.00% (7/7) ✅ | 100.00% (2/2) ✅ | 100.00% (10/10) ✅ |

#### Migration

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `migration\column\mod.rs` | 88.40% (480/543) | 100.00% (54/54) ✅ | 94.65% (336/355) |
| `migration\foreign_key\mod.rs` | 100.00% (42/42) ✅ | 100.00% (6/6) ✅ | 100.00% (39/39) ✅ |
| `migration\hooks\mod.rs` | 100.00% (45/45) ✅ | 100.00% (7/7) ✅ | 100.00% (30/30) ✅ |
| `migration\index\mod.rs` | 100.00% (45/45) ✅ | 100.00% (6/6) ✅ | 100.00% (29/29) ✅ |
| `migration\makemigrations.rs` | 82.83% (357/431) | 57.69% (15/26) ⚠️ | 84.31% (172/204) |
| `migration\migrate.rs` | 22.80% (163/715) ❌ | 41.03% (16/39) ⚠️ | 24.23% (95/392) ❌ |
| `migration\primary_key\mod.rs` | 97.62% (41/42) | 100.00% (7/7) ✅ | 100.00% (38/38) ✅ |
| `migration\relation\mod.rs` | 100.00% (18/18) ✅ | 100.00% (4/4) ✅ | 100.00% (31/31) ✅ |
| `migration\schema\mod.rs` | 66.49% (250/376) | 75.00% (21/28) | 63.56% (157/247) |
| `migration\utils\convertisseur.rs` | 95.24% (20/21) | 100.00% (3/3) ✅ | 92.86% (13/14) |
| `migration\utils\diff.rs` | 100.00% (149/149) ✅ | 100.00% (20/20) ✅ | 100.00% (117/117) ✅ |
| `migration\utils\generators.rs` | 67.50% (297/440) | 88.00% (22/25) | 66.33% (262/395) |
| `migration\utils\helpers.rs` | 71.32% (465/652) | 100.00% (19/19) ✅ | 70.26% (241/343) |
| `migration\utils\parser_builder.rs` | 89.91% (285/317) | 100.00% (10/10) ✅ | 95.78% (159/166) |
| `migration\utils\parser_seaorm.rs` | 66.23% (200/302) | 53.33% (8/15) ⚠️ | 69.65% (140/201) |
| `migration\utils\paths.rs` | 100.00% (356/356) ✅ | 100.00% (47/47) ✅ | 100.00% (192/192) ✅ |
| `migration\utils\types.rs` | 100.00% (17/17) ✅ | 100.00% (1/1) ✅ | 100.00% (10/10) ✅ |

#### Utils

| Fichier | Regions Cover | Functions Cover | Lines Cover |
|---------|---------------|-----------------|-------------|
| `utils\aliases\helpers.rs` | 100.00% (13/13) ✅ | 100.00% (3/3) ✅ | 100.00% (9/9) ✅ |
| `utils\config\autofield.rs` | 100.00% (18/18) ✅ | 100.00% (4/4) ✅ | 100.00% (16/16) ✅ |
| `utils\config\lecture_env.rs` | 100.00% (7/7) ✅ | 100.00% (1/1) ✅ | 100.00% (3/3) ✅ |
| `utils\constante\parse.rs` | 100.00% (50/50) ✅ | 100.00% (7/7) ✅ | 100.00% (44/44) ✅ |
| `utils\constante\regex_template.rs` | 100.00% (16/16) ✅ | 100.00% (4/4) ✅ | 100.00% (9/9) ✅ |
| `utils\forms\parse_html.rs` | 79.21% (80/101) | 55.56% (5/9) ⚠️ | 52.17% (36/69) ⚠️ |
| `utils\forms\sanitizer.rs` | 99.02% (101/102) | 100.00% (8/8) ✅ | 100.00% (68/68) ✅ |
| `utils\init_error\init.rs` | 80.00% (20/25) | 66.67% (2/3) | 89.47% (17/19) |
| `utils\middleware\csp_nonce.rs` | 100.00% (54/54) ✅ | 100.00% (6/6) ✅ | 100.00% (30/30) ✅ |
| `utils\middleware\csrf.rs` | 100.00% (140/140) ✅ | 100.00% (11/11) ✅ | 100.00% (72/72) ✅ |
| `utils\password\mod.rs` | 65.49% (222/339) | 73.68% (28/38) | 68.00% (153/225) |
| `utils\trad\switch_lang.rs` | 90.91% (220/242) | 70.97% (22/31) | 92.21% (142/154) |

---

### Points d'Attention (Couverture &lt; 50%)

| Fichier | Regions | Functions | Lines | Priorité |
|---------|---------|-----------|-------|----------|
| `bin\runique.rs` | 0.00% | 0.00% | 0.00% | 🔴 Critique |
| `migration\migrate.rs` | 22.80% | 41.03% | 24.23% | 🔴 Critique |
| `engine\core.rs` | 28.12% | 50.00% | 29.17% | 🟠 Haute |
| `forms\fields\file.rs` | 40.00% | 61.11% | 44.92% | 🟠 Haute |
| `middleware\dev\cache.rs` | 40.43% | 60.00% | 40.62% | 🟠 Haute |
| `forms\model_form\mod.rs` | 46.15% | 66.67% | 66.67% | 🟡 Moyenne |
| `forms\field.rs` | 52.94% | 58.33% | 55.88% | 🟡 Moyenne |

---

### Fichiers 100% Couverts ✅

- `app\staging\csp_config.rs`
- `app\staging\static_staging.rs`
- `config\app.rs`
- `config\router.rs`
- `config\security.rs`
- `config\server.rs`
- `config\settings.rs`
- `config\static_files.rs`
- `context\tera\csp.rs`
- `flash\flash_manager.rs`
- `flash\flash_struct.rs`
- `forms\options\bool_choice.rs`
- `forms\prisme\rules.rs`
- `forms\prisme\sentinel.rs`
- `forms\validator.rs`
- `macros\context\flash.rs`
- `macros\context\impl_error.rs`
- `macros\forms\enum_kind.rs`
- `macros\forms\impl_form.rs`
- `middleware\auth\form\login.rs`
- `middleware\auth\user_trait.rs`
- `middleware\session\session_parametre.rs`
- `migration\foreign_key\mod.rs`
- `migration\hooks\mod.rs`
- `migration\index\mod.rs`
- `migration\primary_key\mod.rs`
- `migration\relation\mod.rs`
- `migration\utils\diff.rs`
- `migration\utils\paths.rs`
- `migration\utils\types.rs`
- `utils\aliases\helpers.rs`
- `utils\config\autofield.rs`
- `utils\config\lecture_env.rs`
- `utils\constante\parse.rs`
- `utils\constante\regex_template.rs`
- `utils\forms\sanitizer.rs`
- `utils\middleware\csp_nonce.rs`
- `utils\middleware\csrf.rs`

---

---

### Progressions notables (session 2026-03-13 → 2026-03-15)

| Fichier | Avant | Après (functions) |
| ------- | ----- | ----------------- |
| `context\template.rs` | 0% | **80.95%** |
| `context\request\extractor.rs` | 0% | **100.00%** |
| `context\request_extensions.rs` | 40% | **100.00%** |
| `errors\error.rs` | 38.71% | **77.42%** |
| `middleware\security\csp.rs` | 66.67% | **95.00%** |
| `middleware\session\cleaning_store.rs` | — | **95.65%** (nouveau) |
| `app\staging\csp_config.rs` | — | **100.00%** (nouveau) |
| `forms\extractor.rs` | 0% | **81.82%** |
| `forms\fields\datetime.rs` | 43.40% | **84.91%** |
| `forms\fields\number.rs` | 73.68% | **89.47%** |
| `macros\context\impl_error.rs` | 0% | **100.00%** |
| `migration\column\mod.rs` | 79.63% | **100.00%** |

---

### Ce qui reste à couvrir

#### Priorité haute — effort moyen

| Fichier | Functions | Bloquant |
| ------- | --------- | -------- |
| `migration\migrate.rs` | 41.03% | Commandes CLI sea-orm |
| `engine\core.rs` | 50.00% | Initialisation app complète |
| `middleware\dev\cache.rs` | 60.00% | Cache HTTP conditionnel |
| `forms\fields\file.rs` | 61.11% | Upload multipart |
| `middleware\errors\error.rs` | 60.00% | Rendu erreurs HTTP |

#### Priorité moyenne — testable unitairement

| Fichier | Functions | Cible |
| ------- | --------- | ----- |
| `utils\password\mod.rs` | 73.68% | 90% |
| `migration\schema\mod.rs` | 75.00% | 90% |
| `migration\utils\generators.rs` | 88.00% | 95% |
| `forms\form.rs` | 91.67% | 95% |
| `utils\trad\switch_lang.rs` | 70.97% | 90% |
| `forms\fields\special.rs` | 77.59% | 90% |

Mis à jour le 15 mars 2026
