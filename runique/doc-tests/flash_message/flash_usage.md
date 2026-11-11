# Messages flash

Les macros flash enregistrent un message dans la session : il s'affiche à la requête suivante,
typiquement après une redirection.

```rust,no_run
use runique::prelude::*;

async fn create_user(request: Request) -> AppResult<Response> {
    // Message unique
    success!(request.notices => "Utilisateur créé avec succès");

    // Plusieurs messages du même type
    success!(request.notices => "Compte activé", "Email de bienvenue envoyé");

    // Types mixtes
    info!(request.notices => "Vérifiez votre boîte email");
    warning!(request.notices => "Votre session expire dans 10 minutes");

    Ok(Redirect::to("/dashboard").into_response())
}

fn main() {}
```

Les messages sont injectés automatiquement dans le contexte des templates.

## `flash_now!` — messages pour la page en cours

```rust
use runique::flash_now;

// Construit directement un Vec<FlashMessage>, sans passer par la session
let messages: Vec<runique::flash::FlashMessage> =
    flash_now!(info => "Bienvenue !", "Nouveau contenu disponible");
assert_eq!(messages.len(), 2);
```
