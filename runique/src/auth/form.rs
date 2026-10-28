//! `LoginAdmin` — admin login form with username/password fields.
use crate::forms::{
    Forms,
    field::{FormAccess, RuniqueForm},
    fields::text::TextField,
};
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

impl FormAccess for LoginAdmin {
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

impl RuniqueForm for LoginAdmin {}

/// An account as the staff edits it, from the `eihwaz_users` schema: its name
/// and email only. The password is its owner's, set from the email link;
/// ranks go through the gate.
#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UserModelForm {
    pub form: Forms,
}

impl crate::forms::model_form::ModelForm for UserModelForm {
    fn schema() -> crate::migration::schema::ModelSchema {
        crate::auth::user::schema()
    }
    fn fields() -> Option<&'static [&'static str]> {
        Some(&["username", "email"])
    }
}

impl FormAccess for UserModelForm {
    impl_form_access!(model);
}

impl RuniqueForm for UserModelForm {}

impl crate::forms::FormEntity for UserModelForm {
    type Entity = crate::auth::user::Entity;
}

/// A group as the staff edits it: its name, whether it opens the staff
/// summary, whether it is active. Its responsable is never a field: the gate
/// sets it to whoever creates the group, and changes it only by a transfer.
#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct GroupeModelForm {
    pub form: Forms,
}

impl crate::forms::model_form::ModelForm for GroupeModelForm {
    fn schema() -> crate::migration::schema::ModelSchema {
        crate::auth::permissions::groupe::schema()
    }
    fn fields() -> Option<&'static [&'static str]> {
        Some(&["nom", "is_staff", "is_active"])
    }
}

impl FormAccess for GroupeModelForm {
    impl_form_access!(model);
}

impl RuniqueForm for GroupeModelForm {}

impl crate::forms::FormEntity for GroupeModelForm {
    type Entity = crate::auth::permissions::groupe::Entity;
}
