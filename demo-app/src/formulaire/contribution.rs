use crate::entities::contribution::schema as contribution;
use runique::prelude::*;

pub fn contribution_type_choices() -> Vec<ChoiceOption> {
    vec![
        ChoiceOption::new("runique", "Contribution au framework Runique"),
        ChoiceOption::new("cours", "Proposition de cours"),
    ]
}

#[form(schema = contribution, fields = [contribution_type, title, content])]
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
        user_pk: UserId,
    ) -> Result<crate::entities::contribution::Model, DbErr> {
        let new_contribution = crate::entities::contribution::ActiveModel {
            user_id: Set(user_pk.try_into().unwrap()),
            contribution_type: Set(self
                .form
                .get_string("contribution_type")
                .parse::<crate::entities::contribution::ContributionType>()
                .unwrap_or_default()),
            title: Set(self.form.get_string("title")),
            content: Set(self.form.get_string("content")),
            ..Default::default()
        };
        new_contribution.insert(db).await
    }
}
