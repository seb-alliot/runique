//! Minimal `RuniqueUser` implementor for tests exercising `login()`/session
//! helpers without needing a real DB-backed user model.

use runique::auth::user::RuniqueUser;
use runique::utils::config::Pk;

pub struct TestUser {
    id: Pk,
    username: String,
    is_staff: bool,
    is_superuser: bool,
}

/// Builds a `TestUser` — same field order as the old `login()` scalar arguments
/// (`id, username, is_staff, is_superuser`), so call sites only need to wrap
/// their existing arguments in `&test_user(...)`.
pub fn test_user(id: Pk, username: &str, is_staff: bool, is_superuser: bool) -> TestUser {
    TestUser {
        id,
        username: username.to_string(),
        is_staff,
        is_superuser,
    }
}

impl RuniqueUser for TestUser {
    fn user_id(&self) -> Pk {
        self.id
    }
    fn username(&self) -> &str {
        &self.username
    }
    fn email(&self) -> &str {
        ""
    }
    fn password_hash(&self) -> &str {
        ""
    }
    fn is_active(&self) -> bool {
        true
    }
    fn is_staff(&self) -> bool {
        self.is_staff
    }
    fn is_superuser(&self) -> bool {
        self.is_superuser
    }
}
