use runique::config::StaticConfig;
use runique::prelude::*;
use serde::Serialize;

// --- USERNAME FORM ---
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UsernameForm {
    pub form: Forms,
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Entrez votre nom d'utilisateur")
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

// --- FORMULAIRE D'INSCRIPTION ---
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Entrez votre nom d'utilisateur")
                .required("votre pseudo est necessaire"),
        );

        form.field(
            &TextField::email("email")
                .label("Entrez votre email")
                .required("votre email est necessaire"),
        );

        form.field(
            &TextField::password("password")
                .label("Entrez un mot de passe")
                .required("Le mot de passe est obligatoire"),
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
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<crate::models::users::Model, DbErr> {
        use crate::models::users as users_mod;
        let new_user = users_mod::ActiveModel {
            username: Set(self.form.get_value("username").unwrap_or_default()),
            email: Set(self.form.get_value("email").unwrap_or_default()),
            password: Set(self.form.get_value("password").unwrap_or_default()),
            ..Default::default()
        };

        new_user.insert(db).await
    }
}

// --- FORMULAIRE BLOG ---
#[derive(Serialize)]

pub struct Blog {
    pub form: Forms,
}

impl RuniqueForm for Blog {
    fn register_fields(form: &mut Forms) {
        // Titre (Texte simple)
        form.field(
            &TextField::text("title")
                .label("Entrez un titre accrocheur")
                .required("Le titre est obligatoire"),
        );

        // Email de l'auteur
        form.field(
            &TextField::email("email")
                .label("Entrez l'email de l'auteur")
                .required("L'email est requis"),
        );

        // Site Web (URL)
        form.field(&TextField::url("website").label("Entrez le site web source"));

        // Résumé (TextArea)
        form.field(
            &TextField::textarea("summary")
                .label("Un court résumé...")
                .required("Veuillez fournir un résumé"),
        );

        // Contenu (TextArea ou RichText si implémenté dans GenericField)
        form.field(
            &TextField::richtext("content")
                .label("Entrez le contenu de l'article ici")
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
        use crate::models::blog as blog_mod;
        let new_blog = blog_mod::ActiveModel {
            title: Set(self.form.get_value("title").unwrap_or_default()),
            email: Set(self.form.get_value("email").unwrap_or_default()),
            website: Set(self.form.get_value("website")),
            summary: Set(self.form.get_value("summary").unwrap_or_default()),
            content: Set(self.form.get_value("content").unwrap_or_default()),
            ..Default::default()
        };

        new_blog.insert(db).await
    }
}

#[derive(Serialize)]
pub struct Image {
    #[serde(flatten)]
    pub form: Forms,
}

impl RuniqueForm for Image {
    fn register_fields(form: &mut Forms) {
        let config = StaticConfig::from_env();
        form.field(
            &FileField::image("image")
                .label("Choisissez une image à uploader")
                .upload_to(&config)
                .max_size_mb(5)
                .max_files(3)
                .max_dimensions(1920, 1080)
                .allowed_extensions(vec!["png", "jpg", "jpeg", "gif"]),
        );
        form.add_js("js/test_csrf.js");
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
