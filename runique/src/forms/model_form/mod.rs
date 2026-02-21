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
}
