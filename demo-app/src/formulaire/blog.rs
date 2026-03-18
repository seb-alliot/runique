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

        if title.len() < 5 {
            errors.insert(
                "title".to_string(),
                "Title must be at least 5 characters long".to_string(),
            );
        }
        if let Some(ref w) = website
            && !w.starts_with("http")
        {
            errors.insert(
                "website".to_string(),
                "Website must start with http".to_string(),
            );
        }
        if !email.contains('@') {
            errors.insert("email".to_string(), "Invalid email address".to_string());
        }
        if summary.len() < 10 {
            errors.insert(
                "summary".to_string(),
                "Summary must be at least 10 characters long".to_string(),
            );
        }
        if content.len() < 20 {
            errors.insert(
                "content".to_string(),
                "Content must be at least 20 characters long".to_string(),
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
        &self,
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
        new_blog.insert(db).await
    }
}
