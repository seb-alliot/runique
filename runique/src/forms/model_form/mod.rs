//! `ModelForm` trait — links a form to a `ModelSchema`, generates fields automatically.

pub trait ModelForm: Sized + Send + Sync {
    fn schema() -> crate::migration::schema::ModelSchema;
    fn fields() -> Option<&'static [&'static str]> {
        None
    }
    fn exclude() -> Option<&'static [&'static str]> {
        None
    }
    fn model_register_fields(form: &mut crate::forms::Forms) {
        Self::schema().fill_form(form, Self::fields(), Self::exclude());
    }
    /// Called after `customize`: see [`ModelSchema::enforce_limits`](crate::migration::schema::ModelSchema::enforce_limits).
    fn enforce_schema_limits(form: &mut crate::forms::Forms) {
        Self::schema().enforce_limits(form);
    }
}
