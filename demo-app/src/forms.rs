use crate::models::users as users_mod;
use runique::prelude::*;
use runique::serde::{Serialize, Serializer};
use sea_orm::DbErr;
use sea_orm::{ActiveModelTrait, Set};

// --- USERNAME FORM ---
pub struct UsernameForm {
    pub form: Forms,
}

impl Serialize for UsernameForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .placeholder("Entrez votre nom")
                .required("Le nom est requis"),
        );
    }
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

// --- POST FORM ---
#[derive(Serialize)]
#[serde(transparent)]
pub struct PostForm {
    pub form: Forms,
}

impl RuniqueForm for PostForm {
    fn register_fields(form: &mut Forms) {
        // Titre : Utilisation de limites_caractere (min, max)
        form.field(
            &TextField::text("title")
                .placeholder("Entrez le titre ici")
                .required("Le titre est obligatoire")
                .min_length(10, "Le titre doit contenir au moins 10 caractères"),
        );

        // Email
        form.field(
            &TextField::email("email")
                .placeholder("Entrez l'email de l'auteur")
                .required("L'email de l'auteur est obligatoire"),
        );

        // URL
        form.field(&TextField::url("website").placeholder("Entrez le site web source"));

        // Résumé (TextArea)
        form.field(
            &TextField::textarea("summary")
                .placeholder("Entrez le résumé ici")
                .required("Le résumé est obligatoire"),
        );

        // Contenu (RichText)
        form.field(&TextField::richtext("content").required("Le contenu est obligatoire"));
    }

    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

// --- FORMULAIRE D'INSCRIPTION ---
#[derive(Serialize)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .placeholder("Entrez votre nom d'utilisateur")
                .required("Le nom d'utilisateur est requis"),
        );

        form.field(
            &TextField::email("email")
                .placeholder("Entrez votre email")
                .required("L'email est requis"),
        );

        form.field(&TextField::password("password").required("Sécurité requise"));
    }

    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users_mod::Model, DbErr> {
        let username = self.form.get_value("username").unwrap_or_default();
        let email = self.form.get_value("email").unwrap_or_default();

        // Optionnel : Tu peux utiliser field.hash_password() ici si tu récupères l'instance
        let password = self.form.get_value("password").unwrap_or_default();
        let new_user = users_mod::ActiveModel {
            username: Set(username),
            email: Set(email),
            password: Set(password),
            ..Default::default()
        };

        new_user.insert(db).await
    }
}

// --- FORMULAIRE BLOG ---
#[derive(Serialize)]
#[serde(transparent)]
pub struct Blog {
    pub form: Forms,
}

impl RuniqueForm for Blog {
    fn register_fields(form: &mut Forms) {
        // Titre (Texte simple)
        form.field(
            &TextField::text("title")
                .placeholder("Entrez un titre accrocheur")
                .required("Le titre est obligatoire"),
        );

        // Email de l'auteur
        form.field(
            &TextField::email("email")
                .placeholder("Entrez l'email de l'auteur")
                .required("L'email est requis"),
        );

        // Site Web (URL)
        form.field(&TextField::url("website").placeholder("Entrez le site web source"));

        // Résumé (TextArea)
        form.field(
            &TextField::textarea("summary")
                .placeholder("Un court résumé...")
                .required("Veuillez fournir un résumé"),
        );

        // Contenu (TextArea ou RichText si implémenté dans GenericField)
        form.field(
            &TextField::richtext("content")
                .placeholder("Entrez le contenu de l'article ici")
                .required("Le contenu ne peut pas être vide"),
        );

        form.field(
            &TextField::password("password")
                .placeholder("Entrez un mot de passe sécurisé")
                .required("Le mot de passe est obligatoire")
                .min_length(8, "Le mot de passe doit contenir au moins 8 caractères"),
        );
    }

    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

impl Blog {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<crate::models::blog::Model, DbErr> {
        use crate::models::blog as post_mod;

        let new_post = post_mod::ActiveModel {
            title: Set(self.form.get_value("title").unwrap_or_default()),
            email: Set(self.form.get_value("email").unwrap_or_default()),
            website: Set(self.form.get_value("website").unwrap_or_default()),
            content: Set(self.form.get_value("content").unwrap_or_default()),
            summary: Set(self.form.get_value("summary").unwrap_or_default()),
            ..Default::default()
        };

        new_post.insert(db).await
    }
}
