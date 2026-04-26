//! Macro `impl_form_access!` — implements access methods on form structs.

#[macro_export]
macro_rules! impl_form_access {
    (model) => {
        fn register_fields(form: &mut $crate::forms::Forms) {
            <Self as $crate::forms::model_form::ModelForm>::model_register_fields(form);
        }
        fn from_form(form: $crate::forms::Forms) -> Self {
            Self { form }
        }
        fn get_form(&self) -> &$crate::forms::Forms {
            &self.form
        }
        fn get_form_mut(&mut self) -> &mut $crate::forms::Forms {
            &mut self.form
        }
    };
    () => {
        fn from_form(form: $crate::forms::Forms) -> Self {
            Self { form }
        }
        fn get_form(&self) -> &$crate::forms::Forms {
            &self.form
        }
        fn get_form_mut(&mut self) -> &mut $crate::forms::Forms {
            &mut self.form
        }
    };
    ($field:ident) => {
        fn from_form(form: $crate::forms::Forms) -> Self {
            Self { $field: form }
        }
        fn get_form(&self) -> &$crate::forms::Forms {
            &self.$field
        }
        fn get_form_mut(&mut self) -> &mut $crate::forms::Forms {
            &mut self.$field
        }
    };
    (model) => {
        fn register_fields(form: &mut $crate::forms::Forms) {
            <Self as $crate::forms::model_form::ModelForm>::model_register_fields(form);
        }
        fn from_form(form: $crate::forms::Forms) -> Self {
            Self { form }
        }
        fn get_form(&self) -> &$crate::forms::Forms {
            &self.form
        }
        fn get_form_mut(&mut self) -> &mut $crate::forms::Forms {
            &mut self.form
        }
    };
}
