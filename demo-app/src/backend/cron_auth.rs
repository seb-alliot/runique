use runique::prelude::user::{Column, Entity as UserEntity};
use runique::prelude::*;

/// Accounts whose activation link was never used. A blocked account always has
/// `activated_at` set (CHECK on eihwaz_users), so it is never touched.
pub fn delete_accounts_inactive(db: DatabaseConnection) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            tick.tick().await;
            let cutoff = chrono::Utc::now().naive_utc() - chrono::Duration::days(7);
            match UserEntity::delete_many()
                .filter(Column::IsActive.eq(false))
                .filter(Column::ActivatedAt.is_null())
                .filter(Column::CreatedAt.lt(cutoff))
                .exec(&db)
                .await
            {
                Ok(r) if r.rows_affected > 0 => {
                    tracing::info!(
                        n = r.rows_affected,
                        "purge: never activated accounts deleted"
                    )
                }
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "purge of never activated accounts failed"),
            }
        }
    });
}
