# Utilisation dans les handlers

## Pattern avec redirection (messages flash)

```rust
pub async fn sign_up(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");

    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            // ❌ Validation échouée → message immédiat (pas de redirect)
            context_update!(request => {
                "title" => "Erreur de validation",
                "signup_form" => &form,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
            });
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

```rust
pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Ceci est un message de succès.");
    info!(request.notices => "Ceci est un message d'information.");
    warning!(request.notices => "Ceci est un message d'avertissement.");
    error!(request.notices => "Ceci est un message d'erreur.");

    context_update!(request => {
        "title" => "À propos",
    });
    request.render("about.html")
}
```

---

## Comportement flash (une seule lecture)

Les messages flash stockés en session sont **consommés automatiquement** lors de l'affichage :

```
1. POST /signup
   → success!("Bienvenue !")
   → Redirect::to("/")

2. GET /
   → Les messages sont lus depuis la session
   → Affichés dans le template
   → Supprimés de la session

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
