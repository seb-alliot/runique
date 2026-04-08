use crate::formulaire::{LoginForm, RegisterForm};
use runique::middleware::auth::login as auth_login;
use runique::prelude::user::Entity as UserEntity;
use runique::prelude::*;

pub async fn register_user(
    form: &RegisterForm,
    db: &sea_orm::DatabaseConnection,
) -> Result<runique::prelude::user::Model, sea_orm::DbErr> {
    form.save(db).await
}

pub async fn authenticate_user(
    db: &sea_orm::DatabaseConnection,
    username: &str,
    password: &str,
) -> Option<runique::prelude::user::Model> {
    let user = find_user_by_username(db, username).await?;
    if user.is_active && verify(password, &user.password) {
        Some(user)
    } else {
        None
    }
}

pub async fn find_user_by_username(
    db: &sea_orm::DatabaseConnection,
    username: &str,
) -> Option<runique::prelude::user::Model> {
    eprintln!("=== DEBUG find_user_by_username ===");
    eprintln!("Username reçu: {:?}", username);
    eprintln!("Username bytes: {:?}", username.as_bytes());

    // Construit la requête
    let query = search!(UserEntity => Username eq username.trim());

    // Log le SQL généré (si possible)
    // Note: Sea-ORM ne expose pas facilement le SQL, mais on peut vérifier le résultat

    let result = query.first(db).await;
    eprintln!("Résultat SQL: {:?}", result);

    result.unwrap_or(None)
}

pub async fn find_user_by_id(
    db: &sea_orm::DatabaseConnection,
    id: runique::utils::pk::Pk,
) -> Option<runique::prelude::user::Model> {
    UserEntity::find_by_id(id).one(db).await.unwrap_or(None)
}

pub async fn handle_inscription(
    request: &mut Request,
    form: &mut RegisterForm,
) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }
    let template = "auth/inscription.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = crate::backend::fetch_page_examples("inscription", &db).await;
    if request.is_get() {
        context_update!(request => {
            "title"            => "User registration",
            "inscription_form" => &*form,
            "code_examples"    => &code_examples,
            "doc_links"        => &doc_links,
        });
        return request.render(template);
    }
    if request.is_post() && form.is_valid().await {
        match register_user(form, &request.engine.db).await {
            Ok(user) => {
                auth_login(
                    &request.session,
                    &request.engine.db,
                    user.id,
                    &user.username,
                    false,
                    false,
                    None,
                    false,
                )
                .await
                .ok();
                success!(request.notices => format!("Welcome {}! Your account has been created.", user.username));
                return Ok(Redirect::to("/profil").into_response());
            }
            Err(err) => form.get_form_mut().database_error(&err),
        }
    }
    context_update!(request => {
        "title"            => "Validation error",
        "inscription_form" => &*form,
        "code_examples"    => &code_examples,
        "doc_links"        => &doc_links,
        "messages"         => flash_now!(error => "Please correct the errors"),
    });
    request.render(template)
}

pub async fn handle_login(request: &mut Request, form: &mut LoginForm) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }
    let template = "auth/login.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = crate::backend::fetch_page_examples("login", &db).await;
    if request.is_get() {
        context_update!(request => {
            "title"         => "Login",
            "login_form"    => form,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
        });
        return request.render(template);
    }
    if request.is_post() && form.is_valid().await {
        if let Some((username_val, password_val)) = get_credentials(form)
            && let Some(user) =
                authenticate_user(&request.engine.db, &username_val, &password_val).await
        {
            auth_login(
                &request.session,
                &request.engine.db,
                user.id,
                &user.username,
                user.is_staff,
                user.is_superuser,
                None,
                false,
            )
            .await
            .ok();
            success!(request.notices => format!("Welcome {}!", user.username));
            return Ok(Redirect::to("/profil").into_response());
        }
        context_update!(request => {
            "title"         => "Login",
            "login_form"    => form,
            "auth_error"    => &true,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
            "messages"      => flash_now!(error => "Invalid credentials"),
        });
        return request.render(template);
    }
    request.render(template)
}

pub fn get_credentials(form: &LoginForm) -> Option<(String, String)> {
    let u = form.get_form().get_value("username").unwrap_or_default();
    let p = form.get_form().get_value("password").unwrap_or_default();
    if u.is_empty() || p.is_empty() {
        None
    } else {
        Some((u, p))
    }
}

pub async fn get_profile_user(
    user_id: Option<runique::utils::pk::Pk>,
    db: &sea_orm::DatabaseConnection,
) -> Option<runique::prelude::user::Model> {
    find_user_by_id(db, user_id?).await
}
