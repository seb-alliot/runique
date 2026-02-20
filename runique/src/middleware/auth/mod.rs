pub mod admin_auth;
pub mod auth_session;
pub mod default_auth;
pub mod form;
pub mod user;
pub mod user_trait;

pub use auth_session::{
    get_user_id, get_username, has_permission, is_authenticated, load_user_middleware,
    login_required, login_user, login_user_full, logout, redirect_if_authenticated, CurrentUser,
};

pub use admin_auth::{AdminAuth, AdminLoginResult};
pub use user_trait::RuniqueUser;

pub use default_auth::{DefaultAdminAuth, UserEntity};

pub use form::LoginForm;
pub use user::{BuiltinUserEntity, RuniqueAdminAuth};
