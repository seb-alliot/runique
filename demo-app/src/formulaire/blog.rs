use crate::entities::blog::schema as blog;
use runique::prelude::*;

#[form(schema = blog, fields = [title, email, summary, website, content])]
pub struct BlogForm;

#[async_trait]
impl RuniqueForm for BlogForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let title = self.get_string("title");
        let email = self.get_string("email");
        let website = self.get_option("website");
        let summary = self.get_string("summary");
        let content = self.get_string("content");
        let mut errors = StrMap::new();

        if title.len() < 3 {
            errors.insert(
                "title".to_string(),
                "Le titre doit faire au moins 3 caractères.".to_string(),
            );
        }
        if let Some(ref w) = website
            && !w.starts_with("http")
        {
            errors.insert(
                "website".to_string(),
                "Le site web doit commencer par http ou https.".to_string(),
            );
        }
        if !email.contains('@') {
            errors.insert("email".to_string(), "Adresse email invalide.".to_string());
        }
        if summary.len() < 5 {
            errors.insert(
                "summary".to_string(),
                "Le résumé doit faire au moins 5 caractères.".to_string(),
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

#[allow(dead_code)]
impl BlogForm {
    pub async fn save(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<crate::entities::blog::Model, DbErr> {
        let new_blog = crate::entities::blog::ActiveModel {
            title: Set(self.form.get_string("title")),
            email: Set(self.form.get_string("email")),
            website: Set(self.form.get_option("website")),
            summary: Set(self.form.get_string("summary")),
            content: Set(self.form.get_string("content")),
            ..Default::default()
        };
        let new_blog = new_blog.insert(db).await;
        if new_blog.is_ok() {
            self.clear();
        }
        new_blog
    }
}
