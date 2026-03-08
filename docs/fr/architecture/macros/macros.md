# Macros Rust

Runique fournit un ensemble de macros pour simplifier le développement.

---

## Macros de contexte

| Macro | Description | Exemple |
| ----- | ----------- | ------- |
| `context!` | Créer un contexte Tera | `context!("title" => "Page")` |
| `context_update!` | Ajouter au contexte d'une Request | `context_update!(request => { "key" => value })` |

---

## Macros flash messages

| Macro | Description | Exemple |
| ----- | ----------- | ------- |
| `success!` | Message de succès (session) | `success!(request.notices => "OK !")` |
| `error!` | Message d'erreur (session) | `error!(request.notices => "Erreur")` |
| `info!` | Message info (session) | `info!(request.notices => "Info")` |
| `warning!` | Avertissement (session) | `warning!(request.notices => "Attention")` |
| `flash_now!` | Message immédiat (sans session) | `flash_now!(error => "Erreurs")` |

---

## Macros de routage

| Macro | Description | Exemple |
| ----- | ----------- | ------- |
| `urlpatterns!` | Définir des routes avec noms | `urlpatterns!("/" => view!{...}, name = "index")` |
| `view!` | Handler pour toutes méthodes HTTP | `view!{ GET => handler, POST => handler2 }` |
| `impl_objects!` | Manager Django-like pour SeaORM | `impl_objects!(Entity)` |

---

## Macros d'erreur

| Macro | Description |
| ----- | ----------- |
| `impl_from_error!` | Génère `From<Error>` pour `AppError` |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Concepts clés](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/concepts/concepts.md) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Tags & filtres Tera](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/tera/tera.md) | Tags Django-like, filtres, fonctions |
| [Stack middleware](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/middleware/middleware.md) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/lifecycle/lifecycle.md) | Cycle de vie, bonnes pratiques |

## Retour au sommaire

- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md)
