# Models et AST (`model!`)

> Fichiers concernés : `src/entities/*`

La macro `model!` génère les entités SeaORM, les schémas de migration et les formulaires associés à partir d'une DSL déclarative.

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [DSL & AST](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/dsl/dsl.md) | Macros exposées, syntaxe `model!`, AST interne, types et options de champs |
| [Génération & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/generation/generation.md) | Code généré, `ModelSchema`, `to_migration()`, `fill_form()` |
| [Formulaires & enjeux](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/formulaires/formulaires.md) | `#[form(...)]`, enjeux techniques, ordre de lecture |

---

## Ordre recommandé de lecture

1. Ce document (`12-model.md`)
2. [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md) pour l'usage DB
3. [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md) pour l'intégration Prisme et rendu
