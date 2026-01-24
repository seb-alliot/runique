use runique::prelude::*;

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
                .label("Entrez votre nom")
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
