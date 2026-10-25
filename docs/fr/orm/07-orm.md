# ORM & Base de Données

Runique utilise **SeaORM** avec un manager Django-like via la macro `impl_objects!`.

## Table des matières

| Module | Description |
| --- | --- |
| [Manager & helpers](/docs/fr/orm/manager) | `impl_objects!`, `all()`, `filter()`, `get()`, `get_or_404()`, `FormEntity` |
| [Requêtes CRUD](/docs/fr/orm/requetes) | SELECT, INSERT, UPDATE, DELETE, COUNT, `search!` |
| [Avancé](/docs/fr/orm/avance) | Transactions, relations, pattern CRUD complet |

> **Intégration formulaires** : `#[form(schema = …)]` relie automatiquement le formulaire à l'entité de son module de modèle (`schema = blog` → `blog::Entity`), ce qui donne `Form::objects` et `search!(@Form => ...)`. Voir [Formulaires — lien vers l'entité](/docs/fr/model/formulaires).

## Prochaines étapes

← [**Templates**](/docs/fr/template) | [**Middleware & Sécurité**](/docs/fr/middleware) →
