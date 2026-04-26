//! `LoginAdmin` — admin login form with username/password fields.
use crate::forms::{Forms, field::RuniqueForm, fields::text::TextField};
use crate::impl_form_access;

/// Admin login form provided by Runique.
///
/// Used automatically by `admin_login_post` in the admin router.
/// Developers don't need to touch this.
#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginAdmin {
    pub form: Forms,
}

impl RuniqueForm for LoginAdmin {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label(crate::utils::trad::t("admin.username").as_ref())
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label(crate::utils::trad::t("admin.password").as_ref())
                .required(),
        );
    }

    impl_form_access!();
}
