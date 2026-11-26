use crate::migration::steps::generate::indent;

/// Runs `stmt` unless the database is MySQL and `column` holds a
/// foreign key there. InnoDB already indexes every foreign key column: an
/// explicit index would replace that one and then couldn't be dropped while
/// the key exists (error 1553, which broke `migrate down`). Checked at run time
/// so one file works on every engine.
pub(crate) fn unless_mysql_fk(table: &str, column: &str, stmt: &str) -> String {
    format!(
        r#"        {{
            let backs_fk = manager.get_connection().get_database_backend() == sea_orm::DbBackend::MySql
                && manager
                    .get_connection()
                    .query_one(
                        &Query::select()
                            .expr(Expr::val(1))
                            .from((Alias::new("information_schema"), Alias::new("KEY_COLUMN_USAGE")))
                            .and_where(Expr::col(Alias::new("TABLE_SCHEMA")).eq(Expr::cust("DATABASE()")))
                            .and_where(Expr::col(Alias::new("TABLE_NAME")).eq({table:?}))
                            .and_where(Expr::col(Alias::new("COLUMN_NAME")).eq({column:?}))
                            .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_not_null())
                            .to_owned(),
                    )
                    .await?
                    .is_some();
            if !backs_fk {{
{body}            }}
        }}
"#,
        body = indent(stmt, "        "),
    )
}
