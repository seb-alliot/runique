# Models et AST (`model!`)

> Fichiers concernés : `src/entities/*`

La macro `model!` génère les entités SeaORM, les schémas de migration et les formulaires associés à partir d'une DSL déclarative.

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [DSL & AST](/docs/fr/model/dsl) | Macros exposées, syntaxe `model!`, AST interne, types et options de champs |
| [Génération & ModelSchema](/docs/fr/model/generation) | Code généré, `ModelSchema`, `to_migration()`, `fill_form()` |
| [Formulaires & enjeux](/docs/fr/model/formulaires) | `#[form(...)]`, enjeux techniques, ordre de lecture |

---

## Ordre recommandé de lecture

1. Ce document (`12-model.md`)
2. [ORM](/docs/fr/orm) pour l'usage DB
3. [Formulaires](/docs/fr/formulaire) pour l'intégration Prisme et rendu
