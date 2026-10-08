# Exemple complet & AdminPanel

## Exemple complet — Login / Logout

```rust
use runique::prelude::*;

// LoginForm — déclaré séparément, .no_hash() obligatoire sur le champ password
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl RuniqueForm for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Nom d'utilisateur").required());
        form.field(&TextField::password("password").label("Mot de passe").no_hash().required());
    }
    impl_form_access!();
}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form();
    let validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "login_form" => &form });
            return request.render("login.html");
        }
    };

    let db = request.engine.db.clone();
    let username = validated.cleaned_string("username").unwrap_or_default();
    let password = validated.cleaned_string("password").unwrap_or_default();

    // 1. Vérifier les identifiants : mot de passe vérifié même pour un nom inconnu
    //    (pas de fuite par le temps de réponse), compte actif et activé.
    // 2. Ouvrir la session — identifiant de session et jeton CSRF renouvelés.
    if let Some(user) = authenticate_user(&db, username.trim(), &password).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        return Ok(Redirect::to("/dashboard").into_response());
    }

    // Identifiants invalides (message générique — ne pas distinguer user inconnu / mdp faux)
    let form = validated.into_form();
    context_update!(request => {
        "login_form" => &form,
        "messages"   => flash_now!(error => "Identifiants invalides"),
    });
    request.render("login.html")
}

pub async fn logout_view(mut request: Request) -> AppResult<Response> {
    logout(&request.session, None).await.ok();
    Ok(Redirect::to("/login").into_response())
}
```

---

## Authentification pour l'AdminPanel

Rien à configurer : l'admin connecte les comptes de `eihwaz_users`, actifs et staff ou
superuser. Pour ajouter des champs au modèle utilisateur, utiliser `extend!{ table: "eihwaz_users", ... }`.

Pour brancher l'authentification au panneau d'administration, voir aussi [11-Admin.md](/docs/fr/admin).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](/docs/fr/auth/modele) | Built-in, trait `RuniqueUser` |
| [Helpers de session](/docs/fr/auth/session) | `login`, `logout` |
| [Middlewares & CurrentUser](/docs/fr/auth/middleware) | Protection des routes |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
