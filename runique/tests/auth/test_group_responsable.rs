//! What a group's responsable does to its rights, as `pull_groupes_db` loads
//! them: a dirigeant leaves them whole, an admin caps them to his own grid,
//! and an inactive or lambda responsable, or a deactivated group, gives
//! nothing at all.
use crate::helpers::{
    db, framework_tables,
    pk::{pk, pk_sql_literal},
};
use runique::auth::permissions::{Groupe, pull_groupes_db};
use runique::sea_orm::DatabaseConnection;

/// Account 1 in group 1, answered for by account 2 whose flags are
/// `(is_active, is_dirigeant, is_admin)`. The group grants every right on
/// `articles` and reading `commandes`.
async fn setup(responsable: (u8, u8, u8), group_active: bool) -> DatabaseConnection {
    let conn = db::fresh_db().await;
    for ddl in [
        framework_tables::USERS_DDL,
        framework_tables::GROUPES_DDL,
        framework_tables::GROUPES_DROITS_DDL,
        framework_tables::USERS_GROUPES_DDL,
    ] {
        db::exec(&conn, ddl).await;
    }
    let (member, resp) = (pk_sql_literal(1), pk_sql_literal(2));
    let (active, dirigeant, admin) = responsable;
    db::exec(
        &conn,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_dirigeant, is_admin) VALUES \
             ({member}, 'membre', 'membre@example.com', 'x', 1, 0, 0), \
             ({resp}, 'resp', 'resp@example.com', 'x', {active}, {dirigeant}, {admin})"
        ),
    )
    .await;
    db::exec(
        &conn,
        &format!(
            "INSERT INTO eihwaz_groupes (id, nom, is_active, responsable_id) \
             VALUES (1, 'redacteurs', {}, {resp})",
            u8::from(group_active)
        ),
    )
    .await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_create, can_read, \
         can_read_own, can_update, can_update_own, can_delete, can_delete_own) VALUES \
         (1, 'articles', 1, 1, 1, 1, 1, 1, 1), (1, 'commandes', 0, 1, 0, 0, 0, 0, 0)",
    )
    .await;
    db::exec(
        &conn,
        &format!("INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES ({member}, 1)"),
    )
    .await;
    conn
}

async fn groups(conn: &DatabaseConnection) -> Vec<Groupe> {
    pull_groupes_db(conn, pk(1)).await
}

#[tokio::test]
async fn a_dirigeant_leaves_his_group_whole() {
    let conn = setup((1, 1, 0), true).await;
    let groupes = groups(&conn).await;
    assert_eq!(groupes.len(), 1);
    let articles = &groupes[0].permissions[0];
    assert!(articles.can_create && articles.can_update && articles.can_delete);
    assert_eq!(groupes[0].permissions.len(), 2, "commandes kept too");
}

#[tokio::test]
async fn an_admin_caps_his_group_to_his_grid() {
    let conn = setup((1, 0, 1), true).await;
    db::exec(
        &conn,
        &format!(
            "INSERT INTO eihwaz_groupes_droits (user_id, resource_key, can_read, can_update_own) \
             VALUES ({}, 'articles', 1, 1)",
            pk_sql_literal(2)
        ),
    )
    .await;
    let groupes = groups(&conn).await;
    assert_eq!(groupes.len(), 1);
    assert_eq!(
        groupes[0].permissions.len(),
        1,
        "commandes: not in his grid"
    );
    let articles = &groupes[0].permissions[0];
    assert!(articles.can_read && articles.can_read_own && articles.can_update_own);
    assert!(!articles.can_update, "never turned into more than he has");
    assert!(!articles.can_create && !articles.can_delete && !articles.can_delete_own);
}

#[tokio::test]
async fn an_inactive_responsable_gives_nothing() {
    let conn = setup((0, 1, 0), true).await;
    assert!(groups(&conn).await.is_empty());
}

#[tokio::test]
async fn a_lambda_responsable_gives_nothing() {
    let conn = setup((1, 0, 0), true).await;
    assert!(groups(&conn).await.is_empty());
}

#[tokio::test]
async fn a_deactivated_group_gives_nothing() {
    let conn = setup((1, 1, 0), false).await;
    assert!(groups(&conn).await.is_empty());
}
