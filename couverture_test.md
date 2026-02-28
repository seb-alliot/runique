
# Coverage Report - Runique Framework

**Generated:** 2026-02-28 15:29
**Tool:** cargo-llvm-cov (llvm-cov version 21.1.2-rust-1.91.0-stable)

---

## Summary

| Metric | Coverage | Details |
|--------|----------|---------|
| **Function Coverage** | 51.23% | 752/1468 |
| **Line Coverage** | 44.66% | 4692/10505 |
| **Region Coverage** | 41.35% | 6651/16084 |
| **Branch Coverage** | - | 0/0 |

---

## Detailed Coverage by File

### App Module

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| app\builder.rs | 0.00% (0/19) | 0.00% (0/129) | 0.00% (0/205) | - (0/0) |
| app\error_build.rs | 0.00% (0/15) | 0.00% (0/92) | 0.00% (0/127) | - (0/0) |
| app\runique_app.rs | 0.00% (0/4) | 0.00% (0/23) | 0.00% (0/41) | - (0/0) |
| app\staging\admin_staging.rs | 0.00% (0/11) | 0.00% (0/74) | 0.00% (0/79) | - (0/0) |
| app\staging\core_staging.rs | 0.00% (0/9) | 0.00% (0/59) | 0.00% (0/65) | - (0/0) |
| app\staging\middleware_staging.rs | 0.00% (0/26) | 0.00% (0/177) | 0.00% (0/247) | - (0/0) |
| app\staging\static_staging.rs | 0.00% (0/7) | 0.00% (0/24) | 0.00% (0/24) | - (0/0) |
| app\templates.rs | 0.00% (0/9) | 0.00% (0/117) | 0.00% (0/181) | - (0/0) |

### Bin & Config

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| bin\runique.rs | 0.00% (0/9) | 0.00% (0/180) | 0.00% (0/339) | - (0/0) |
| config\app.rs | 100.00% (3/3) | 100.00% (15/15) | 100.00% (19/19) | - (0/0) |
| config\routes.rs | 0.00% (0/4) | 0.00% (0/16) | 0.00% (0/22) | - (0/0) |
| config\security.rs | 100.00% (8/8) | 100.00% (31/31) | 100.00% (44/44) | - (0/0) |
| config\server.rs | 100.00% (4/4) | 100.00% (14/14) | 100.00% (24/24) | - (0/0) |
| config\settings.rs | 100.00% (2/2) | 100.00% (24/24) | 100.00% (39/39) | - (0/0) |
| config\static_files.rs | 100.00% (15/15) | 100.00% (51/51) | 100.00% (80/80) | - (0/0) |

### Context Module

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| context\request_extractor.rs | 0.00% (0/4) | 0.00% (0/44) | 0.00% (0/60) | - (0/0) |
| context\request_extensions.rs | 40.00% (4/10) | 44.87% (35/78) | 35.29% (42/119) | - (0/0) |
| context\template.rs | 0.00% (0/18) | 0.00% (0/135) | 0.00% (0/200) | - (0/0) |
| context\tera\csp.rs | 0.00% (0/1) | 0.00% (0/8) | 0.00% (0/11) | - (0/0) |
| context\tera\form.rs | 73.68% (28/38) | 78.26% (126/161) | 76.58% (206/269) | - (0/0) |
| context\tera\static_tera.rs | 0.00% (0/6) | 0.00% (0/39) | 0.00% (0/70) | - (0/0) |
| context\tera\url.rs | 100.00% (7/7) | 96.88% (31/32) | 96.49% (55/57) | - (0/0) |

### Database & Engine

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| db\config.rs | 20.00% (5/25) | 22.22% (38/171) | 28.28% (69/244) | - (0/0) |
| engine\core.rs | 0.00% (0/2) | 0.00% (0/42) | 0.00% (0/57) | - (0/0) |

### Errors & Flash

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| errors\error.rs | 38.71% (12/31) | 48.06% (136/283) | 35.77% (137/383) | - (0/0) |
| flash\flash_manager.rs | 100.00% (14/14) | 100.00% (49/49) | 100.00% (71/71) | - (0/0) |
| flash\flash_struct.rs | 100.00% (6/6) | 100.00% (37/37) | 100.00% (27/27) | - (0/0) |

