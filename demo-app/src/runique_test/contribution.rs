//! Contributions with data that should go through and data the database must
//! turn down. Each test prints what it tried and what came back, so the
//! output reads like a log of the run.
use crate::backend::contribution::list_contributions;
use crate::entities::contribution::{ActiveModel as ContributionActiveModel, ContributionType};
use runique::prelude::runique_users::{ActiveModel as UserActiveModel, Entity as UserEntity};
use runique::prelude::*;
use runique::runique_test::{TestFailure, expect_db_error, runique_test};
use sea_orm::DbErr;

const TITLE: &str = "runique_test contribution";

async fn new_user(db: &ADb, username: &str) -> Result<runique_users::Model, DbErr> {
    UserActiveModel {
        username: Set(username.to_string()),
        email: Set(format!("{username}@example.com")),
        password: Set("not-a-real-hash".to_string()),
        is_active: Set(false),
        is_staff: Set(false),
        is_superuser: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await
}

fn contribution(user_id: Pk, title: Option<&str>) -> ContributionActiveModel {
    ContributionActiveModel {
        user_id: Set(user_id),
        contribution_type: Set(ContributionType::Runique),
        title: title.map_or(NotSet, |title| Set(title.to_string())),
        content: Set("<p>Written by a runique test.</p>".to_string()),
        ..Default::default()
    }
}

async fn is_listed(db: &ADb, title: &str) -> bool {
    list_contributions(db)
        .await
        .iter()
        .any(|item| item.contribution.title == title)
}

#[tokio::test]
async fn contribution_from_a_real_user_is_listed() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let user = new_user(db, "runique_test_contributor").await?;
        eprintln!("    data: user #{} ({})", user.id, user.username);

        let saved = contribution(user.id, Some(TITLE)).insert(db).await?;
        eprintln!(
            "    ✓ saved: contribution #{} \"{}\"",
            saved.id, saved.title
        );

        let listed = list_contributions(db).await;
        let Some(item) = listed.iter().find(|item| item.contribution.id == saved.id) else {
            return Err(DbErr::Custom("the contribution isn't in the list".into()));
        };
        eprintln!(
            "    ✓ listed: \"{}\" by {:?}",
            item.contribution.title, item.username
        );
        if item.username.as_deref() != Some(user.username.as_str()) {
            return Err(DbErr::Custom(format!(
                "listed under {:?} instead of {}",
                item.username, user.username
            )));
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn contribution_from_a_deleted_user_is_rejected() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let user = new_user(db, "runique_test_gone").await?;
        UserEntity::delete_by_id(user.id).exec(db).await?;
        eprintln!("    data: user #{} created, then deleted", user.id);

        // `expect_db_error` runs it in a savepoint (on Postgres a failed
        // statement spoils the whole transaction) and tells the builder this
        // failure is wanted, so it isn't taken for a swallowed error.
        let refused = expect_db_error(db, async |sp| {
            contribution(user.id, Some(TITLE)).insert(sp).await
        })
        .await?;
        eprintln!("    ✓ refused as expected: {refused}");

        if is_listed(db, TITLE).await {
            return Err(DbErr::Custom("the refused contribution got listed".into()));
        }
        eprintln!("    ✓ not listed");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn contribution_without_a_title_is_rejected() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let user = new_user(db, "runique_test_untitled").await?;
        eprintln!("    data: user #{}, contribution with no title", user.id);

        let refused =
            expect_db_error(db, async |sp| contribution(user.id, None).insert(sp).await).await?;
        eprintln!("    ✓ refused as expected: {refused}");
        Ok(())
    })
    .await
}
