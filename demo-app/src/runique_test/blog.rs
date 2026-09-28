use crate::backend::blog::{get_article, list_articles};
use crate::entities::blog::ActiveModel as BlogActiveModel;
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use sea_orm::DbErr;

const TITLE: &str = "runique_test rollback article";

/// Creates an article, then finds it through the blog search and by its id.
/// Once the run is over, it must not show up in the admin's blog list.
#[tokio::test]
async fn created_article_is_found_by_search_and_id() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let article = BlogActiveModel {
            title: Set(TITLE.to_string()),
            email: Set("runique_test@example.com".to_string()),
            summary: Set("Written by a runique test.".to_string()),
            content: Set("<p>Rolled back at the end of the test.</p>".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let found = list_articles(db, Some("runique_test rollback")).await;
        if !found.iter().any(|a| a.id == article.id) {
            return Err(DbErr::Custom(
                "the article should show up in the blog search".into(),
            ));
        }
        get_article(db, article.id)
            .await
            .map(|_| ())
            .ok_or_else(|| DbErr::Custom("the article should be found by its id".into()))
    })
    .await
}
