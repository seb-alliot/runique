//! Macro `impl_form_access!` — the body of a form's `impl FormAccess`.

#[macro_export]
macro_rules! impl_form_access {
    (model) => {
        fn register_fields(form: &mut $crate::forms::Forms) {
            <Self as $crate::forms::model_form::ModelForm>::model_register_fields(form);
            <Self as $crate::forms::field::RuniqueForm>::customize(form);
            <Self as $crate::forms::model_form::ModelForm>::enforce_schema_limits(form);
        }
        fn from_form(form: $crate::forms::Forms) -> Self {
            Self { form }
        }
        fn forms(&self) -> &$crate::forms::Forms {
            &self.form
        }
        fn forms_mut(&mut self) -> &mut $crate::forms::Forms {
            &mut self.form
        }
    };
    () => {
        fn from_form(form: $crate::forms::Forms) -> Self {
            Self { form }
        }
        fn forms(&self) -> &$crate::forms::Forms {
            &self.form
        }
        fn forms_mut(&mut self) -> &mut $crate::forms::Forms {
            &mut self.form
        }
    };
}
