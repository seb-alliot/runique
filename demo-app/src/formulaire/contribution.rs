use crate::entities::contribution;
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

    // Replaces the schema-generated `contribution_type` field with a proper
    // `ChoiceField` (labeled options) instead of the raw text/enum widget
    // `to_form_field()` would derive. Static choices, no request dependency —
    // `customize()` already runs right after `register_fields()` on every
    // construction path (`impl_form_access!(model)`), so this is always in
    // place before validation, including the very first GET render.
    fn customize(form: &mut Forms) {
        form.field(
            &ChoiceField::new("contribution_type")
                .label("Contribution type")
                .choices(contribution_type_choices()),
        );
    }

    async fn clean(&mut self) -> Result<(), StrMap> {
        let title = self.cleaned_string("title").unwrap_or_default();
        let content = self.cleaned_string("content").unwrap_or_default();
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
        db: &ADb,
        user_pk: Pk,
    ) -> Result<crate::entities::contribution::Model, DbErr> {
        let db = db.as_ref();
        let new_contribution = crate::entities::contribution::ActiveModel {
            user_id: Set(user_pk),
            contribution_type: Set(self
                .cleaned_string("contribution_type")
                .unwrap_or_default()
                .parse::<crate::entities::contribution::ContributionType>()
                .unwrap_or_default()),
            title: Set(self.cleaned_string("title").unwrap_or_default()),
            content: Set(self.cleaned_string("content").unwrap_or_default()),
            ..Default::default()
        };
        new_contribution.insert(db).await
    }
}
