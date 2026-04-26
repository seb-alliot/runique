//! Authentication — session, guards, permissions, password reset.
pub mod form;
pub mod guard;
pub mod password;
pub mod session;
pub mod user;
pub mod user_trait;

pub use form::LoginAdmin;
pub use guard::LoginGuard;
pub use password::{
    ForgotPasswordForm, PasswordResetAdapter, PasswordResetConfig, PasswordResetForm,
    PasswordResetHandler, PasswordResetStaging, handle_forgot_password, handle_password_reset,
};
pub use session::{
    AdminAuth, AdminLoginResult, CurrentUser, DefaultAdminAuth, UserEntity, auth_login,
    get_user_id, get_username, has_permission, is_admin_authenticated, is_authenticated,
    load_user_middleware, login, logout, protect_session, unprotect_session,
};
pub use user::{BuiltinUserEntity, RuniqueAdminAuth};
