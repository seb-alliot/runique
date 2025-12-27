use rusti::rusti_form;
use rusti::formulaire::formsrusti::{Forms, FormulaireTrait};
use rusti::formulaire::field::{CharField};
use std::collections::HashMap;



#[rusti_form]
pub struct UsernameForm {
    pub form: Forms,
}
impl FormulaireTrait for UsernameForm {
    fn new() -> Self {
        Self { form: Forms::new() }
    }
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.require("username", &CharField { allow_blank: false }, raw_data);
        self.is_valid()
    }

}