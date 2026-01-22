use crate::forms::UsernameForm;
use crate::models::users;
use crate::models::users::Entity as User;
use crate::models::users::ModelForm;
use runique::axum::Extension;
use runique::prelude::*;
use std::sync::Arc;
use tera::Tera;

pub async fn index(template: Template) -> Response {
    let ctx = context! {
        "title" => "Welcome to Runique",
        "description" => "A modern web framework inspired by Django",
        "status" => "Framework under development...",
        "backend" => "I use Axum for the backend",
        "template" => "Tera for template engine.",
        "tokio" => "The tokio asynchronous runtime",
        "session" => "Tower for session management."
    };

    template.render("index.html", &ctx)
}

pub async fn form_register_user(
    template: Template,
    Extension(tera): Extension<Arc<Tera>>,
) -> Response {
    let register_form = ModelForm::build(tera.clone());

    let ctx = context! {
        "title" => "User Profile",
        "register_form" => register_form
    };
    template.render("profile/register_user.html", &ctx)
}

pub async fn user_profile_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(mut register_form): ExtractForm<ModelForm>,
) -> Response {
    if register_form.is_valid() {
        match register_form.save(&db).await {
            Ok(created_user) => {
                success!(message => "User profile created successfully!");
                let target = reverse_with_parameters(
                    "user_profile",
                    &[
                        ("id", &created_user.id.to_string()),
                        ("name", &created_user.username),
                    ],
                )
                .unwrap();
                return Redirect::to(&target).into_response();
            }
            Err(db_err) => {
                register_form.get_form_mut().handle_database_error(&db_err);

                let mut ctx = context! {
                    "title" => "Database error",
                    "register_form" => register_form
                };

                ctx.insert(
                    "messages",
                    &flash_now!(error => "Database error occurred"),
                );

                return template.render("profile/register_user.html", &ctx);
            }
        }
    }

    let ctx = context! {
        "title" => "Validation error",
        "register_form" => register_form,
        "messages" => flash_now!(error => "Form validation error")
    };
    template.render("profile/register_user.html", &ctx)
}

pub async fn user(template: Template, Extension(tera): Extension<Arc<Tera>>) -> Response {
    let user = UsernameForm::build(tera.clone());

    let ctx = context! {
        "title" => "Search for a user",
        "user" => user
    };
    template.render("profile/view_user.html", &ctx)
}

pub async fn view_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(form): ExtractForm<UsernameForm>,
) -> Response {
    let name: String = form.get_form().get_value("username").unwrap_or_default();

    match User::objects
        .filter(users::Column::Username.eq(&name))
        .first(&db)
        .await
    {
        Ok(Some(user)) => {
            let mut ctx = context! {
                "title" => "User View",
                "username" => &user.username,
                "email" => &user.email,
                "age" => &user.age,
                "user" => form
            };
            ctx.insert(
                "messages",
                &flash_now!(warning => &user.username.to_uppercase(), "Welcome to Runique!"),
            );
            template.render("profile/view_user.html", &ctx)
        }
        Ok(None) => {
            let mut ctx = context! {
                "title" => "User not found",
                "user" => &form
            };
            let message_okanime = format!("{} !!!", name.to_uppercase());
            ctx.insert(
                "messages",
                &flash_now!(error => message_okanime, "I don't know you in my DB, Come HERE NOW :p"),
            );
            template.render("profile/view_user.html", &ctx)
        }
        Err(_) => template.render_500("Error during search"),
    }
}

/// "About" page
pub async fn about(template: Template, mut message: Message) -> Response {
    success!(message => "This is a test success message.");
    info!(message => "This is a test information message.");
    error!(message => "This is a test error message.");
    warning!(message => "This is a test warning message.");

    let ctx = context! {
        "title" => "About Runique Framework",
        "content" => "Runique is a web framework inspired by Django, built on Axum and Tera."
    };
    template.render("about/about.html", &ctx)
}
