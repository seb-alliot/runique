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

## En situation

Les macros se combinent dans un handler typique :

```rust
use runique::prelude::*;

pub async fn contact(mut request: Request) -> AppResult<Response> {
    let mut form: ContactForm = request.form();

    if request.is_post() && form.is_valid().await {
        // Flash en session + redirection (pattern Post/Redirect/Get)
        success!(request.notices => "Message envoyé !");
        return Ok(Redirect::to("/contact").into_response());
    }

    // Ajoute des variables au contexte de la requête
    context_update!(request => {
        "title" => "Contact",
        "contact_form" => &form,
    });
    request.render("contact.html")
}
```

> `success!` / `error!` / `info!` / `warning!` écrivent en session (visibles après redirection). `flash_now!` produit un message immédiat sans session — utile quand on réaffiche la même page sans rediriger.

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
