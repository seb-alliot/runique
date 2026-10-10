//! `text_eq_ci`: the lookup behind `bulk_create`'s duplicate check — `Gluten`
//! and `gluten` are the same value, `Lait` is not.
use crate::helpers::db::{exec, fresh_db_with_schema};
use runique::admin::helper::text_eq_ci;
use runique::sea_orm::ConnectionTrait;
use runique::sea_orm::sea_query::{Alias, Asterisk, Expr, ExprTrait, Query};
use runique::utils::aliases::ADb;

async fn count(db: &ADb, value: &str) -> i64 {
    let query = Query::select()
        .expr(Expr::col(Asterisk).count())
        .from(Alias::new("allergenes"))
        .and_where(text_eq_ci(db, "nom", value))
        .to_owned();
    let stmt = db.get_database_backend().build(&query);
    db.query_one_raw(stmt)
        .await
        .unwrap()
        .unwrap()
        .try_get_by_index::<i64>(0)
        .unwrap()
}

#[tokio::test]
async fn a_value_matches_whatever_its_case() {
    let conn =
        fresh_db_with_schema("CREATE TABLE allergenes (id INTEGER PRIMARY KEY, nom TEXT NOT NULL)")
            .await;
    exec(&conn, "INSERT INTO allergenes (nom) VALUES ('Gluten')").await;
    let db = ADb::from_connection(conn);

    for spelling in ["Gluten", "gluten", "GLUTEN", "gLuTeN"] {
        assert_eq!(count(&db, spelling).await, 1, "{spelling}");
    }
    // The other side: another value, or a prefix, isn't a match.
    assert_eq!(count(&db, "Lait").await, 0);
    assert_eq!(count(&db, "glu").await, 0);
}
