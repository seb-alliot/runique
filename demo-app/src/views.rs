use runique::prelude::*;

use crate::forms::Blog as blog_mod;
use crate::forms::PostForm as test_new_form;
use crate::forms::RegisterForm as register_form;
use crate::forms::UsernameForm as username_form;
use crate::models::model_derive::ModelForm as InscriptionForm;

use crate::models::users as users_mod;
use crate::models::users::Entity as UserEntity;

// Index.html
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

// Formulaire d'enregistrement utilisateur
pub async fn inscription(
    template: Template, 
    Extension(tera): Extension<Arc<Tera>>
    ) -> Response {
        
    let inscription_form = InscriptionForm::build(tera.clone());

    let ctx = context! {
        "title" => "Profil Utilisateur",
        "inscription_form" => inscription_form
    };
    template.render("inscription_form.html", &ctx)
}

// Soumission du formulaire d'enregistrement utilisateur
pub async fn soumissioninscription(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(mut inscription_form): ExtractForm<InscriptionForm>,
) -> Response {
    if inscription_form.is_valid().await {
        match inscription_form.save(&db).await {
            Ok(created_user) => {
                success!(message => "User profile created successfully!");
                success!(message => format!("Bienvenue, {} ! Votre compte a été créé avec l'ID {}.", created_user.username, created_user.id));
                
                return Redirect::to("/").into_response();
            }
            Err(db_err) => {
                let debug_error = inscription_form.get_form_mut();
                println!(
                    "Database error during user creation: {:?}",
                    debug_error.errors()
                );
                debug_error.database_error(&db_err);
                let ctx = context! {
                    "title" => "Erreur de base de données",
                    "inscription_form" => inscription_form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous")
                };
                return template.render("inscription_form.html", &ctx);
            }
        }
    }

    let ctx = context! {
        "title" => "Erreur de validation",
        "inscription_form" => inscription_form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous")
    };
    template.render("inscription_form.html", &ctx)
}

// Affichage d'un utilisateur
pub async fn cherche_user(template: Template, Extension(tera): Extension<Arc<Tera>>) -> Response {
    let user = username_form::build(tera.clone());

    let ctx = context! {
        "title" => "Rechercher un utilisateur",
        "user" => user
    };
    template.render("profile/view_user.html", &ctx)
}

// Soumission du formulaire de recherche utilisateur
pub async fn info_user(
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




// Formulaire avec des champs avancés
pub async fn blog_form(template: Template, Extension(tera): Extension<Arc<Tera>>) -> Response {
    let blog_form = blog_mod::build(tera.clone());

    let ctx = context! {
        "title" => "Blog form",
        "blog_form" => blog_form
    };
    template.render("blog/blog.html", &ctx)
}

// Soumission du formulaire avec des champs avancés
pub async fn soumission_blog_info(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(mut blog_form): ExtractForm<blog_mod>,
) -> Response {
    println!("=== Soumission du formulaire de blog ===");

    // Vérification de la validité du formulaire
    if blog_form.is_valid().await {
        match blog_form.save(&db).await {
            Ok(_saved_data) => {
                success!(message => "Formulaire sauvegardé avec succès !");
                return Redirect::to("/blog").into_response();
            }
            Err(db_err) => {
                println!("Erreur lors de la sauvegarde en base : {:?}", db_err);
                blog_form.get_form_mut().database_error(&db_err);

                let ctx = context! {
                    "title" => "Erreur de base de données",
                    "blog_form" => blog_form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous")
                };
                return template.render("blog/blog.html", &ctx);
            }
        }
    }

    let ctx = context! {
        "title" => "Erreur de retour",
        "blog_form" => blog_form
    };
    template.render("blog/blog.html", &ctx)
}





// Test du formulaire avec des champs avancés
pub async fn test_champs_form(
    Extension(tera): Extension<Arc<Tera>>,
    template: Template,
) -> Response {
    let test_new_form = test_new_form::build(tera.clone());

    let ctx = context! {
        "title" => "Test des champs",
        "test_new_form" => test_new_form
    };
    template.render("test_champs_form.html", &ctx)
}

// Soumission du formulaire avec des champs avancés
pub async fn soumission_champs_form(
    Extension(_db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
    ExtractForm(mut test_new_form): ExtractForm<test_new_form>,
) -> Response {
    println!("=== Soumission du formulaire ===");
    if test_new_form.is_valid().await {
        println!("Formulaire valide. Tentative de sauvegarde en base...");
        success!(message => "Formulaire sauvegardé avec succès !");
        return Redirect::to("/").into_response();
    } else {
        println!("Formulaire invalide. Erreurs présentes :");
        for (field, errors) in test_new_form.form.errors() {
            println!(" - Champ '{}': {}", field, errors);
        }
    }
    let ctx = context! {
        "title" => "Test des champs",
        "test_champs_form" => test_new_form
    };
    template.render("test_champs_form.html", &ctx)
}








// test de script en js pour CSRF
pub async fn test_csrf(mut message: Message) -> Response {
    success!(message => "CSRF token validé avec succès !");
    Redirect::to("/").into_response()
}

// À propos de Runique
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










// test d'un formulaire avec derive_form
pub async fn affiche_form_generer(
    Extension(tera): Extension<Arc<Tera>>,
    template: Template,
) -> Response {
    let register_form = register_form::build(tera.clone());

    let ctx = context! {
        "title" => "Formulaire d'inscription généré",
        "register_form" => register_form
    };

    template.render("profile/register_form.html", &ctx)
}

// POST - Traiter la soumission
pub async fn soumission_form_generer(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
    ExtractForm(mut register_form): ExtractForm<register_form>,
) -> Response {
    if register_form.is_valid().await {
        // Sauvegarder en base de données
        match register_form.save(&db).await {
            Ok(user) => {
                success!(message => format!("Compte créé avec succès !"));
                println!("Nouvel utilisateur créé : {:?}", user);
                return Redirect::to("/").into_response();
            }
            Err(e) => {
                register_form.database_error(&e);
                let ctx = context! {
                    "title" => "Formulaire d'inscription généré",
                    "register_form" => register_form,
                    "messages" => flash_now!(warning => "Erreur lors de la sauvegarde en base, veuillez corriger les erreurs.")
                };
                println!("Erreur lors de la sauvegarde en base : {:?}", e);
                return template.render("profile/register_form.html", &ctx);
            }
        }
    }

    // Réafficher le formulaire avec les erreurs
    let ctx = context! {
        "title" => "Formulaire d'inscription généré",
        "register_form" => register_form
    };
    println!("Formulaire invalide, réaffichage avec erreurs.");
    template.render("profile/register_form.html", &ctx)
}
