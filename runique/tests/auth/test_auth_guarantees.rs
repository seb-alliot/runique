//! What authentication must guarantee, checked on its outcome.
//!
//! Written from cargo-mutants survivors (2026-10-02): each test fails if the
//! matching piece of code is altered — a password check loosened, a superuser
//! flag forced, a login that silently does nothing. The code was right; nothing
//! would have noticed if it stopped being.
use crate::helpers::{
    admin_server, db,
    pk::{pk, pk_sql_literal},
};
use runique::auth::session::{LoginError, login};
use runique::auth::{BuiltinUserEntity, authenticate_user};
use runique::db::ADb;
use runique::utils::config::Pk;
use runique::utils::constante::session_key::session::SESSION_USER_ID_KEY;
use runique::utils::password::{Manual, PasswordConfig, PasswordService};
use std::sync::Arc;
use tower_sessions::{MemoryStore, Session};

const PASSWORD: &str = "correct horse battery";

async fn users_db() -> ADb {
    let conn = db::fresh_db().await;
    db::exec(&conn, admin_server::USERS_DDL).await;
    db::exec(&conn, admin_server::SESSIONS_DDL).await;
    ADb::from_connection(conn)
}

async fn insert_user(db: &ADb, n: u32, username: &str, active: bool) {
    let hash = runique::utils::password::hash(PASSWORD).expect("hash");
    insert_user_hashed(db, n, username, active, &hash).await;
}

async fn insert_user_hashed(db: &ADb, n: u32, username: &str, active: bool, hash: &str) {
    runique::sea_orm::ConnectionTrait::execute_unprepared(
        db,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
             VALUES ({}, '{username}', '{username}@example.com', '{hash}', {}, 0, 0, '2026-01-01 00:00:00')",
            pk_sql_literal(n),
            active as i32,
        ),
    )
    .await
    .expect("insert user");
}

fn session() -> Session {
    Session::new(None, Arc::new(MemoryStore::default()), None)
}

// ── authenticate_user ────────────────────────────────────────────────────────

#[tokio::test]
async fn authenticate_user_needs_the_right_password_and_an_active_account() {
    let db = users_db().await;
    insert_user(&db, 1, "alice", true).await;
    insert_user(&db, 2, "bob", false).await;

    assert!(
        authenticate_user(&db, "alice", PASSWORD).await.is_some(),
        "right password, active"
    );
    assert!(
        authenticate_user(&db, "alice", "wrong").await.is_none(),
        "wrong password"
    );
    assert!(
        authenticate_user(&db, "bob", PASSWORD).await.is_none(),
        "inactive account"
    );
    assert!(
        authenticate_user(&db, "nobody", PASSWORD).await.is_none(),
        "unknown user"
    );
}

// ── rehash at sign-in ────────────────────────────────────────────────────────

fn bcrypt_hash() -> String {
    PasswordService::new(PasswordConfig::auto_with(Manual::Bcrypt))
        .hash(PASSWORD)
        .expect("bcrypt hash")
}

async fn stored_hash(db: &ADb, username: &str) -> String {
    BuiltinUserEntity::find_by_username(db, username)
        .await
        .expect("account")
        .password
}

#[tokio::test]
async fn a_sign_in_rewrites_a_hash_made_with_another_algorithm() {
    let db = users_db().await;
    insert_user_hashed(&db, 1, "alice", true, &bcrypt_hash()).await;

    let user = authenticate_user(&db, "alice", PASSWORD)
        .await
        .expect("signed in");
    let stored = stored_hash(&db, "alice").await;
    assert!(
        stored.starts_with("$argon2"),
        "rewritten with the configured algorithm: {stored}"
    );
    assert_eq!(
        user.password, stored,
        "the returned account carries the new hash"
    );
    assert!(
        authenticate_user(&db, "alice", PASSWORD).await.is_some(),
        "the same password still signs in"
    );
}

