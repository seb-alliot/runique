use crate::entities::blog;
use runique::prelude::*;

#[form(schema = blog, fields = [title, email, summary, website, content])]
pub struct BlogForm;

#[async_trait]
impl RuniqueForm for BlogForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let title = self.cleaned::<String>("title").unwrap_or_default();
        let email = self.cleaned::<String>("email").unwrap_or_default();
        let website = self.cleaned::<String>("website");
        let summary = self.cleaned::<String>("summary").unwrap_or_default();
        let content = self.cleaned::<String>("content").unwrap_or_default();
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
    pub async fn save(&mut self, db: &ADb) -> Result<crate::entities::blog::Model, DbErr> {
        let new_blog = crate::entities::blog::ActiveModel {
            title: Set(self.cleaned::<String>("title").unwrap_or_default()),
            email: Set(self.cleaned::<String>("email").unwrap_or_default()),
            website: Set(self.cleaned::<String>("website")),
            summary: Set(self.cleaned::<String>("summary").unwrap_or_default()),
            content: Set(self.cleaned::<String>("content").unwrap_or_default()),
            ..Default::default()
        };
        let new_blog = new_blog.insert(db).await;
        if new_blog.is_ok() {
            self.clear();
        }
        new_blog
    }
}
