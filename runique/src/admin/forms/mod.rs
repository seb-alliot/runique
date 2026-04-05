//! Formulaires d'administration fournis par Runique.
//!
//! Ces forms sont disponibles directement dans `runique::admin::forms` et peuvent
//! être référencés via `create_form:` dans `admin!{}` sans avoir à les réécrire.

use crate::forms::field::RuniqueForm;
use crate::forms::fields::{BooleanField, HiddenField, TextField};
use crate::forms::form::Forms;
use crate::impl_form_access;
use crate::utils::aliases::definition::StrMap;

// ─── DroitAdminForm ──────────────────────────────────────────────────────────

#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct DroitAdminForm {
    pub form: Forms,
}

impl RuniqueForm for DroitAdminForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("nom").label("Nom").required());
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
        form.field(&TextField::text("nom").label("Nom").required());
    }
    impl_form_access!();
}

/// Formulaire de création d'utilisateur depuis l'interface admin.
///
/// Le champ `password` est caché — Runique injecte automatiquement un hash aléatoire
/// et envoie un email de reset à l'utilisateur créé.
///
/// Usage dans `admin!{}` :
/// ```rust,ignore
/// users: eihwaz_users::Model => MyForm {
///     title: "Utilisateurs",
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
                .label("Nom d'utilisateur")
                .required(),
        );
        form.field(&TextField::email("email").label("Email").required());
        form.field(&HiddenField::new("password"));
        form.field(&BooleanField::new("is_staff").label("Staff"));
        form.field(&BooleanField::new("is_superuser").label("Superuser"));
    }

    impl_form_access!();

    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.get_string("username");
        let email = self.get_string("email");
        let mut errors = StrMap::new();

        if username.len() < 3 {
            errors.insert(
                "username".to_string(),
                "Le nom d'utilisateur doit faire au moins 3 caractères.".to_string(),
            );
        }
        if !email.contains('@') {
            errors.insert("email".to_string(), "Adresse email invalide.".to_string());
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
                .label("Nom d'utilisateur")
                .required(),
        );
        form.field(&TextField::email("email").label("Email").required());
        form.field(&BooleanField::new("is_active").label("Actif"));
        form.field(&BooleanField::new("is_staff").label("Staff"));
        form.field(&BooleanField::new("is_superuser").label("Superuser"));
    }
    impl_form_access!();
}