### Forms Module

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| forms\base.rs | 35.29% (12/34) | 40.00% (52/130) | 37.36% (65/174) | - (0/0) |
| forms\extractor.rs | 0.00% (0/10) | 0.00% (0/65) | 0.00% (0/89) | - (0/0) |
| forms\field.rs | 0.00% (0/8) | 0.00% (0/22) | 0.00% (0/32) | - (0/0) |
| forms\fields\boolean.rs | 75.00% (9/12) | 72.88% (43/59) | 62.92% (56/89) | - (0/0) |
| forms\fields\choice.rs | 71.11% (32/45) | 74.21% (164/221) | 71.08% (236/332) | - (0/0) |
| forms\fields\datetime.rs | 43.40% (23/53) | 43.17% (177/410) | 38.84% (228/587) | - (0/0) |
| forms\fields\file.rs | 60.00% (21/35) | 44.96% (107/238) | 41.89% (155/370) | - (0/0) |
| forms\fields\hidden.rs | 77.78% (7/9) | 85.42% (41/48) | 80.30% (53/66) | - (0/0) |
| forms\fields\number.rs | 73.68% (14/19) | 57.71% (116/201) | 52.85% (139/263) | - (0/0) |
| forms\fields\special.rs | 77.59% (45/58) | 81.23% (277/341) | 76.48% (374/489) | - (0/0) |
| forms\fields\text.rs | 76.67% (23/30) | 81.01% (145/179) | 77.89% (222/285) | - (0/0) |
| forms\form.rs | 36.21% (21/58) | 27.52% (82/298) | 22.74% (136/598) | - (0/0) |
| forms\generic.rs | 19.23% (5/26) | 19.23% (15/78) | 20.65% (19/92) | - (0/0) |
| forms\model_form\mod.rs | 0.00% (0/3) | 0.00% (0/9) | 0.00% (0/13) | - (0/0) |
| forms\options\bool_choice.rs | 0.00% (0/1) | 0.00% (0/3) | 0.00% (0/3) | - (0/0) |
| forms\prisme\mod.rs | 0.00% (0/6) | 0.00% (0/41) | 0.00% (0/59) | - (0/0) |
| forms\prisme\sondage.rs | 0.00% (0/4) | 0.00% (0/22) | 0.00% (0/32) | - (0/0) |
| forms\prisme\rules.rs | 100.00% (11/11) | 100.00% (69/69) | 100.00% (69/69) | - (0/0) |
| forms\prisme\sentinel.rs | 0.00% (0/1) | 0.00% (0/7) | 0.00% (0/16) | - (0/0) |
| forms\renderer.rs | 33.33% (3/9) | 38.36% (28/73) | 30.09% (34/113) | - (0/0) |
| forms\validator.rs | 66.67% (6/9) | 46.43% (26/56) | 50.54% (47/93) | - (0/0) |

### Macros

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| macros\admin\macros_admin.rs | 0.00% (0/4) | 0.00% (0/11) | 0.00% (0/16) | - (0/0) |
| macros\bdd\objects.rs | 62.50% (15/24) | 62.99% (97/154) | 61.97% (145/234) | - (0/0) |
| macros\bdd\query.rs | 76.00% (19/25) | 69.51% (114/164) | 67.57% (175/259) | - (0/0) |
| macros\context\flash.rs | 100.00% (19/19) | 100.00% (65/65) | 100.00% (96/96) | - (0/0) |
| macros\context\helper.rs | 0.00% (0/6) | 0.00% (0/26) | 0.00% (0/31) | - (0/0) |
| macros\context\impl_error.rs | 0.00% (0/2) | 0.00% (0/2) | 0.00% (0/8) | - (0/0) |
| macros\forms\enum_kind.rs | 100.00% (1/1) | 100.00% (3/3) | 100.00% (3/3) | - (0/0) |
| macros\forms\impl_form.rs | 0.00% (0/3) | 0.00% (0/9) | 0.00% (0/9) | - (0/0) |
| macros\routeur\register_url.rs | 0.00% (0/7) | 0.00% (0/36) | 0.00% (0/63) | - (0/0) |

