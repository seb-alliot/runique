pub mod admin_auth;
pub mod auth_session;
pub mod default_auth;
pub mod form;
pub mod login_guard;
pub mod reset;
pub mod user;
pub mod user_trait;

pub use auth_session::{
    CurrentUser, get_user_id, get_username, has_permission, is_admin_authenticated,
    is_authenticated, load_user_middleware, login, login_required, login_staff, logout,
    redirect_if_authenticated,
};

pub use admin_auth::{AdminAuth, AdminLoginResult};
pub use user_trait::RuniqueUser;

pub use default_auth::{DefaultAdminAuth, UserEntity};
pub use login_guard::LoginGuard;
pub use reset::{
    ForgotPasswordForm, PasswordResetAdapter, PasswordResetConfig, PasswordResetForm,
    PasswordResetHandler, PasswordResetStaging, handle_forgot_password, handle_password_reset,
};

pub use form::LoginAdmin;
pub use user::{BuiltinUserEntity, RuniqueAdminAuth};
