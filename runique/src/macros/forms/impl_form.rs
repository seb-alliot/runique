/// Macro pour implémenter le boilerplate `from_form`, `get_form`, `get_form_mut`
/// du trait `RuniqueForm`.
///
/// # Utilisation
///
/// ```rust,ignore
/// impl RuniqueForm for RegisterForm {
///     fn register_fields(form: &mut Forms) {
///         form.field(&TextField::text("username").required());
///     }
///     impl_form_access!();          // si le champ s'appelle `form`
///     // impl_form_access!(formulaire); // si le champ s'appelle autrement
/// }
/// ```
#[macro_export]
macro_rules! impl_form_access {
    // Variante par défaut : le champ s'appelle `form`
    () => {
        fn from_form(form: ::runique::forms::manager::Forms) -> Self {
            Self { form }
        }
        fn get_form(&self) -> &::runique::forms::manager::Forms {
            &self.form
        }
        fn get_form_mut(&mut self) -> &mut ::runique::forms::manager::Forms {
            &mut self.form
        }
    };
    // Variante avec nom de champ personnalisé
    ($field:ident) => {
        fn from_form(form: ::runique::forms::manager::Forms) -> Self {
            Self { $field: form }
        }
        fn get_form(&self) -> &::runique::forms::manager::Forms {
            &self.$field
        }
        fn get_form_mut(&mut self) -> &mut ::runique::forms::manager::Forms {
            &mut self.$field
        }
    };
}