### Middleware

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| middleware\auth\auth_session.rs | 100.00% (26/26) | 100.00% (135/135) | 93.91% (185/197) | - (0/0) |
| middleware\auth\default_auth.rs | 66.67% (2/3) | 75.00% (6/8) | 75.00% (6/8) | - (0/0) |
| middleware\auth\form\login.rs | 0.00% (0/1) | 0.00% (0/10) | 0.00% (0/14) | - (0/0) |
| middleware\auth\user.rs | 0.00% (0/13) | 0.00% (0/40) | 0.00% (0/46) | - (0/0) |
| middleware\auth\user_trait.rs | 100.00% (2/2) | 100.00% (6/6) | 100.00% (11/11) | - (0/0) |
| middleware\config.rs | 91.67% (11/12) | 98.51% (66/67) | 93.75% (45/48) | - (0/0) |
| middleware\dev\cache.rs | 60.00% (3/5) | 40.62% (13/32) | 40.43% (19/47) | - (0/0) |
| middleware\errors\error.rs | 66.67% (10/15) | 91.84% (225/245) | 68.18% (90/132) | - (0/0) |
| middleware\security\allowed_hosts.rs | 61.90% (13/21) | 73.68% (98/133) | 83.20% (213/256) | - (0/0) |
| middleware\security\csp.rs | 66.67% (12/18) | 62.30% (119/191) | 69.58% (247/355) | - (0/0) |
| middleware\security\csrf.rs | 87.50% (7/8) | 87.34% (69/79) | 85.48% (106/124) | - (0/0) |
| middleware\session\session_parametre.rs | 100.00% (2/2) | 100.00% (10/10) | 100.00% (7/7) | - (0/0) |

### Migration Module

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| migration\column\mod.rs | 79.63% (43/54) | 70.42% (250/355) | 59.27% (323/545) | - (0/0) |
| migration\foreign_key\mod.rs | 100.00% (6/6) | 100.00% (39/39) | 100.00% (42/42) | - (0/0) |
| migration\hooks\mod.rs | 100.00% (7/7) | 100.00% (30/30) | 100.00% (45/45) | - (0/0) |
| migration\index\mod.rs | 100.00% (6/6) | 100.00% (29/29) | 100.00% (45/45) | - (0/0) |
| migration\makemigrations.rs | 3.85% (1/26) | 4.52% (9/199) | 4.57% (18/394) | - (0/0) |
| migration\migrate.rs | 0.00% (0/38) | 0.00% (0/374) | 0.00% (0/654) | - (0/0) |
| migration\primary_key\mod.rs | 100.00% (7/7) | 100.00% (38/38) | 97.62% (41/42) | - (0/0) |
| migration\relation\mod.rs | 100.00% (4/4) | 100.00% (31/31) | 100.00% (18/18) | - (0/0) |
| migration\schema\mod.rs | 87.50% (21/24) | 66.24% (157/237) | 69.44% (250/360) | - (0/0) |
| migration\utils\convertisseur.rs | 100.00% (3/3) | 92.86% (13/14) | 95.24% (20/21) | - (0/0) |
| migration\utils\diff.rs | 60.00% (12/20) | 84.62% (99/117) | 83.89% (125/149) | - (0/0) |
| migration\utils\generators.rs | 0.00% (0/25) | 0.00% (0/387) | 0.00% (0/440) | - (0/0) |
| migration\utils\helpers.rs | 100.00% (19/19) | 62.68% (215/343) | 63.80% (416/652) | - (0/0) |
| migration\utils\parser_builder.rs | 100.00% (11/11) | 86.67% (143/165) | 71.61% (222/310) | - (0/0) |
| migration\utils\parser_seaorm.rs | 13.33% (2/15) | 2.04% (4/196) | 2.68% (8/299) | - (0/0) |
| migration\utils\paths.rs | 100.00% (47/47) | 100.00% (192/192) | 100.00% (356/356) | - (0/0) |
| migration\utils\types.rs | 100.00% (1/1) | 100.00% (10/10) | 100.00% (17/17) | - (0/0) |

### Utils Module

