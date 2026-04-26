//! Administration forms provided by Runique.
//!
//! These forms are available directly in `runique::admin::forms` and can
//! be referenced via `create_form:` in `admin!{}` without having to rewrite them.

use crate::forms::field::RuniqueForm;
use crate::forms::fields::{BooleanField, CheckboxField, ChoiceField, HiddenField, TextField};
use crate::forms::form::Forms;
use crate::impl_form_access;
use crate::utils::aliases::definition::StrMap;
use crate::utils::trad::t;

#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct DroitAdminForm {
    pub form: Forms,
}

impl RuniqueForm for DroitAdminForm {
    fn register_fields(form: &mut Forms) {
        // groupe_id, resource_key overridden dynamically in builtin.rs
        form.field(
            &ChoiceField::new("groupe_id")
                .label(t("admin.group").as_ref())
                .required(),
        );
        form.field(&CheckboxField::new("resource_key").label(t("admin.resources").as_ref()));
        form.field(&BooleanField::new("can_create").label(t("admin.can_create").as_ref()));
        form.field(&BooleanField::new("can_read").label(t("admin.can_read").as_ref()));
        form.field(&BooleanField::new("can_update").label(t("admin.can_update").as_ref()));
        form.field(&BooleanField::new("can_delete").label(t("admin.can_delete").as_ref()));
        form.field(&BooleanField::new("can_update_own").label(t("admin.can_update_own").as_ref()));
        form.field(&BooleanField::new("can_delete_own").label(t("admin.can_delete_own").as_ref()));
    }
    impl_form_access!();
}

// ─── GroupeAdminForm ─────────────────────────────────────────────────────────

#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct GroupeAdminForm {
    pub form: Forms,
}

impl RuniqueForm for GroupeAdminForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("nom")
                .label(t("admin.name").as_ref())
                .required(),
        );
    }
    impl_form_access!();
}

/// User creation form from the admin interface.
///
/// The `password` field is hidden — Runique automatically injects a random hash
/// and sends a reset email to the created user.
///
/// Usage in `admin!{}`:
/// ```rust,ignore
/// users: eihwaz_users::Model => MyForm {
///     title: "Users",
///     permissions: ["admin"],
///     create_form: runique::admin::forms::UserAdminCreateForm,
///     edit_form: crate::formulaire::UserEditForm,
/// }
/// ```
#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UserAdminCreateForm {
    pub form: Forms,
}

#[async_trait::async_trait]
impl RuniqueForm for UserAdminCreateForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label(t("admin.username").as_ref())
                .required(),
        );
        form.field(
            &TextField::email("email")
                .label(t("admin.email").as_ref())
                .required(),
        );
        form.field(&HiddenField::new("password"));
        form.field(&BooleanField::new("is_staff").label(t("admin.staff").as_ref()));
        form.field(&BooleanField::new("is_superuser").label(t("admin.superuser").as_ref()));
        // Field overridden dynamically with available groups (see builtin.rs)
        form.field(&CheckboxField::new("groupes").label(t("admin.groups").as_ref()));
    }

    impl_form_access!();

    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.cleaned_string("username").unwrap_or_default();
        let email = self.cleaned_string("email").unwrap_or_default();
        let mut errors = StrMap::new();

        if username.len() < 3 {
            errors.insert(
                "username".to_string(),
                t("admin.user.username_too_short").to_string(),
            );
        }
        if !email.contains('@') {
            errors.insert(
                "email".to_string(),
                t("admin.user.invalid_email").to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ─── UserAdminEditForm ────────────────────────────────────────────────────────

#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UserAdminEditForm {
    pub form: Forms,
}

impl RuniqueForm for UserAdminEditForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label(t("admin.username").as_ref())
                .required(),
        );
        form.field(
            &TextField::email("email")
                .label(t("admin.email").as_ref())
                .required(),
        );
        form.field(&BooleanField::new("is_active").label(t("admin.active").as_ref()));
        form.field(&BooleanField::new("is_staff").label(t("admin.staff").as_ref()));
        form.field(&BooleanField::new("is_superuser").label(t("admin.superuser").as_ref()));
        // Field overridden dynamically with available groups (see builtin.rs)
        form.field(&CheckboxField::new("groupes").label(t("admin.groups").as_ref()));
    }
    impl_form_access!();
}
