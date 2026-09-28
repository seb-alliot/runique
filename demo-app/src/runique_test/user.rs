use crate::backend::auth::find_user_by_username;
use runique::prelude::runique_users::ActiveModel as UserActiveModel;
use runique::prelude::*;
use runique::runique_test::runique_test;
use sea_orm::DbErr;

const USERNAME: &str = "runique_test_rollback";

/// Creates a user and finds it again through the app's own lookup. Once the
/// run is over, this user must not exist in the database: check the admin.
#[tokio::test]
async fn created_user_is_visible_inside_the_test() {
    runique_test::<ADb>(super::ENV, async |db| {
        UserActiveModel {
            username: Set(USERNAME.to_string()),
            email: Set("runique_test_rollback@example.com".to_string()),
            password: Set("not-a-real-hash".to_string()),
            is_active: Set(false),
            is_staff: Set(false),
            is_superuser: Set(false),
            ..Default::default()
        }
        .insert(db)
        .await?;

        find_user_by_username(db, USERNAME)
            .await
            .map(|_| ())
            .ok_or_else(|| {
                DbErr::Custom("the user should be visible in its own transaction".into())
            })
    })
    .await;
}

#[tokio::test]
async fn unknown_username_is_not_found() {
    runique_test::<ADb>(super::ENV, async |db| {
        match find_user_by_username(db, "nobody_has_this_name").await {
            None => Ok(()),
            Some(_) => Err(DbErr::Custom("an unknown username was found".into())),
        }
    })
    .await;
}
