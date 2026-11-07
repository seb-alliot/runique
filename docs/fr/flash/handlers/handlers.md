# Utilisation dans les handlers

## Pattern avec redirection (messages flash)

```rust
pub async fn sign_up(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");

    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "title" => "Inscription", "signup_form" => &form });
            // ❌ Envoyé mais invalide → message immédiat (pas de redirect).
            // Au premier affichage (GET), rien n'a été envoyé : pas de message.
            if form.is_submitted() {
                context_update!(request => {
                    "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
                });
            }
            return request.render("signup.html");
        }
    };

    // ✅ Message flash → affiché après le redirect. `save` rend le formulaire.
    match validated.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Bienvenue, votre compte est créé !");
            Ok(Redirect::to("/").into_response())
        }
        Err(form) => {
            context_update!(request => {
                "title" => "Erreur de validation",
                "signup_form" => &form,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
            });
            request.render("signup.html")
        }
    }
}
```

---

## Plusieurs types de messages

Pour la page affichée par ce handler, `flash_now!` (sans session) :

```rust
pub async fn about(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "À propos",
        "messages" => flash_now!(info => "Ceci est un message d'information.", "Et un second."),
    });
    request.render("about.html")
}
```

Pour la page suivante, les macros de session, puis une redirection :

```rust
pub async fn publier(request: Request) -> AppResult<Response> {
    success!(request.notices => "Ceci est un message de succès.");
    warning!(request.notices => "Ceci est un message d'avertissement.");
    Ok(Redirect::to("/articles").into_response())
}
```

> `success!` suivi d'un `render` dans la même requête n'affiche rien maintenant : les messages de session sont lus **au début** de la requête, avant le handler. Ils apparaîtraient à la requête suivante, sur une autre page.

---

## Comportement flash (une seule lecture)

Les messages flash stockés en session sont **lus puis retirés** au début de la requête suivante (à l'extraction de `Request`), qu'ils soient affichés ou non :

```
1. POST /signup
   → success!("Bienvenue !")
   → Redirect::to("/")

2. GET /
   → Les messages sont lus depuis la session et supprimés
   → Placés dans le contexte (`messages`), affichés par {% messages %}

3. GET / (reload)
   → Plus de messages (déjà consommés)
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](/docs/fr/flash/macros) | Toutes les macros flash + différences |
| [Templates](/docs/fr/flash/templates) | Affichage dans les templates |

## Retour au sommaire

- [Flash Messages](/docs/fr/flash)
