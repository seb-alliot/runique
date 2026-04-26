# ORM & Base de Données

Runique utilise **SeaORM** avec un manager Django-like via la macro `impl_objects!`.

## Table des matières

| Module | Description |
| --- | --- |
| [Manager & helpers](/docs/fr/orm/manager) | `impl_objects!`, `all()`, `filter()`, `get()`, `get_or_404()`, `FormEntity` |
| [Requêtes CRUD](/docs/fr/orm/requetes) | SELECT, INSERT, UPDATE, DELETE, COUNT, `search!` |
| [Avancé](/docs/fr/orm/avance) | Transactions, relations, pattern CRUD complet |

> **Intégration formulaires** : via `#[form(model = Entity)]`, un form expose `Form::objects` et supporte `search!(@Form => ...)`. Voir [Formulaires — attribut model](/docs/fr/model/formulaires/formulaires).

## Prochaines étapes

← [**Templates**](/docs/fr/template) | [**Middleware & Sécurité**](/docs/fr/middleware) →
