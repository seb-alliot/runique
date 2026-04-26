# Messages flash

Les macros flash envoient des messages qui sont affichés à l'utilisateur lors de la prochaine requête.

```rust,ignore
use runique::prelude::*;
use runique::{success, error, info, warning};

// Handler avec messages flash (nécessite `Message` en paramètre axum)
async fn create_user_handler(
    mut message: Message,
    ctx: Request,
    Form(data): Form<HashMap<String, String>>,
) -> Response {
    // Message unique
    success!(message => "Utilisateur créé avec succès");

    // Plusieurs messages du même type
    success!(message => "Compte activé", "Email de bienvenue envoyé");

    // Types mixtes
    info!(message => "Vérifiez votre boîte email");
    warning!(message => "Votre session expire dans 10 minutes");

    Redirect::to("/dashboard").into_response()
}

// Handler qui affiche les messages flash reçus
async fn dashboard_handler(ctx: Request) -> Response {
    let mut context = ctx.context.clone();
    // Les messages flash sont automatiquement injectés dans le contexte Tera
    // sous la clé "flash_messages"
    ctx.render("dashboard.html", &context)
}
```

## `flash_now!` — Messages sans session

```rust,ignore
use runique::prelude::*;
use runique::flash_now;

// Retourne directement un Vec<FlashMessage> sans passer par la session
fn get_info_messages() -> Vec<runique::flash::FlashMessage> {
    flash_now!(info => "Bienvenue !", "Nouveau contenu disponible")
}
```
