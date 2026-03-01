use runique::utils::trad::switch_lang::Lang;

mod switch_lang_utils {
    #[test]
    fn test_lang_enum() {
        use runique::utils::trad::switch_lang::Lang;
        let l = Lang::Fr;
        assert_eq!(format!("{:?}", l), "Fr");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_simple() {
        let fr = Lang::Fr;
        assert_eq!(fr.get("forms.required"), "Ce champ est obligatoire");

        let en = Lang::En;
        assert_eq!(en.get("forms.required"), "This field is required");
    }

    #[test]
    fn test_get_nested() {
        let fr = Lang::Fr;
        assert_eq!(fr.get("error.title.not_found"), "Page non trouvée");
    }

    #[test]
    fn test_format() {
        let fr = Lang::Fr;
        assert_eq!(fr.format("forms.too_short", &[5]), "Trop court (min 5)");
    }

    #[test]
    fn test_missing_key() {
        let fr = Lang::Fr;
        assert_eq!(fr.get("missing.key"), "missing.key");
    }
}
