use crate::models::users as users_mod;
use runique::prelude::*;
use runique::serde::{Serialize, Serializer};
use sea_orm::DbErr;
use sea_orm::{ActiveModelTrait, Set};
// pub struct PostModel {
//     pub title: String,
//     pub author_email: String,
//     pub website: String,
//     pub content: String,
//     pub summary: String,
// }

// Dans ton fichier forms.rs (ou là où sont tes formulaires)

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
        // C'est ici qu'on définit les champs
        form.register_field(&TextField::new("username", "Pseudo").max_length(50, "Trop long"));
        form.register_field(&EmailField::new("email","Mon email").required("L'email est requis",
        ));
        form.register_field(
            &PasswordField::new("password", "Mot de passe").required("Sécurité requise"),
        );
    }

    // Boilerplate nécessaire pour le trait
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
        // 1. On récupère les valeurs nettoyées des champs
        // get_value cherche dans Forms.fields le champ par son nom
        let username = self.form.get_value("username").unwrap_or_default();
        let email = self.form.get_value("email").unwrap_or_default();
        let password = self.form.get_value("password").unwrap_or_default();

        // 2. On crée l'ActiveModel SeaORM
        let new_user = users_mod::ActiveModel {
            username: Set(username),
            email: Set(email),
            password: Set(password), // Si PasswordField, la valeur est déjà hashée !
            ..Default::default()
        };

        // 3. On insère en base de données
        new_user.insert(db).await
    }
}
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
            &TextField::new("username", "Nom d'utilisateur").required("Le nom est requis"),
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
        form.register_field(
            &TextField::new("title", "Titre de l'article")
                .placeholder("Entrez le titre ici")
                .required("Le titre est obligatoire")
                .max_length(150, "Le titre ne peut pas dépasser 150 caractères"),
        );

        form.register_field(
            &EmailField::new("author_email", "Email de l'auteur")
                .placeholder("Entrez l'email de l'auteur")
                .required("L'email de l'auteur est obligatoire"),
        );

        form.register_field(
            &URLField::new("website", "Site web source").placeholder("Entrez le site web source"),
        );

        form.register_field(
            &TextAreaField::new("summary", "Résumé (texte brut)", "Entrez le résumé ici")
                .required("Le résumé est obligatoire"),
        );

        form.register_field(
            &RichTextField::new("content", "Contenu de l'article")
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
        // Champ Titre
        form.register_field(
            &TextField::new("title", "Titre de l'article")
                .placeholder("Entrez un titre accrocheur")
                .required("Le titre est obligatoire")
                .max_length(150, "Le titre est trop long"),
        );

        // Champ Email de l'auteur
        form.register_field(
            &EmailField::new("author_email", "Email de l'auteur")
                .placeholder("auteur@exemple.com")
                .required("L'email est requis pour le suivi"),
        );

        // Champ Site Web (Optionnel)
        form.register_field(
            &URLField::new("website", "Site Web source")
                .placeholder("https://..."),
        );

        // Champ Résumé (Simple texte)
        form.register_field(
            &TextAreaField::new("summary", "Résumé", "Un court résumé de l'article...")
                .required("Veuillez fournir un résumé"),
        );

        // Champ Contenu (Peut être un RichText si tu l'as implémenté)
        form.register_field(
            &TextAreaField::new("content", "Contenu de l'article", "Écrivez votre article ici...")
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
    pub async fn save(&self, db: &DatabaseConnection) -> Result<crate::models::blog::Model, sea_orm::DbErr> {
        use sea_orm::{ActiveModelTrait, Set};
        use crate::models::Blog as post_mod;

        // Extraction des valeurs nettoyées
        let title = self.form.get_value("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let author_email = self.form.get_value("author_email").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let website = self.form.get_value("website").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let content = self.form.get_value("content").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let summary = self.form.get_value("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();

        let new_post = post_mod::ActiveModel {
            title: Set(title),
            author_email: Set(author_email),
            website: Set(website),
            content: Set(content),
            summary: Set(summary),
            // id et created_at sont gérés automatiquement si configurés en base
            ..Default::default()
        };

        new_post.insert(db).await
    }
}