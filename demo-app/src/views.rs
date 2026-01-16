use runique::axum::Extension;
use runique::prelude::*;
use std::sync::Arc;
use tera::Tera;

use crate::forms::PostForm as test_new_form;
use crate::forms::RegisterForm as register_form;
use crate::forms::UsernameForm as username_form;

use crate::models::users as users_mod;
use crate::models::users::Entity as UserEntity;

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
    let register_form = register_form::build(tera.clone());

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
    ExtractForm(mut register_form): ExtractForm<register_form>,
) -> Response {
    if register_form.form.is_valid() {
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
                let debug_error = register_form.get_form_mut();
                println!("Database error during user creation: {:?}", debug_error.all_errors());
                debug_error.database_error(&db_err);
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
    let user = username_form::build(tera.clone());

    let ctx = context! {
        "title" => "Rechercher un utilisateur",
        "user" => user
    };
    template.render("profile/view_user.html", &ctx)
}

pub async fn view_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(user): ExtractForm<username_form>,
) -> Response {
    let name: String = user.get_form().get_value("username").unwrap_or_default();

    match UserEntity::objects
        .filter(users_mod::Column::Username.eq(&name))
        .first(&db)
        .await
    {
        Ok(Some(user)) => {
            let mut ctx = context! {
                "title" => "Vue Utilisateur",
                "username" => &user.username,
                "email" => &user.email,
                "user" => &user
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
                "user" => &user
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

pub async fn show_form(template: Template, Extension(tera): Extension<Arc<Tera>>) -> Response {
    let test_form = register_form::build(tera.clone());

    let ctx = context! {
        "title" => "Test des champs",
        "test_form" => test_form
    };
    template.render("test-field/test.html", &ctx)
}

pub async fn submit_form(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(mut test_form): ExtractForm<register_form>,
) -> Response {
    println!("=== Soumission du formulaire ===");

    // Récupération des données nettoyées

    // Affiche chaque champ avec type et valeur
    for (key, value) in test_form.form.cleaned_data().iter() {
        match value {
            tera::Value::String(s) => println!("Champ '{}' (String): '{}'", key, s),
            tera::Value::Number(n) => println!("Champ '{}' (Number): {}", key, n),
            tera::Value::Bool(b) => println!("Champ '{}' (Bool): {}", key, b),
            _ => println!("Champ '{}' (Autre type): {:?}", key, value),
        }
    }

    // Vérification de la validité du formulaire
    if test_form.form.is_valid() {
        println!("Formulaire valide. Tentative de sauvegarde en base...");
        match test_form.save(&db).await {
            Ok(_saved_data) => {
                println!("Formulaire sauvegardé avec succès !");
                success!(message => "Formulaire sauvegardé avec succès !");
                return Redirect::to("/test-fields").into_response();
            }
            Err(db_err) => {
                println!("Erreur lors de la sauvegarde en base : {:?}", db_err);
                test_form.get_form_mut().database_error(&db_err);

                let ctx = context! {
                    "title" => "Erreur de base de données",
                    "test_form" => test_form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous")
                };
                return template.render("test-field/test.html", &ctx);
            }
        }
    } else {
        println!("Formulaire invalide. Erreurs présentes :");
        for (field, errors) in test_form.form.all_errors() {
            println!(" - Champ '{}': {}", field, errors);
        }
    }

    // Ré-affichage du formulaire avec erreurs
    let ctx = context! {
        "title" => "Test des champs",
        "test_form" => test_form
    };
    template.render("test-field/test.html", &ctx)
}

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

pub async fn test_csrf(mut message: Message) -> Response {
    success!(message => "CSRF token validé avec succès !");
    Redirect::to("/").into_response()
}

pub async fn test_new_form(Extension(tera): Extension<Arc<Tera>>, template: Template) -> Response {
    let test_new_form = test_new_form::build(tera.clone());

    let ctx = context! {
        "title" => "Test des champs",
        "test_new_form" => test_new_form
    };
    template.render("test_new_form.html", &ctx)
}

pub async fn test_new_form_submit(
    Extension(_db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
    ExtractForm(mut test_new_form): ExtractForm<test_new_form>,
) -> Response {
    println!("=== Soumission du formulaire ===");
    // Récupération des données nettoyées
    // Vérification de la validité du formulaire
    if test_new_form.form.is_valid() {
        println!("Formulaire valide. Tentative de sauvegarde en base...");
        // Ici, vous pouvez ajouter la logique pour sauvegarder les données en base
        success!(message => "Formulaire sauvegardé avec succès !");
        return Redirect::to("/test-new-form").into_response();
    } else {
        println!("Formulaire invalide. Erreurs présentes :");
        for (field, errors) in test_new_form.form.all_errors() {
            println!(" - Champ '{}': {}", field, errors);
        }
    }
    // Ré-affichage du formulaire avec erreurs
    let ctx = context! {
        "title" => "Test des champs",
        "test_new_form" => test_new_form
    };
    template.render("test_new_form.html", &ctx)
}
