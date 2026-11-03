use crate::backend::auth::find_user_by_username;
use crate::formulaire::RegisterForm;
use runique::prelude::runique_users::ActiveModel as UserActiveModel;
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use sea_orm::DbErr;

const USERNAME: &str = "runique_test_rollback";

/// Creates a user and finds it again through the app's own lookup. Once the
/// run is over, this user must not exist in the database: check the admin.
#[tokio::test]
async fn created_user_is_visible_inside_the_test() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        UserActiveModel {
            username: Set(USERNAME.to_string()),
            email: Set("runique_test_rollback@example.com".to_string()),
            password: Set("not-a-real-hash".to_string()),
            is_active: Set(false),
            is_dirigeant: Set(false),
            is_admin: Set(false),
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
    .await
}

#[tokio::test]
async fn unknown_username_is_not_found() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        match find_user_by_username(db, "nobody_has_this_name").await {
            None => Ok(()),
            Some(_) => Err(DbErr::Custom("an unknown username was found".into())),
        }
    })
    .await
}

const SIGNUP: &str = "runique_test_signup";
const TYPED: &str = "Runique-Test-2026!";

/// The generic save, outside the admin, writes every field of the form, the
/// password among them: stored as its hash, never as the text typed in.
/// `-- --nocapture` shows the queries it ran.
#[tokio::test]
async fn the_generic_save_writes_the_password_as_its_hash() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let req = super::form_save::named_request(
            db,
            "register",
            &[],
            &[
                ("username", SIGNUP),
                ("email", "runique_test_signup@example.com"),
                ("password", TYPED),
            ],
        );
        let validated = ValidationForm::try_form(req.form::<RegisterForm>(), &req, "register")
            .await
            .map_err(|form| {
                DbErr::Custom(format!(
                    "the form should be valid: {:?}",
                    form.forms().errors()
                ))
            })?;
        if validated.save(db).await.is_err() {
            return Err(DbErr::Custom(
                "the generic save should create the account".into(),
            ));
        }

        let account = find_user_by_username(db, SIGNUP)
            .await
            .ok_or_else(|| DbErr::Custom("the account should exist".into()))?;
        if account.password == TYPED {
            return Err(DbErr::Custom("the password was stored as typed".into()));
        }
        if !runique::utils::password::verify(TYPED, &account.password) {
            return Err(DbErr::Custom(format!(
                "the stored value isn't the typed password's hash: {}",
                account.password
            )));
        }
        Ok(())
    })
    .await
}
