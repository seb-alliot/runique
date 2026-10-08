//! Authentication — session, guards, permissions, password reset.
pub mod form;
pub mod guard;
pub mod password;
pub mod permissions;
pub mod session;
pub mod user;
pub mod user_trait;

pub use form::LoginAdmin;
pub use guard::LoginGuard;
pub use password::{
    ExtraContextFn, ForgotPasswordForm, PasswordResetConfig, PasswordResetForm,
    PasswordResetStaging, handle_forgot_password, handle_password_reset,
};
pub use permissions::{Groupe, Permission, pull_groupes_db};
pub use session::{
    CurrentUser, LoginError, is_authenticated, login, logout, protect_session, unprotect_session,
};
pub use user::{BuiltinUserEntity, authenticate_admin, authenticate_user};
