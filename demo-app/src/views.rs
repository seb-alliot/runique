use crate::forms::{Blog as BlogForm, UsernameForm};
use crate::models::model_derive;
use crate::models::users;
use crate::models::users::Entity as UserEntity;
use runique::prelude::*;
use runique::{error, flash_now, info, success, warning};

/// Page d'accueil
pub async fn index(mut template: TemplateContext) -> Result<Response, AppError> {
    context_update!(template => {
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne inspiré de Django",
        "status" => "Framework en cours de développement...",
        "backend" => "Axum pour le backend",
        "template" => "Tera comme moteur de templates",
        "tokio" => "Runtime asynchrone tokio",
        "session" => "Tower pour la gestion des sessions",
    });
    
    template.render("index.html")
}

/// Formulaire d'inscription
pub async fn inscription(mut template: TemplateContext) -> Result<Response, AppError> {
    let form = model_derive::ModelForm::build(template.engine.tera.clone());
    context_update!(template => {
        "title" => "Inscription utilisateur",
        "inscription_form" => &form,
    });
    
    template.render("inscription_form.html")
}

/// Soumission du formulaire d'inscription
pub async fn soumission_inscription(
    mut template: TemplateContext,
    ExtractForm(mut form): ExtractForm<model_derive::ModelForm>,
) -> Result<Response, AppError> {
    let db = template.engine.db.clone();
    
    if form.is_valid().await {
        match form.save(&*db).await {
            Ok(user) => {
                success!(template.flash_manager => format!("Bienvenue {}, votre compte a été créé !", user.username));
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
                context_update!(template => {
                    "title" => "Erreur de base de données",
                    "inscription_form" => &form,
                    "messages" => &flash_now!(warning => "Veuillez corriger les erreurs ci-dessous"),
                });
                
                return template.render("inscription_form.html");
            }
        }
    }

    context_update!(template => {
        "title" => "Erreur de validation",
        "inscription_form" => &form,
        "messages" => &flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
    });
    
    template.render("inscription_form.html")
}

/// Formulaire de recherche d'utilisateur
pub async fn search_user_form(mut template: TemplateContext) -> Result<Response, AppError> {
    let mut form = UsernameForm::build(template.engine.tera.clone());
    println!("****************************************");
    println!("Création du formulaire de recherche d'utilisateur : {:?}", form);
    println!("****************************************");
    println!("valeur des field du formulaire : {:?}", form.get_form().fields);
    println!("****************************************");
    form.get_form_mut().add_value("csrf_token", &template.csrf_token.masked().as_str().to_string());
    println!("Après ajout du token CSRF, valeur des field du formulaire : {:?}", form.get_form().fields);
    context_update!(template => {
        "title" => "Rechercher un utilisateur",
        "user" => &form,
    });
    template.render("profile/view_user.html")
}

/// Exemple pour chercher un utilisateur
pub async fn info_user(
    mut template: TemplateContext,
    ExtractForm(form): ExtractForm<UsernameForm>,
) -> Result<Response, AppError> {
    let username = form.get_form().get_value("username").unwrap_or_default();
    let db = template.engine.db.clone();

    let user = UserEntity::objects
        .filter(users::Column::Username.eq(&username))
        .first(&db)
        .await?;

    match user {
        Some(user) => {
            context_update!(template => {
                "title" => "Vue utilisateur",
                "username" => &user.username,
                "email" => &user.email,
                "user" => &user,
                "messages" => &flash_now!(warning => &user.username),
            });
            
            template.render("profile/view_user.html")
        }
        None => {
            warning!(template.flash_manager  => format!("Utilisateur '{}' non trouvé.", username));
            template.render("profile/view_user.html")
        }
    }
}

/// Blog form
pub async fn blog_form(mut template: TemplateContext) -> Result<Response, AppError> {
    let form = BlogForm::build(template.engine.tera.clone());
    
    context_update!(template => {
        "title" => "Créer un article de blog",
        "blog_form" => &form,
    });
    
    template.render("blog/blog.html")
}

/// Page "À propos"
pub async fn about(mut template: TemplateContext) -> Result<Response, AppError> {
    success!(template.flash_manager => "Ceci est un message de succès.");
    info!(template.flash_manager => "Ceci est un message d'information.");
    warning!(template.flash_manager => "Ceci est un message d'avertissement.");
    error!(template.flash_manager => "Ceci est un message d'erreur.");
    println!("Flash messages ajoutés à la session.{:?}", template.flash_manager);

    context_update!(template => {
        "title" => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    });
    
    template.render("about/about.html")
}

/// Teste Csrf
pub async fn test_csrf(mut template: TemplateContext) -> Result<Response, AppError> {
    success!(template.flash_manager => "CSRF token validé avec succès !");
    Ok(Redirect::to("/").into_response())
}