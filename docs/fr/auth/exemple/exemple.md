# Exemple complet & AdminPanel

## Exemple complet — Login / Logout

```rust
use runique::prelude::*;

pub async fn login_post(
    mut request: Request,
    Prisme(mut form): Prisme<LoginForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        // 1. Chercher l'utilisateur par username
        let username = form.get_form().get_string("username");
        let user = users::Entity::objects
            .filter(users::Column::Username.eq(&username))
            .first(&*request.engine.db)
            .await?;

        if let Some(user) = user {
            // 2. Vérifier le mot de passe (en clair vs hash)
            let password = form.get_form().get_string("password");
            if verify(&password, &user.password) {
                // 3. Ouvrir la session (charge les données depuis la DB par user_id)
                auth_login(&request.session, &request.engine.db, user.id).await.ok();
                return Ok(Redirect::to("/dashboard").into_response());
            }
        }

        // Identifiants invalides
        context_update!(request => {
            "login_form" => &form,
            "messages" => flash_now!(error => "Identifiants invalides"),
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
