//! Administration forms provided by Runique.
//!
//! These forms are available directly in `runique::admin::forms` and can
//! be referenced via `create_form:` in `admin!{}` without having to rewrite them.

use crate::forms::field::RuniqueForm;
use crate::forms::fields::{BooleanField, CheckboxField, ChoiceField, HiddenField, TextField};
use crate::forms::form::Forms;
use crate::impl_form_access;
use crate::utils::aliases::definition::StrMap;

#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct DroitAdminForm {
    pub form: Forms,
}

impl RuniqueForm for DroitAdminForm {
    fn register_fields(form: &mut Forms) {
        // groupe_id, resource_key overridden dynamically in builtin.rs
        form.field(&ChoiceField::new("groupe_id").label("Group").required());
        form.field(&CheckboxField::new("resource_key").label("Targeted Resources"));
        form.field(&BooleanField::new("can_create").label("Can create"));
        form.field(&BooleanField::new("can_read").label("Can read"));
        form.field(&BooleanField::new("can_update").label("Can update"));
        form.field(&BooleanField::new("can_delete").label("Can delete"));
        form.field(&BooleanField::new("can_update_own").label("Can update own data"));
        form.field(&BooleanField::new("can_delete_own").label("Can delete own data"));
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
        form.field(&TextField::text("nom").label("Name").required());
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
        form.field(&TextField::text("username").label("Username").required());
        form.field(&TextField::email("email").label("Email").required());
        form.field(&HiddenField::new("password"));
        form.field(&BooleanField::new("is_staff").label("Staff"));
        form.field(&BooleanField::new("is_superuser").label("Superuser"));
        // Field overridden dynamically with available groups (see builtin.rs)
        form.field(&CheckboxField::new("groupes").label("Groups"));
    }

    impl_form_access!();

    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.cleaned_string("username").unwrap_or_default();
        let email = self.cleaned_string("email").unwrap_or_default();
        let mut errors = StrMap::new();

        if username.len() < 3 {
            errors.insert(
                "username".to_string(),
                "Username must be at least 3 characters long.".to_string(),
            );
        }
        if !email.contains('@') {
            errors.insert("email".to_string(), "Invalid email address.".to_string());
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
        form.field(&TextField::text("username").label("Username").required());
        form.field(&TextField::email("email").label("Email").required());
        form.field(&BooleanField::new("is_active").label("Active"));
        form.field(&BooleanField::new("is_staff").label("Staff"));
        form.field(&BooleanField::new("is_superuser").label("Superuser"));
        // Field overridden dynamically with available groups (see builtin.rs)
        form.field(&CheckboxField::new("groupes").label("Groups"));
    }
    impl_form_access!();
}