| Filename | Function | Line | Region | Branch |
|----------|----------|------|--------|--------|
| utils\aliases\helpers.rs | 33.33% (1/3) | 33.33% (3/9) | 38.46% (5/13) | - (0/0) |
| utils\config\autofield.rs | 100.00% (4/4) | 100.00% (16/16) | 100.00% (18/18) | - (0/0) |
| utils\config\lecture_env.rs | 100.00% (1/1) | 100.00% (3/3) | 100.00% (7/7) | - (0/0) |
| utils\constante\parse.rs | 42.86% (3/7) | 86.36% (38/44) | 72.00% (36/50) | - (0/0) |
| utils\forms\parse_html.rs | 0.00% (0/9) | 0.00% (0/52) | 0.00% (0/99) | - (0/0) |
| utils\forms\sanitizer.rs | 100.00% (7/7) | 100.00% (65/65) | 99.02% (101/102) | - (0/0) |
| utils\init_error\init.rs | 0.00% (0/1) | 0.00% (0/10) | 0.00% (0/8) | - (0/0) |
| utils\middleware\csp_nonce.rs | 100.00% (6/6) | 100.00% (30/30) | 100.00% (54/54) | - (0/0) |
| utils\middleware\csrf.rs | 100.00% (11/11) | 100.00% (72/72) | 100.00% (139/139) | - (0/0) |
| utils\password\mod.rs | 68.42% (26/38) | 64.29% (144/224) | 62.12% (205/330) | - (0/0) |
| utils\trad\switch_lang.rs | 75.00% (9/12) | 86.21% (50/58) | 90.20% (92/102) | - (0/0) |

---

## Files with 100% Coverage ✅

- admin\registry.rs
- config\app.rs
- config\security.rs
- config\server.rs
- config\settings.rs
- config\static_files.rs
- context\tera\url.rs
- flash\flash_manager.rs
- flash\flash_struct.rs
- forms\prisme\rules.rs
- macros\context\flash.rs
- macros\forms\enum_kind.rs
- middleware\auth\auth_session.rs
- middleware\auth\user_trait.rs
- middleware\session\session_parametre.rs
- migration\foreign_key\mod.rs
- migration\hooks\mod.rs
- migration\index\mod.rs
- migration\primary_key\mod.rs
- migration\relation\mod.rs
- migration\utils\convertisseur.rs
- migration\utils\helpers.rs
- migration\utils\parser_builder.rs
- migration\utils\paths.rs
- migration\utils\types.rs
- utils\config\autofield.rs
- utils\config\lecture_env.rs
- utils\forms\sanitizer.rs
- utils\middleware\csp_nonce.rs
- utils\middleware\csrf.rs

---

## Files with 0% Coverage ❌

- admin\cli_admin.rs
- admin\config\config_admin.rs
- admin\daemon\generator.rs
- admin\daemon\parser.rs
- admin\daemon\watcher.rs
- admin\middleware\admin_middleware.rs
- admin\router\admin_router.rs
- app\builder.rs
- app\error_build.rs
- app\runique_app.rs
- app\staging\admin_staging.rs
- app\staging\core_staging.rs
- app\staging\middleware_staging.rs
- app\staging\static_staging.rs
- app\templates.rs
- bin\runique.rs
- config\routes.rs
- context\request_extractor.rs
- context\template.rs
- context\tera\csp.rs
- context\tera\static_tera.rs
- db\config.rs
- engine\core.rs
- forms\extractor.rs
- forms\field.rs
- forms\model_form\mod.rs
- forms\options\bool_choice.rs
- forms\prisme\mod.rs
- forms\prisme\sondage.rs
- forms\prisme\sentinel.rs
- macros\admin\macros_admin.rs
- macros\context\helper.rs
- macros\context\impl_error.rs
- macros\forms\impl_form.rs
- macros\routeur\register_url.rs
- middleware\auth\form\login.rs
- middleware\auth\user.rs
- migration\migrate.rs
- migration\utils\generators.rs
- migration\utils\parser_seaorm.rs
- utils\forms\parse_html.rs
- utils\init_error\init.rs

---

## Critical Low Coverage Areas (< 10%)

| Module | Files | Risk Level |
|--------|-------|------------|
| **Admin/CLI** | 9 files | 🔴 High |
| **App Core** | 8 files | 🔴 High |
| **Migration Tools** | migrate.rs, makemigrations.rs | 🔴 High |
| **Forms Prisme** | 4 files | 🟡 Medium |
| **Engine** | core.rs | 🔴 High |

---
