use crate::forms::{field::RuniqueForm, fields::text::TextField, Forms};
use crate::impl_form_access;

/// Formulaire de connexion admin fourni par Runique.
///
/// Utilisé automatiquement par `admin_login_post` dans le router admin.
/// Le dev n'a pas besoin d'y toucher.
#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl RuniqueForm for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Nom d'utilisateur")
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label("Mot de passe")
                .required(),
        );
    }

    impl_form_access!();
}
