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
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne inspiré de Django",
        "status" => "Framework en cours de développement...",
        "backend" => "J'utilise axum pour le backend",
        "template" => "Tera pour moteur de templates.",
        "tokio" => "Le runtime asynchrone tokio",
        "session" => "Tower pour la gestion des sessions."
    };

    template.render("index.html", &ctx)
}

pub async fn form_register_user(
    template: Template,
    Extension(tera): Extension<Arc<Tera>>,
) -> Response {
    let register_form = ModelForm::build(tera.clone());

    let ctx = context! {
        "title" => "Profil Utilisateur",
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
    // 1. Vérifier la validation de base
    if register_form.is_valid() {
        // 2. Essayer de sauvegarder en BDD
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
                let ctx = context! {
                    "title" => "Erreur de base de données",
                    "register_form" => register_form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous")
                };
                return template.render("profile/register_user.html", &ctx);
            }
        }
    }

    let ctx = context! {
        "title" => "Erreur de validation",
        "register_form" => register_form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous")
    };
    template.render("profile/register_user.html", &ctx)
}

pub async fn user(template: Template, Extension(tera): Extension<Arc<Tera>>) -> Response {
    let user = UsernameForm::build(tera.clone());

    let ctx = context! {
        "title" => "Rechercher un utilisateur",
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
                "title" => "Vue Utilisateur",
                "username" => &user.username,
                "email" => &user.email,
                "age" => &user.age,
                "user" => form
            };
            ctx.insert(
                "messages",
                &flash_now!(warning => &user.username.to_uppercase(), ("Existe , louez moi !!!").to_uppercase(), "en promotion chez C DISCOUNT"),
            );
            template.render("profile/view_user.html", &ctx)
        }
        Ok(None) => {
            let mut ctx = context! {
                "title" => "Utilisateur non trouvé",
                "user" => &form
            };
            let message_okanime = format!("{} !!!", name.to_uppercase());
            ctx.insert(
                "messages",
                &flash_now!(error => message_okanime, " Je ne te connais pas dans ma BDD, Come HERE NOW :p "),
            );
            template.render("profile/view_user.html", &ctx)
        }
        Err(_) => template.render_500("Erreur lors de la recherche"),
    }
}

/// Page "À propos"
pub async fn about(template: Template, mut message: Message) -> Response {
    success!(message => "Ceci est un message de succès de test.");
    info!(message => "Ceci est un message d'information de test.");
    error!(message => "Ceci est un message d'erreur de test.");
    warning!(message => "Ceci est un message d'avertissement de test.");

    let ctx = context! {
        "title", "À propos de Runique Framework";
        "content", "Runique est un framework web inspiré de Django, construit sur Axum et Tera."
    };
    template.render("about/about.html", &ctx)
}

/// Ajax test CSRF
pub async fn test_csrf(mut message: Message) -> Response {
    success!(message => "CSRF token validé avec succès !");
    Redirect::to("/").into_response()
}