#[tokio::test]
async fn a_sign_in_leaves_a_current_hash_untouched() {
    let db = users_db().await;
    insert_user(&db, 1, "alice", true).await;
    let before = stored_hash(&db, "alice").await;

    authenticate_user(&db, "alice", PASSWORD)
        .await
        .expect("signed in");
    assert_eq!(stored_hash(&db, "alice").await, before);
}

#[tokio::test]
async fn a_failed_sign_in_never_rewrites_the_hash() {
    let db = users_db().await;
    let old = bcrypt_hash();
    insert_user_hashed(&db, 1, "alice", true, &old).await;
    insert_user_hashed(&db, 2, "bob", false, &bcrypt_hash()).await;
    let bob_before = stored_hash(&db, "bob").await;

    assert!(authenticate_user(&db, "alice", "wrong").await.is_none());
    assert_eq!(stored_hash(&db, "alice").await, old, "wrong password");
    assert!(authenticate_user(&db, "bob", PASSWORD).await.is_none());
    assert_eq!(
        stored_hash(&db, "bob").await,
        bob_before,
        "inactive account"
    );
}

// ── login() with an admin account ────────────────────────────────────

#[tokio::test]
async fn admin_login_stores_who_logged_in_never_their_rights() {
    let session = session();
    let staff = crate::helpers::user::test_user(pk(7), "staffer", true, false);
    login(&session, &staff, None, false).await.expect("login");

    assert_eq!(
        session.get::<Pk>(SESSION_USER_ID_KEY).await.unwrap(),
        Some(pk(7))
    );
    // The account (name, rights) is read from the database on every request:
    // a copy in the session would outlive a rename or a demotion.
    for key in ["username", "is_staff", "is_superuser"] {
        assert_eq!(session.get::<bool>(key).await.unwrap(), None, "{key}");
    }
    // The login request already carries the long authenticated lifetime.
    match session.expiry() {
        Some(tower_sessions::Expiry::OnInactivity(d)) => {
            assert!(d.whole_seconds() > 60, "authenticated session lasts {d}")
        }
        other => panic!("expected an inactivity expiry, got {other:?}"),
    }
}

#[tokio::test]
async fn superuser_login_stores_no_rights_either() {
    let session = session();
    let admin = crate::helpers::user::test_user(pk(8), "root", false, true);
    login(&session, &admin, None, false).await.expect("login");
    assert_eq!(
        session.get::<Pk>(SESSION_USER_ID_KEY).await.unwrap(),
        Some(pk(8))
    );
    assert_eq!(session.get::<String>("username").await.unwrap(), None);
    for key in ["is_staff", "is_superuser"] {
        assert_eq!(session.get::<bool>(key).await.unwrap(), None, "{key}");
    }
}

// ── login: the account must be allowed to sign in ──────────────────────────

/// An account waiting for its first activation (`is_active` off, no `activated_at`).
async fn insert_pending(db: &ADb, n: u32, username: &str) {
    runique::sea_orm::ConnectionTrait::execute_unprepared(
        db,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
             VALUES ({}, '{username}', '{username}@example.com', 'h', 0, 0, 0, NULL)",
            pk_sql_literal(n),
        ),
    )
    .await
    .expect("insert pending user");
}

async fn account(db: &ADb, n: u32) -> runique::auth::user::Model {
    BuiltinUserEntity::find_by_id(db, pk(n))
        .await
        .expect("account")
}

/// `login` checks the account itself, whatever path loaded it: an inactive or
/// never-activated account gets no session, and the caller is told so.
#[tokio::test]
async fn login_refuses_an_account_that_cannot_sign_in() {
    let db = users_db().await;
    insert_user(&db, 3, "carol", true).await;
    insert_user(&db, 4, "dave", false).await;
    insert_pending(&db, 5, "erin").await;

    let signed_in = session();
    login(&signed_in, &account(&db, 3).await, None, false)
        .await
        .expect("active and activated");
    assert_eq!(
        signed_in.get::<Pk>(SESSION_USER_ID_KEY).await.unwrap(),
        Some(pk(3))
    );

    for n in [4, 5] {
        let session = session();
        let refused = login(&session, &account(&db, n).await, None, false).await;
        assert!(
            matches!(refused, Err(LoginError::CannotSignIn)),
            "account {n}"
        );
        assert_eq!(session.get::<Pk>(SESSION_USER_ID_KEY).await.unwrap(), None);
    }
}

