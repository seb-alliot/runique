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
    let mut form: LoginForm = request.form();
    if request.is_post() && form.is_valid().await {
        let db = request.engine.db.clone();
        let username = form.cleaned_string("username").unwrap_or_default();
        let password = form.cleaned_string("password").unwrap_or_default();

        // 1. Chercher l'utilisateur par username via search!
        let query = search!(users::Entity => Username eq username.trim());
        let user = query.first(&db).await.unwrap_or(None);

        if let Some(user) = user
            && user.is_active
            && verify(&password, &user.password)
        {
            // 2. Ouvrir la session — cycle_id() anti-fixation de session inclus
            auth_login(&request.session, &db, user.id).await.ok();
            return Ok(Redirect::to("/dashboard").into_response());
        }

        // Identifiants invalides (message générique — ne pas distinguer user inconnu / mdp faux)
        context_update!(request => {
            "login_form" => &form,
            "messages"   => flash_now!(error => "Identifiants invalides"),
        });
    } else {
        context_update!(request => { "login_form" => &form });
    }

    request.render("login.html")
}

pub async fn logout_view(mut request: Request) -> AppResult<Response> {
    logout(&request.session, None).await.ok();
    Ok(Redirect::to("/login").into_response())
}
```

---

## Authentification pour l'AdminPanel

### Avec le User built-in (zéro config)

```rust
.with_admin(|a| a.auth(RuniqueAdminAuth::new()))
```

### Avec un modèle custom

```rust
use runique::prelude::*;

// 1. Implémenter UserEntity sur votre entité
impl UserEntity for users::Entity {
    type Model = users::Model;

    async fn find_by_id(db: &DatabaseConnection, id: Pk) -> Option<Self::Model> {
        users::Entity::find_by_id(id).one(db).await.ok().flatten()
    }

    async fn find_by_username(db: &DatabaseConnection, username: &str) -> Option<Self::Model> {
        users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model> {
        users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    async fn update_password(db: &DatabaseConnection, email: &str, new_hash: &str) -> Result<(), sea_orm::DbErr> {
        // implémentation mise à jour du hash
        todo!()
    }
}

// 2. Passer DefaultAdminAuth à la config admin
.with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
```

Pour brancher l'authentification au panneau d'administration, voir aussi [11-Admin.md](/docs/fr/admin).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](/docs/fr/auth/modele) | Built-in, trait `RuniqueUser` |
| [Helpers de session](/docs/fr/auth/session) | `login`, `auth_login`, `logout` |
| [Middlewares & CurrentUser](/docs/fr/auth/middleware) | Protection des routes |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
