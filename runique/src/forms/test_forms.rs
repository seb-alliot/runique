#[cfg(test)]
mod tests {
    use crate::forms::base::FormField;
    use crate::forms::field::RuniqueForm;
    use crate::forms::fields::boolean::BooleanField;
    use crate::forms::fields::number::NumericField;
    use crate::forms::fields::text::TextField;
    use crate::forms::form::Forms;
    use crate::utils::aliases::StrMap;
    use axum::http::Method;
    use std::collections::HashMap;

    // ── TextField ────────────────────────────────────────────────────────────────
    #[test]
    fn test_formulaire_get_valide() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("search").required());
        // Simule un GET avec un champ rempli
        form.add_value("search", "rust");
        assert!(form.is_valid().is_ok());
    }
    #[test]
    fn test_text_required_empty() {
        let mut field = TextField::text("username").required();
        field.set_value("");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_text_required_filled() {
        let mut field = TextField::text("username").required();
        field.set_value("alice");
        assert!(field.validate());
        assert!(field.error().is_none());
    }

    #[test]
    fn test_text_not_required_empty_is_valid() {
        let mut field = TextField::text("optional");
        field.set_value("");
        assert!(field.validate());
    }

    #[test]
    fn test_text_min_length_too_short() {
        let mut field = TextField::text("name").min_length(3, "Trop court");
        field.set_value("ab");
        assert!(!field.validate());
        assert_eq!(field.error().map(String::as_str), Some("Trop court"));
    }

    #[test]
    fn test_text_min_length_exact() {
        let mut field = TextField::text("name").min_length(3, "Trop court");
        field.set_value("abc");
        assert!(field.validate());
    }

    #[test]
    fn test_text_max_length_too_long() {
        let mut field = TextField::text("name").max_length(5, "Trop long");
        field.set_value("toolongvalue");
        assert!(!field.validate());
        assert_eq!(field.error().map(String::as_str), Some("Trop long"));
    }

    #[test]
    fn test_text_max_length_exact() {
        let mut field = TextField::text("name").max_length(5, "Trop long");
        field.set_value("alice");
        assert!(field.validate());
    }

    #[test]
    fn test_text_min_max_length_combined_invalid() {
        let mut field = TextField::text("code").min_length(3, "").max_length(6, "");
        field.set_value("ab");
        assert!(!field.validate());
    }

    #[test]
    fn test_text_min_max_length_combined_valid() {
        let mut field = TextField::text("code").min_length(3, "").max_length(6, "");
        field.set_value("hello");
        assert!(field.validate());
    }

    // ── Email ────────────────────────────────────────────────────────────────────

    #[test]
    fn test_email_invalid() {
        let mut field = TextField::email("email");
        field.set_value("not-an-email");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_email_valid() {
        let mut field = TextField::email("email");
        field.set_value("test@example.com");
        assert!(field.validate());
        assert!(field.error().is_none());
    }

    #[test]
    fn test_email_trimmed_and_lowercased() {
        let mut field = TextField::email("email");
        field.set_value("Test@Example.COM");
        assert!(field.validate());
        assert_eq!(field.value(), "test@example.com");
    }

    #[test]
    fn test_email_optional_empty_is_valid() {
        let mut field = TextField::email("email");
        field.set_value("");
        assert!(field.validate());
    }

    // ── URL ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_url_invalid() {
        let mut field = TextField::url("website");
        field.set_value("not-a-url");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_url_valid() {
        let mut field = TextField::url("website");
        field.set_value("https://example.com");
        assert!(field.validate());
        assert!(field.error().is_none());
    }

    // ── NumericField ─────────────────────────────────────────────────────────────

    #[test]
    fn test_integer_required_empty() {
        let mut field = NumericField::integer("age");
        field.set_required(true, None);
        field.set_value("");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_integer_valid() {
        let mut field = NumericField::integer("age");
        field.set_value("25");
        assert!(field.validate());
        assert!(field.error().is_none());
    }

    #[test]
    fn test_integer_invalid_text() {
        let mut field = NumericField::integer("age");
        field.set_value("abc");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_integer_optional_empty_is_valid() {
        let mut field = NumericField::integer("age");
        field.set_value("");
        assert!(field.validate());
    }

    #[test]
    fn test_integer_min_too_low() {
        let mut field = NumericField::integer("age").min(18.0, "");
        field.set_value("16");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_integer_min_ok() {
        let mut field = NumericField::integer("age").min(18.0, "Minimum 18");
        field.set_value("18");
        assert!(field.validate());
    }

    #[test]
    fn test_integer_max_too_high() {
        let mut field = NumericField::integer("qty").max(100.0, "");
        field.set_value("150");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_integer_max_ok() {
        let mut field = NumericField::integer("qty").max(100.0, "Maximum 100");
        field.set_value("99");
        assert!(field.validate());
    }

    #[test]
    fn test_float_valid() {
        let mut field = NumericField::float("price");
        field.set_value("19.99");
        assert!(field.validate());
    }

    #[test]
    fn test_float_comma_separator() {
        let mut field = NumericField::float("price");
        field.set_value("19,99");
        assert!(field.validate());
    }

    // ── BooleanField ─────────────────────────────────────────────────────────────

    #[test]
    fn test_boolean_required_unchecked() {
        let mut field = BooleanField::new("accept").required();
        field.set_value("false");
        assert!(!field.validate());
        assert!(field.error().is_some());
    }

    #[test]
    fn test_boolean_required_checked() {
        let mut field = BooleanField::new("accept").required();
        field.set_value("true");
        assert!(field.validate());
        assert!(field.error().is_none());
    }

    #[test]
    fn test_boolean_not_required_false_is_valid() {
        let mut field = BooleanField::new("newsletter");
        field.set_value("false");
        assert!(field.validate());
    }

    // ── Forms struct ─────────────────────────────────────────────────────────────

    #[test]
    fn test_forms_add_value_and_get_string() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("name"));
        form.add_value("name", "Alice");
        assert_eq!(form.get_string("name"), "Alice");
    }

    #[test]
    fn test_forms_get_string_missing_returns_empty() {
        let form = Forms::new("csrf");
        assert_eq!(form.get_string("nonexistent"), "");
    }

    #[test]
    fn test_forms_get_i32() {
        let mut form = Forms::new("csrf");
        form.field(&NumericField::integer("age"));
        form.add_value("age", "42");
        assert_eq!(form.get_i32("age"), 42);
    }

    #[test]
    fn test_forms_get_i32_invalid_returns_zero() {
        let mut form = Forms::new("csrf");
        form.field(&NumericField::integer("age"));
        form.add_value("age", "not_a_number");
        assert_eq!(form.get_i32("age"), 0);
    }

    #[test]
    fn test_forms_get_f64_dot() {
        let mut form = Forms::new("csrf");
        form.field(&NumericField::float("price"));
        form.add_value("price", "19.99");
        assert!((form.get_f64("price") - 19.99).abs() < 0.001);
    }

    #[test]
    fn test_forms_get_f64_comma() {
        let mut form = Forms::new("csrf");
        form.field(&NumericField::float("price"));
        form.add_value("price", "19,99");
        assert!((form.get_f64("price") - 19.99).abs() < 0.001);
    }

    #[test]
    fn test_forms_get_bool_true() {
        let mut form = Forms::new("csrf");
        form.field(&BooleanField::new("active"));
        form.add_value("active", "true");
        assert!(form.get_bool("active"));
    }

    #[test]
    fn test_forms_get_bool_one() {
        let mut form = Forms::new("csrf");
        form.field(&BooleanField::new("active"));
        form.add_value("active", "1");
        assert!(form.get_bool("active"));
    }

    #[test]
    fn test_forms_get_bool_false() {
        let mut form = Forms::new("csrf");
        form.field(&BooleanField::new("active"));
        form.add_value("active", "false");
        assert!(!form.get_bool("active"));
    }

    #[test]
    fn test_forms_get_option_some() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("name"));
        form.add_value("name", "Bob");
        assert_eq!(form.get_option("name"), Some("Bob".to_string()));
    }

    #[test]
    fn test_forms_get_option_none_on_empty() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("optional"));
        form.add_value("optional", "");
        assert_eq!(form.get_option("optional"), None);
    }

    #[test]
    fn test_forms_get_option_none_on_whitespace() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("optional"));
        form.add_value("optional", "   ");
        assert_eq!(form.get_option("optional"), None);
    }

    #[test]
    fn test_forms_fill() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("username"));
        form.field(&NumericField::integer("age"));

        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("username".to_string(), "bob".to_string());
        data.insert("age".to_string(), "30".to_string());
        form.fill(&data, Method::POST);

        assert_eq!(form.get_string("username"), "bob");
        assert_eq!(form.get_i32("age"), 30);
    }

    #[test]
    fn test_forms_fill_skips_password_fields() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::password("pwd"));

        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("pwd".to_string(), "secret".to_string());
        form.fill(&data, Method::GET);

        // fill() skips password fields — value stays empty
        assert_eq!(form.get_string("pwd"), "");
    }

    #[test]
    fn test_forms_is_valid_ok() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("name").required());
        form.add_value("name", "Alice");
        assert!(form.is_valid().is_ok());
    }

    #[test]
    fn test_forms_is_valid_missing_required() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("name").required());
        // name not filled → validation fails
        assert!(form.is_valid().is_err());
    }

    #[test]
    fn test_forms_fill_patch_relaxes_password_required() {
        // In edit mode (PATCH), a required password field left empty must not fail validation.
        // An empty password means "keep existing" — NotSet at DB level.
        let mut form = Forms::new("csrf");
        form.field(&TextField::password("pwd").required());

        let data: HashMap<String, String> = HashMap::new(); // empty — no password submitted
        form.fill(&data, Method::PATCH);

        // required is relaxed → valid even with no password
        assert!(form.is_valid().is_ok());
    }

    #[test]
    fn test_forms_has_errors_after_invalid() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("name").required());
        let _ = form.is_valid();
        assert!(form.has_errors());
    }

    #[test]
    fn test_forms_has_no_errors_when_valid() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("name"));
        form.add_value("name", "Alice");
        let _ = form.is_valid();
        assert!(!form.has_errors());
    }

    #[test]
    fn test_forms_errors_map() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("email").required());
        form.field(&NumericField::integer("age").min(0.0, ""));
        form.add_value("age", "-1");
        let _ = form.is_valid();
        let errors = form.errors();
        assert!(errors.contains_key("email"));
    }

    #[test]
    fn test_forms_get_option_i32_some() {
        let mut form = Forms::new("csrf");
        form.field(&NumericField::integer("qty"));
        form.add_value("qty", "5");
        assert_eq!(form.get_option_i32("qty"), Some(5));
    }

    #[test]
    fn test_forms_get_option_i32_none_empty() {
        let mut form = Forms::new("csrf");
        form.field(&NumericField::integer("qty"));
        assert_eq!(form.get_option_i32("qty"), None);
    }

    // ── RuniqueForm — validation basique ─────────────────────────────────────────

    struct LoginForm {
        form: Forms,
    }

    impl RuniqueForm for LoginForm {
        fn register_fields(form: &mut Forms) {
            form.field(&TextField::text("username").required());
            form.field(&TextField::password("password").required().no_hash());
        }
        fn from_form(form: Forms) -> Self {
            Self { form }
        }
        fn get_form(&self) -> &Forms {
            &self.form
        }
        fn get_form_mut(&mut self) -> &mut Forms {
            &mut self.form
        }
    }

    #[tokio::test]
    async fn test_runique_form_valid() {
        let mut form = LoginForm {
            form: Forms::new("csrf"),
        };
        LoginForm::register_fields(&mut form.form);
        form.get_form_mut().add_value("username", "alice");
        form.get_form_mut().add_value("password", "secret");
        assert!(form.is_valid().await);
    }

    #[tokio::test]
    async fn test_runique_form_missing_username() {
        let mut form = LoginForm {
            form: Forms::new("csrf"),
        };
        LoginForm::register_fields(&mut form.form);
        form.get_form_mut().add_value("password", "secret");
        assert!(!form.is_valid().await);
    }

    #[tokio::test]
    async fn test_runique_form_missing_password() {
        let mut form = LoginForm {
            form: Forms::new("csrf"),
        };
        LoginForm::register_fields(&mut form.form);
        form.get_form_mut().add_value("username", "alice");
        assert!(!form.is_valid().await);
    }

    // ── RuniqueForm — clean() (validation croisée) ───────────────────────────────

    struct PasswordChangeForm {
        form: Forms,
    }

    #[async_trait::async_trait]
    impl RuniqueForm for PasswordChangeForm {
        fn register_fields(form: &mut Forms) {
            form.field(&TextField::password("password").required().no_hash());
            form.field(&TextField::password("confirm").required().no_hash());
        }
        fn from_form(form: Forms) -> Self {
            Self { form }
        }
        fn get_form(&self) -> &Forms {
            &self.form
        }
        fn get_form_mut(&mut self) -> &mut Forms {
            &mut self.form
        }

        async fn clean(&mut self) -> Result<(), StrMap> {
            let password = self.get_form().get_string("password");
            let confirm = self.get_form().get_string("confirm");
            if password != confirm {
                let mut errors = HashMap::new();
                errors.insert(
                    "confirm".to_string(),
                    "Les mots de passe ne correspondent pas".to_string(),
                );
                return Err(errors);
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_clean_passwords_match() {
        let mut form = PasswordChangeForm {
            form: Forms::new("csrf"),
        };
        PasswordChangeForm::register_fields(&mut form.form);
        form.get_form_mut().add_value("password", "secret123");
        form.get_form_mut().add_value("confirm", "secret123");
        assert!(form.is_valid().await);
    }

    #[tokio::test]
    async fn test_clean_passwords_mismatch() {
        let mut form = PasswordChangeForm {
            form: Forms::new("csrf"),
        };
        PasswordChangeForm::register_fields(&mut form.form);
        form.get_form_mut().add_value("password", "secret123");
        form.get_form_mut().add_value("confirm", "different");
        assert!(!form.is_valid().await);

        let errors = form.get_form().errors();
        assert!(errors.contains_key("confirm"));
    }

    #[tokio::test]
    async fn test_clean_not_called_when_fields_invalid() {
        // Si les champs sont invalides, clean() ne doit pas être appelé
        let mut form = PasswordChangeForm {
            form: Forms::new("csrf"),
        };
        PasswordChangeForm::register_fields(&mut form.form);
        // password vide → required échoue avant clean()
        form.get_form_mut().add_value("confirm", "something");
        assert!(!form.is_valid().await);
    }

    // ── RuniqueForm — clean_field() (validation métier par champ) ────────────────

    struct UsernameForm {
        form: Forms,
    }

    #[async_trait::async_trait]
    impl RuniqueForm for UsernameForm {
        fn register_fields(form: &mut Forms) {
            form.field(&TextField::text("username").required());
        }
        fn from_form(form: Forms) -> Self {
            Self { form }
        }
        fn get_form(&self) -> &Forms {
            &self.form
        }
        fn get_form_mut(&mut self) -> &mut Forms {
            &mut self.form
        }

        async fn clean_field(&mut self, name: &str) -> bool {
            if name == "username" {
                let val = self.get_form().get_string("username");
                if val.to_lowercase().contains("admin") {
                    if let Some(f) = self.get_form_mut().fields.get_mut("username") {
                        f.set_error("Le nom 'admin' est réservé".to_string());
                    }
                    return false;
                }
            }
            true
        }
    }

    #[tokio::test]
    async fn test_clean_field_reserved_name_rejected() {
        let mut form = UsernameForm {
            form: Forms::new("csrf"),
        };
        UsernameForm::register_fields(&mut form.form);
        form.get_form_mut().add_value("username", "admin_user");
        assert!(!form.is_valid().await);

        let errors = form.get_form().errors();
        assert!(errors.contains_key("username"));
        assert_eq!(
            errors.get("username").map(String::as_str),
            Some("Le nom 'admin' est réservé")
        );
    }

    #[tokio::test]
    async fn test_clean_field_valid_name_accepted() {
        let mut form = UsernameForm {
            form: Forms::new("csrf"),
        };
        UsernameForm::register_fields(&mut form.form);
        form.get_form_mut().add_value("username", "alice");
        assert!(form.is_valid().await);
    }

    #[tokio::test]
    async fn test_clean_field_not_called_when_required_missing() {
        // Si un champ requis est vide, clean_field ne doit pas valider la logique métier.
        // On simule un vrai POST (fill avec méthode POST) pour que submitted = true.
        let mut form = UsernameForm {
            form: Forms::new("csrf"),
        };
        UsernameForm::register_fields(&mut form.form);
        // POST avec username vide → submitted=true, required échoue, clean_field non invoqué
        let data: HashMap<String, String> = HashMap::new();
        form.form.fill(&data, Method::POST);
        assert!(!form.is_valid().await);
    }
}