/// Activation returns the account as it now is, so it can sign in at once;
/// it happens once only — a second call, or an activated account, gets `None`.
#[tokio::test]
async fn activate_account_returns_the_account_ready_to_sign_in() {
    let db = users_db().await;
    insert_pending(&db, 6, "frank").await;
    insert_user(&db, 7, "gina", false).await; // activated before, deactivated since

    let activated = BuiltinUserEntity::activate_account(&db, pk(6))
        .await
        .unwrap()
        .expect("pending account activated");
    assert!(activated.is_active && activated.activated_at.is_some());
    let session = session();
    login(&session, &activated, None, false)
        .await
        .expect("signs in right away");

    assert!(
        BuiltinUserEntity::activate_account(&db, pk(6))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        BuiltinUserEntity::activate_account(&db, pk(7))
            .await
            .unwrap()
            .is_none(),
        "reactivation is the staff's"
    );
    assert!(!account(&db, 7).await.is_active);
}

// ── update_password_by_id ───────────────────────────────────────────────────

#[tokio::test]
async fn update_password_by_id_really_changes_the_password() {
    let db = users_db().await;
    insert_user(&db, 5, "erin", true).await;
    let new_hash = runique::utils::password::hash("brand new secret").expect("hash");
    BuiltinUserEntity::update_password_by_id(&db, pk(5), &new_hash)
        .await
        .expect("updated");
    assert!(
        authenticate_user(&db, "erin", "brand new secret")
            .await
            .is_some()
    );
    assert!(
        authenticate_user(&db, "erin", PASSWORD).await.is_none(),
        "old one gone"
    );
}

// ── Activation: once, by the owner; reactivation is the staff's ──────────────

async fn account_state(db: &ADb, n: u32) -> (bool, bool) {
    let user = BuiltinUserEntity::find_by_id(db, pk(n))
        .await
        .expect("account");
    (user.is_active, user.activated_at.is_some())
}

/// A password change never reactivates a blocked account — it used to, which
/// let a deactivated user unblock themselves through "forgot password".
#[tokio::test]
async fn a_new_password_never_reactivates_a_blocked_account() {
    let db = users_db().await;
    insert_user(&db, 6, "frank", false).await; // activated once, blocked since
    let hash = runique::utils::password::hash("another secret").expect("hash");
    BuiltinUserEntity::update_password_by_id(&db, pk(6), &hash)
        .await
        .expect("updated");
    BuiltinUserEntity::set_password_and_activate(&db, pk(6), &hash)
        .await
        .expect("updated");
    assert_eq!(account_state(&db, 6).await, (false, true), "still blocked");
    assert!(
        authenticate_user(&db, "frank", "another secret")
            .await
            .is_none()
    );
}

#[tokio::test]
async fn the_owners_first_password_activates_a_pending_account() {
    use runique::sea_orm::ConnectionTrait;
    let db = users_db().await;
    insert_user(&db, 7, "gina", false).await;
    db.execute_unprepared("UPDATE eihwaz_users SET activated_at = NULL WHERE username = 'gina'")
        .await
        .unwrap();
    assert!(
        authenticate_user(&db, "gina", PASSWORD).await.is_none(),
        "pending"
    );

    let hash = runique::utils::password::hash("gina's own").expect("hash");
    BuiltinUserEntity::set_password_and_activate(&db, pk(7), &hash)
        .await
        .expect("activated");
    assert_eq!(account_state(&db, 7).await, (true, true));
    assert!(authenticate_user(&db, "gina", "gina's own").await.is_some());
}

/// Active without having been activated can't sign in, whatever wrote it.
#[tokio::test]
async fn an_active_but_never_activated_account_cannot_sign_in() {
    use runique::sea_orm::ConnectionTrait;
    let db = users_db().await;
    insert_user(&db, 8, "hugo", true).await;
    db.execute_unprepared("UPDATE eihwaz_users SET activated_at = NULL WHERE username = 'hugo'")
        .await
        .unwrap();
    assert!(authenticate_user(&db, "hugo", PASSWORD).await.is_none());
}

