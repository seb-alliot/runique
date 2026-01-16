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
        form.register_field(
            &GenericField::new_text("username", "Nom d'utilisateur").required("Le nom est requis"),
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
pub struct PostForm {
    pub form: Forms,
}

impl Serialize for PostForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for PostForm {
    fn register_fields(form: &mut Forms) {
        // Titre : Utilisation de limites_caractere (min, max)
        form.register_field(
            &GenericField::new_text("title", "Titre de l'article")
                .placeholder("Entrez le titre ici")
                .required("Le titre est obligatoire")
                .limites_caractere(
                    None,
                    Some(150),
                    "Le titre ne peut pas dépasser 150 caractères",
                ),
        );

        // Email
        form.register_field(
            &GenericField::new_email("author_email", "Email de l'auteur")
                .placeholder("Entrez l'email de l'auteur")
                .required("L'email de l'auteur est obligatoire"),
        );

        // URL
        form.register_field(
            &GenericField::new_url("website", "Site web source")
                .placeholder("Entrez le site web source"),
        );

        // Résumé (TextArea)
        form.register_field(
            &GenericField::new_textarea("summary", "Résumé")
                .placeholder("Entrez le résumé ici")
                .required("Le résumé est obligatoire"),
        );

        // Contenu (RichText)
        form.register_field(
            &GenericField::new_richtext("content", "Contenu de l'article")
                .required("Le contenu est obligatoire"),
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

// --- FORMULAIRE D'INSCRIPTION ---
pub struct RegisterForm {
    pub form: Forms,
}

impl Serialize for RegisterForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        // Utilisation de GenericField au lieu de TextField/EmailField
        form.register_field(&GenericField::new_text("username", "Pseudo"));

        form.register_field(
            &GenericField::new_email("email", "Mon email").required("L'email est requis"),
        );

        form.register_field(
            &GenericField::new_password("password", "Mot de passe").required("Sécurité requise"),
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
pub struct Blog {
    pub form: Forms,
}

impl Serialize for Blog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for Blog {
    fn register_fields(form: &mut Forms) {
        // Titre (Texte simple)
        form.register_field(
            &GenericField::new_text("title", "Titre de l'article")
                .placeholder("Entrez un titre accrocheur")
                .required("Le titre est obligatoire"),
        );

        // Email de l'auteur
        form.register_field(
            &GenericField::new_email("email", "Email de l'auteur").required("L'email est requis"),
        );

        // Site Web (URL)
        form.register_field(&GenericField::new_url("website", "Site Web source"));

        // Résumé (TextArea)
        form.register_field(
            &GenericField::new_textarea("summary", "Résumé")
                .placeholder("Un court résumé...")
                .required("Veuillez fournir un résumé"),
        );

        // Contenu (TextArea ou RichText si implémenté dans GenericField)
        form.register_field(
            &GenericField::new_textarea("content", "Contenu de l'article")
                .required("Le contenu ne peut pas être vide"),
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
