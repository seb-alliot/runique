# Utilisation dans les handlers

## Pattern avec redirection (messages flash)

```rust
pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;

            // ✅ Message flash → affiché après le redirect
            success!(request.notices => format!(
                "Bienvenue {}, votre compte est créé !",
                user.username
            ));
            return Ok(Redirect::to("/").into_response());
        }

        // ❌ Validation échouée → message immédiat (pas de redirect)
        context_update!(request => {
            "title" => "Erreur de validation",
            "inscription_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render("inscription_form.html");
    }

    // GET → afficher le formulaire
    context_update!(request => {
        "title" => "Inscription",
        "inscription_form" => &form,
    });
    request.render("inscription_form.html")
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
    request.render("about/about.html")
}
```

---

## Comportement flash (une seule lecture)

Les messages flash stockés en session sont **consommés automatiquement** lors de l'affichage :

```
1. POST /inscription
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
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/macros/macros.md) | Toutes les macros flash + différences |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/templates/templates.md) | Affichage dans les templates |

## Retour au sommaire

- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md)