// ── login_required ───────────────────────────────────────────────────────────

mod login_required {
    use crate::helpers::pk::pk;
    use axum::{Router, body::Body, http::Request, middleware::Next, routing::get};
    use runique::macros::routeur::router_ext::RouterExt;
    use runique::utils::constante::session_key::session::SESSION_USER_ID_KEY;
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

    fn app(logged_in: bool) -> Router {
        let protected = Router::new().login_required(
            "/private",
            "auth_guarantees_private",
            get(|| async { "secret page" }),
            "/login",
        );
        let app = if logged_in {
            protected.layer(axum::middleware::from_fn(
                |session: Session, req: Request<Body>, next: Next| async move {
                    session.insert(SESSION_USER_ID_KEY, pk(1)).await.unwrap();
                    next.run(req).await
                },
            ))
        } else {
            protected
        };
        app.layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false))
    }

    async fn visit(logged_in: bool) -> (u16, Option<String>, String) {
        let res = app(logged_in)
            .oneshot(Request::get("/private").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status().as_u16();
        let location = res
            .headers()
            .get("location")
            .map(|v| v.to_str().unwrap().to_string());
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (
            status,
            location,
            String::from_utf8_lossy(&body).into_owned(),
        )
    }

    #[tokio::test]
    async fn an_anonymous_visitor_is_sent_to_the_login_page() {
        let (status, location, body) = visit(false).await;
        assert!((300..400).contains(&status), "redirect, got {status}");
        assert_eq!(location.as_deref(), Some("/login"));
        assert!(!body.contains("secret page"));
    }

    #[tokio::test]
    async fn a_logged_in_user_gets_the_page() {
        let (status, _, body) = visit(true).await;
        assert_eq!(status, 200);
        assert_eq!(body, "secret page");
    }
}

// ── LoginGuard ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_periodic_cleanup_keeps_a_lockout_that_is_still_running() {
    let guard = runique::auth::guard::LoginGuard::new()
        .max_attempts(1)
        .lockout_secs(300);
    guard.record_failure("mallory");
    assert!(guard.is_locked("mallory"));
    guard.spawn_cleanup(tokio::time::Duration::from_millis(5));
    tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
    assert!(
        guard.is_locked("mallory"),
        "a 300 s lockout outlives a 5 ms cleanup tick"
    );
}

// ── Permissions ──────────────────────────────────────────────────────────────

#[test]
fn merging_permissions_adds_rights_never_removes_them() {
    use runique::auth::permissions::Permission;
    let mut mine = Permission {
        resource_key: "articles".into(),
        can_create: true,
        can_read: false,
        can_update: true,
        can_delete: false,
        can_update_own: true,
        can_delete_own: false,
    };
    let other = Permission {
        resource_key: "articles".into(),
        can_create: false,
        can_read: true,
        can_update: false,
        can_delete: true,
        can_update_own: false,
        can_delete_own: true,
    };
    mine.merge_from(&other);
    assert!(
        mine.can_create
            && mine.can_read
            && mine.can_update
            && mine.can_delete
            && mine.can_update_own
            && mine.can_delete_own,
        "a right granted by either group stays granted"
    );
}

#[tokio::test]
#[serial_test::serial]
async fn pruning_orphan_rights_reports_what_it_deleted() {
    use runique::auth::permissions::prune_orphan_droits;

    let conn = db::fresh_db().await;
    db::exec(&conn, admin_server::GROUPES_DROITS_DDL).await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key) VALUES \
         (1, 'kept'), (1, 'gone_a'), (2, 'gone_b')",
    )
    .await;

    let pruned = prune_orphan_droits(&conn, &["kept"]).await.expect("prune");
    assert_eq!(pruned, 2);
    db::assert_count(&conn, "eihwaz_groupes_droits", 1).await;
}
