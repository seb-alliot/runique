// Tests pour RuniqueUser (trait + implémentations par défaut)

use crate::helpers::pk::pk;
use runique::auth::user::RuniqueUser;

// ═══════════════════════════════════════════════════════════════
// Mock minimal implémentant RuniqueUser
// ═══════════════════════════════════════════════════════════════

struct MockUser {
    id: runique::utils::pk::Pk,
    username: String,
    email: String,
    is_active: bool,
    is_superuser: bool,
}

impl RuniqueUser for MockUser {
    fn user_id(&self) -> runique::utils::pk::Pk {
        self.id
    }
    fn username(&self) -> &str {
        &self.username
    }
    fn email(&self) -> &str {
        &self.email
    }
    fn is_active(&self) -> bool {
        self.is_active
    }
    fn is_superuser(&self) -> bool {
        self.is_superuser
    }
}

fn mock_user(is_active: bool, is_superuser: bool) -> MockUser {
    MockUser {
        id: pk(1),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        is_active,
        is_superuser,
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests — getters RuniqueUser
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_user_getters() {
    let user = MockUser {
        id: pk(42),
        username: "bob".to_string(),
        email: "bob@test.com".to_string(),
        is_active: true,
        is_superuser: false,
    };
    assert_eq!(user.user_id(), pk(42));
    assert_eq!(user.username(), "bob");
    assert_eq!(user.email(), "bob@test.com");
    assert!(user.is_active());
    assert!(!user.is_superuser());
}

// ═══════════════════════════════════════════════════════════════
// Tests — can_sign_in() logique par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_can_sign_in_follows_is_active() {
    assert!(mock_user(true, false).can_sign_in());
    assert!(!mock_user(false, false).can_sign_in());
}

#[test]
fn test_can_sign_in_inactive_superuser_refused() {
    assert!(!mock_user(false, true).can_sign_in());
}
