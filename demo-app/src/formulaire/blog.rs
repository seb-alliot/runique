use crate::entities::blog::schema as blog;
use runique::prelude::*;

#[form(schema = blog, fields = [title, email, summary, website, content])]
pub struct BlogForm;

impl BlogForm {
    async fn clean_fields(&self) -> Result<(), String> {
        let title = self.form.get_string("title");
        let email = self.form.get_string("email");
        let website = self.form.get_option("website");
        let summary = self.form.get_string("summary");
        let content = self.form.get_string("content");

        if title.len() < 5 {
            return Err("Title must be at least 5 characters long".to_string());
        }
        if let Some(website) = website {
            if !website.starts_with("http") {
                return Err("Website must start with http".to_string());
            }
        }
        if !email.contains('@') {
            return Err("Invalid email address".to_string());
        }
        if summary.len() < 10 {
            return Err("Summary must be at least 10 characters long".to_string());
        }
        if content.len() < 20 {
            return Err("Content must be at least 20 characters long".to_string());
        }
        Ok(())
    }

    async fn clean(&self) -> Result<(), String> {
        self.clean_fields().await
    }

    pub async fn save(
        &self,
        db: &DatabaseConnection,
    ) -> Result<crate::entities::blog::Model, DbErr> {
        if let Err(e) = self.clean().await {
            return Err(DbErr::Custom(e));
        }
        let new_blog: crate::entities::blog::ActiveModel = crate::entities::blog::ActiveModel {
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
