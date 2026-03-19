use crate::entities::contribution::schema as contribution;
use runique::prelude::*;

#[form(schema = contribution, fields = [title, content])]
pub struct ContributionForm;

#[async_trait]
impl RuniqueForm for ContributionForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let title = self.get_string("title");
        let content = self.get_string("content");
        let mut errors = StrMap::new();

        if title.len() < 3 {
            errors.insert(
                "title".to_string(),
                "Le titre doit faire au moins 3 caractères.".to_string(),
            );
        }
        if content.len() < 10 {
            errors.insert(
                "content".to_string(),
                "Le contenu doit faire au moins 10 caractères.".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ContributionForm {
    pub async fn save(
        &mut self,
        db: &DatabaseConnection,
        user_id: i32,
    ) -> Result<crate::entities::contribution::Model, DbErr> {
        let new_contribution = crate::entities::contribution::ActiveModel {
            user_id: Set(user_id),
            title: Set(self.form.get_string("title")),
            content: Set(self.form.get_string("content")),
            ..Default::default()
        };
        new_contribution.insert(db).await
    }
}
