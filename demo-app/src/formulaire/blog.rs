use runique::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "blog")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub email: String, // Email de l'auteur
    pub website: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub summary: String, // Pour le TextArea
    #[sea_orm(column_type = "Text")]
    pub content: String, // Pour le RichText
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: NotSet,
            title: NotSet,
            email: NotSet,
            website: NotSet,
            summary: NotSet,
            content: NotSet,
            created_at: Set(chrono::Utc::now()),
        }
    }
}

impl_objects!(Entity);

// --- FORMULAIRE BLOG ---
#[derive(Serialize)]

pub struct BlogForm {
    pub form: Forms,
}

impl RuniqueForm for BlogForm {
    fn register_fields(form: &mut Forms) {
        // Titre (Texte simple)
        form.field(
            &TextField::text("title")
                .label("Entrez un titre accrocheur")
                .required()
                .min_length(10, "Le titre doit contenir au moins 10 caractères"),
        );

        // Email de l'auteur
        form.field(
            &TextField::email("email")
                .label("Entrez l'email de l'auteur")
                .required(),
        );

        // Site Web (URL)
        form.field(&TextField::url("website").label("Entrez le site web source"));

        // Résumé (TextArea)
        form.field(
            &TextField::textarea("summary")
                .label("Un court résumé...")
                .required(),
        );

        // Contenu (TextArea ou RichText si implémenté dans GenericField)
        form.field(
            &TextField::richtext("content")
                .label("Entrez le contenu de l'article ici")
                .required(),
        );
    }

    impl_form_access!();
}

impl BlogForm {
    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<crate::formulaire::blog::Model, DbErr> {
        let new_blog = crate::formulaire::blog::ActiveModel {
            title: Set(self.form.get_string("title")),
            email: Set(self.form.get_string("email")),
            website: Set(self.form.get_option("website")),
            summary: Set(self.form.get_string("summary")),
            content: Set(self.form.get_string("content")),
            ..Default::default()
        };

        new_blog.insert(db).await
    }
}
