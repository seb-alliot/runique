// Tests pour RuniqueUser (trait + implémentations par défaut)
// et DefaultAdminAuth (construction uniquement, sans DB)

use runique::auth::{
    session::{DefaultAdminAuth, UserEntity},
    user::RuniqueUser,
};
use sea_orm::DatabaseConnection;

// ═══════════════════════════════════════════════════════════════
// Mock minimal implémentant RuniqueUser
// ═══════════════════════════════════════════════════════════════

struct MockUser {
    id: runique::utils::pk::Pk,
    username: String,
    email: String,
    password: String,
    is_active: bool,
    is_staff: bool,
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
    fn password_hash(&self) -> &str {
        &self.password
    }
    fn is_active(&self) -> bool {
        self.is_active
    }
    fn is_staff(&self) -> bool {
        self.is_staff
    }
    fn is_superuser(&self) -> bool {
        self.is_superuser
    }
}

fn mock_user(is_active: bool, is_staff: bool, is_superuser: bool) -> MockUser {
    MockUser {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "hash".to_string(),
        is_active,
        is_staff,
        is_superuser,
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests — getters RuniqueUser
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_user_getters() {
    let user = MockUser {
        id: 42,
        username: "bob".to_string(),
        email: "bob@test.com".to_string(),
        password: "argon2hash".to_string(),
        is_active: true,
        is_staff: false,
        is_superuser: false,
    };
    assert_eq!(user.user_id(), 42);
    assert_eq!(user.username(), "bob");
    assert_eq!(user.email(), "bob@test.com");
    assert_eq!(user.password_hash(), "argon2hash");
    assert!(user.is_active());
    assert!(!user.is_staff());
    assert!(!user.is_superuser());
}

// ═══════════════════════════════════════════════════════════════
// Tests — roles() par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_roles_default_retourne_vec_vide() {
    let user = mock_user(true, true, false);
    assert!(user.roles().is_empty());
}

// ═══════════════════════════════════════════════════════════════
// Tests — can_access_admin() logique par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_can_access_admin_actif_et_staff() {
    assert!(mock_user(true, true, false).can_access_admin());
}

#[test]
fn test_can_access_admin_actif_et_superuser() {
    assert!(mock_user(true, false, true).can_access_admin());
}

#[test]
fn test_can_access_admin_actif_staff_et_superuser() {
    assert!(mock_user(true, true, true).can_access_admin());
}

#[test]
fn test_can_access_admin_inactif_refuse() {
    // Compte inactif → jamais admin, même si staff+superuser
    assert!(!mock_user(false, true, true).can_access_admin());
}

#[test]
fn test_can_access_admin_inactif_staff_refuse() {
    assert!(!mock_user(false, true, false).can_access_admin());
}

#[test]
fn test_can_access_admin_inactif_superuser_refuse() {
    assert!(!mock_user(false, false, true).can_access_admin());
}

#[test]
fn test_can_access_admin_actif_sans_droits_refuse() {
    assert!(!mock_user(true, false, false).can_access_admin());
}

// ═══════════════════════════════════════════════════════════════
// Mock UserEntity (sans appels DB réels)
// ═══════════════════════════════════════════════════════════════

struct MockUserEntity;

#[async_trait::async_trait]
impl UserEntity for MockUserEntity {
    type Model = MockUser;

    async fn find_by_username(_db: &DatabaseConnection, _username: &str) -> Option<Self::Model> {
        None
    }

    async fn find_by_email(_db: &DatabaseConnection, _email: &str) -> Option<Self::Model> {
        None
    }

    async fn find_by_id(
        _db: &DatabaseConnection,
        _id: runique::utils::pk::Pk,
    ) -> Option<Self::Model> {
        None
    }

    async fn update_password(
        _db: &DatabaseConnection,
        _email: &str,
        _new_hash: &str,
    ) -> Result<(), sea_orm::DbErr> {
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests — DefaultAdminAuth construction
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_default_admin_auth_new() {
    let _auth = DefaultAdminAuth::<MockUserEntity>::new();
}

#[test]
fn test_default_admin_auth_default() {
    let _auth = DefaultAdminAuth::<MockUserEntity>::default();
}
