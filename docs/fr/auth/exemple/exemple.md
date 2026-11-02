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

impl FormAccess for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Nom d'utilisateur").required());
        form.field(&TextField::password("password").label("Mot de passe").no_hash().required());
    }
    impl_form_access!();
}

impl RuniqueForm for LoginForm {}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form();
    let validated = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "login_form" => &form });
            return request.render("login.html");
        }
    };

    let db = request.engine.db.clone();
    let username = validated.cleaned::<String>("username").unwrap_or_default();
    let password = validated.cleaned::<String>("password").unwrap_or_default();

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

// Route en POST seulement : "/logout" => post(views::logout_view), name = "logout"
pub async fn logout_view(request: Request) -> AppResult<Response> {
    // Cette vue ne lit pas de formulaire : elle vérifie le jeton CSRF elle-même.
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    // Une erreur ici veut dire que la session n'a pas pu être fermée :
    // elle est toujours active, l'utilisateur reçoit une page d'erreur.
    logout(&request.session, None).await?;
    Ok(Redirect::to("/login").into_response())
}
```

Le formulaire de déconnexion, dans le template :

```html
<form method="post" action='{% link "logout" %}'>
    {% csrf %}
    <button type="submit">Se déconnecter</button>
</form>
```

Une déconnexion en lien (GET) pourrait être déclenchée par un autre site (une image, un lien) : elle doit passer par un POST avec son jeton CSRF.

---

## Authentification pour l'AdminPanel

Rien à configurer : l'admin connecte les comptes de `eihwaz_users` actifs et activés, de rang
admin, dirigeant ou superuser, ou membres d'un groupe qui ouvre l'admin. Pour ajouter des champs au modèle utilisateur, utiliser `extend!{ table: "eihwaz_users", ... }`.

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
