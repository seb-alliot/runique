//! Contributions with data that should go through and data the database must
//! turn down. Each test prints what it tried and what came back, so the
//! output reads like a log of the run.
use crate::backend::contribution::list_contributions;
use crate::entities::contribution::{
    ActiveModel as ContributionActiveModel, Column as ContributionColumn, ContributionType,
    Entity as ContributionEntity,
};
use runique::auth::current_user::CurrentUser;
use runique::auth::permissions::rang::Rang;
use runique::auth::permissions::{ColumnRight, Groupe, Permission};
use runique::prelude::runique_users::{ActiveModel as UserActiveModel, Entity as UserEntity};
use runique::prelude::*;
use runique::runique_test::{TestFailure, expect_db_error, runique_test};
use sea_orm::DbErr;

const TITLE: &str = "runique_test contribution";

#[derive(Debug, PartialEq, sea_orm::DerivePartialModel)]
#[sea_orm(entity = "ContributionEntity")]
struct ContributionTitle {
    title: String,
}

async fn new_user(db: &ADb, username: &str) -> Result<runique_users::Model, DbErr> {
    UserActiveModel {
        username: Set(username.to_string()),
        email: Set(format!("{username}@example.com")),
        password: Set("not-a-real-hash".to_string()),
        is_active: Set(false),
        is_dirigeant: Set(false),
        is_admin: Set(false),
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

#[tokio::test]
async fn gate_scope_and_search_ordering_compose() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let owner = new_user(db, "runique_test_gate_owner").await?;
        let other = new_user(db, "runique_test_gate_other").await?;

        contribution(owner.id, Some("runique_test_gate_alpha"))
            .insert(db)
            .await?;
        contribution(owner.id, Some("runique_test_gate_zulu"))
            .insert(db)
            .await?;
        contribution(other.id, Some("runique_test_gate_outside"))
            .insert(db)
            .await?;

        let mut permission = Permission::zeroed("contributions".to_string());
        permission.can_read_own = true;
        permission.colonnes.push(ColumnRight {
            colonne: "title".to_string(),
            can_read: true,
            can_create: false,
            can_update: false,
        });
        let user = CurrentUser {
            id: owner.id,
            username: owner.username,
            rang: Rang::Lambda,
            groupes: vec![Groupe {
                id: 1,
                nom: "runique_test_gate".to_string(),
                is_staff: true,
                permissions: vec![permission],
            }],
            droits: Vec::new(),
        };
        let proof = runique::auth::gate::check::authorize_list::<ContributionEntity>(
            &runique::auth::gate::GateCtx { db, user: &user },
        )
        .map_err(|e| DbErr::Custom(format!("gate refused the test user: {}", e.message())))?;

        let descending = search!(ContributionEntity => desc Title)
            .filter_by_gate(&proof)
            .all(db)
            .await?;
        let descending_titles = descending
            .iter()
            .map(|row| row.title.as_str())
            .collect::<Vec<_>>();
        if descending_titles != ["runique_test_gate_zulu", "runique_test_gate_alpha"] {
            return Err(DbErr::Custom(format!(
                "gate + descending search returned {descending_titles:?}"
            )));
        }

        let projected = search!(ContributionEntity => desc Title)
            .filter_by_gate(&proof)
            .into_select()
            .into_partial_model::<ContributionTitle>()
            .all(db)
            .await?;
        let projected_titles = projected
            .iter()
            .map(|row| row.title.as_str())
            .collect::<Vec<_>>();
        if projected_titles != ["runique_test_gate_zulu", "runique_test_gate_alpha"] {
            return Err(DbErr::Custom(format!(
                "gate + projected search returned {projected_titles:?}"
            )));
        }

        let ascending = search!(ContributionEntity)
            .filter_by_gate(&proof)
            .order_by_asc(ContributionColumn::Title)
            .all(db)
            .await?;
        let ascending_titles = ascending
            .iter()
            .map(|row| row.title.as_str())
            .collect::<Vec<_>>();
        if ascending_titles != ["runique_test_gate_alpha", "runique_test_gate_zulu"] {
            return Err(DbErr::Custom(format!(
                "gate + ascending search returned {ascending_titles:?}"
            )));
        }
        Ok(())
    })
    .await
}
