use crate::forms::UsernameForm;
use crate::models::users;
use crate::models::users::Entity as User;
use crate::models::users::ModelForm;
use runique::axum::Extension;
use runique::prelude::*;
use std::sync::Arc;

/// Page d'accueil
pub async fn index(template: Template) -> Response {
    let ctx = context! {
    "title", "Bienvenue sur Runique";
    "description", "Un framework web moderne inspiré de Django";
    "status", "Framework en cours de développement...";
    "backend", "J'utilise axum pour le backend";
    "template", "Tera pour moteur de templates.";
    "tokio", "Le runtime asynchrone tokio";
    "session", "Tower pour la gestion des sessions."};

    template.render("index.html", &ctx)
}

pub async fn form_register_user(template: Template) -> Response {
    let ctx = context! {
        "title", "Profil Utilisateur";
        "form", ModelForm::build();
    };
    template.render("profile/register_profile.html", &ctx)
}

/// GET - Afficher le profil utilisateur
pub async fn user_profile_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(user): ExtractForm<ModelForm>,
) -> Response {
    if user.is_valid() {
        match user.save(&db).await {
            Ok(created_user) => {
                success!(message, "Profil utilisateur créé avec succès !");
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
            Err(err) => {
                let error_msg = if err.to_string().contains("unique") {
                    if err.to_string().contains("username") {
                        "Ce nom d'utilisateur existe déjà !"
                    } else if err.to_string().contains("email") {
                        "Cet email est déjà utilisé !"
                    } else {
                        "Cette valeur existe déjà"
                    }
                } else {
                    "Fait ta migrations de modele stp !"
                };

                let mut reloaded_form = user.rebuild_form();
                reloaded_form
                    .get_form_mut()
                    .errors
                    .insert("username".to_string(), error_msg.to_string());

                // On prépare manuellement l'objet message pour Tera
                let mut ctx = context! {
                    "form", &reloaded_form;
                    "title", "Erreur de base de données";
                    "forms_errors", &reloaded_form.get_errors();
                };
                ctx.insert(
                    "messages",
                    &flash_now!(
                        error,
                        "Une erreur est survenue lors de la création du profil utilisateur"
                    ),
                );

                return template.render("profile/register_profile.html", &ctx);
            }
        }
    }

    error!(message, "Erreur de validation du formulaire");
    let ctx = context! {
        "form", &user;
        "forms_errors", &user.get_errors();
        "title", "Erreur de validation"
    };
    template.render("profile/register_profile.html", &ctx)
}

/// GET - Affiche la page avec le formulaire de recherche
pub async fn user(template: Template) -> Response {
    let ctx = context! {
        "title", "Rechercher un utilisateur";
        "form", UsernameForm::build()
    };
    template.render("profile/view_user.html", &ctx)
}

/// POST - Traite le formulaire de recherche et affiche les résultats
pub async fn view_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(form): ExtractForm<UsernameForm>,
) -> Response {
    let name: String = form.form.get_value("username").unwrap_or_default();

    // Rechercher l'utilisateur dans la base de données
    match User::objects
        .filter(users::Column::Username.eq(&name))
        .first(&db)
        .await
    {
        Ok(Some(user)) => {
            // Utilisateur trouvé
            let ctx = context! {
                "title", "Vue Utilisateur";
                "username", &user.username;
                "email", &user.email;
                "form", UsernameForm::build()
            };
            success!(message, "Utilisateur trouvé avec succès.");
            template.render("profile/view_user.html", &ctx)
        }
        Ok(None) => {
            // Utilisateur non trouvé
            error!(message, "Utilisateur non trouvé flash  message.");
            let ctx = context! {
                "title", "Utilisateur non trouvé";
                "form", UsernameForm::rebuild_form(&form)
            };
            template.render("profile/view_user.html", &ctx)
        }
        Err(_) => {
            // Erreur base de données - 500
            template.render_500("Erreur lors de la recherche")
        }
    }
}

/// Page "À propos"
pub async fn about(template: Template, mut message: Message) -> Response {
    success!(message, "Ceci est un message de succès de test.");
    info!(message, "Ceci est un message d'information de test.");
    error!(message, "Ceci est un message d'erreur de test.");
    warning!(message, "Ceci est un message d'avertissement de test.");

    let ctx = context! {
        "title", "À propos de Runique Framework";
        "content", "Runique est un framework web inspiré de Django, construit sur Axum et Tera."
    };
    template.render("about/about.html", &ctx)
}

/// Ajax test CSRF
pub async fn test_csrf(mut message: Message) -> Response {
    success!(message, "CSRF token validé avec succès !");
    Redirect::to("/").into_response()
}
