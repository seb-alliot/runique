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
| `view!` | Handler pour toutes méthodes HTTP | `view!{ handler }` |
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
| [Concepts clés](/docs/fr/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](/docs/fr/architecture/lifecycle) | Cycle de vie, bonnes pratiques |

## Retour au sommaire

- [Architecture](/docs/fr/architecture)
