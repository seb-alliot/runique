//! Authentication — session, guards, permissions, password reset.
pub mod current_user;
pub mod form;
pub mod guard;
pub mod migrations_table;
pub mod password;
pub mod permissions;
pub mod session;
pub mod user;
pub mod user_trait;

pub use current_user::CurrentUser;
pub use form::LoginAdmin;
pub use guard::LoginGuard;
pub use password::{
    ExtraContextFn, ForgotPasswordForm, PasswordResetConfig, PasswordResetForm,
    PasswordResetStaging, handle_forgot_password, handle_password_reset,
};
pub use permissions::rang::Rang;
pub use permissions::{Groupe, Permission, pull_groupes_db};
pub use session::{
    LoginError, is_authenticated, login, logout, protect_session, unprotect_session,
};
pub use user::{BuiltinUserEntity, authenticate_admin, authenticate_user};
