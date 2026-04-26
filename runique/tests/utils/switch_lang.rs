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

    // ── From<&str> ──────────────────────────────────────────────

    #[test]
    fn test_from_str_fr_variants() {
        assert_eq!(Lang::from("fr"), Lang::Fr);
        assert_eq!(Lang::from("fr-FR"), Lang::Fr);
        assert_eq!(Lang::from("fr_FR"), Lang::Fr);
        assert_eq!(Lang::from("fr_FR.UTF-8"), Lang::Fr);
        assert_eq!(Lang::from("fr-ca"), Lang::Fr);
        assert_eq!(Lang::from("fr-be"), Lang::Fr);
        assert_eq!(Lang::from("fr-ch"), Lang::Fr);
    }

    #[test]
    fn test_from_str_en_variants() {
        assert_eq!(Lang::from("en"), Lang::En);
        assert_eq!(Lang::from("en-US"), Lang::En);
        assert_eq!(Lang::from("en-GB"), Lang::En);
        assert_eq!(Lang::from("en-ca"), Lang::En);
    }

    #[test]
    fn test_from_str_all_languages() {
        assert_eq!(Lang::from("it"), Lang::It);
        assert_eq!(Lang::from("it-it"), Lang::It);
        assert_eq!(Lang::from("es"), Lang::Es);
        assert_eq!(Lang::from("es-mx"), Lang::Es);
        assert_eq!(Lang::from("de"), Lang::De);
        assert_eq!(Lang::from("de-at"), Lang::De);
        assert_eq!(Lang::from("pt"), Lang::Pt);
        assert_eq!(Lang::from("pt-br"), Lang::Pt);
        assert_eq!(Lang::from("ja"), Lang::Ja);
        assert_eq!(Lang::from("ja-jp"), Lang::Ja);
        assert_eq!(Lang::from("zh"), Lang::Zh);
        assert_eq!(Lang::from("zh-cn"), Lang::Zh);
        assert_eq!(Lang::from("zh-tw"), Lang::Zh);
        assert_eq!(Lang::from("ru"), Lang::Ru);
        assert_eq!(Lang::from("ru-ru"), Lang::Ru);
        assert_eq!(Lang::from("ru-by"), Lang::Ru);
    }

    #[test]
    fn test_from_str_unknown_falls_back_to_en() {
        assert_eq!(Lang::from("xx"), Lang::En);
        assert_eq!(Lang::from(""), Lang::En);
        assert_eq!(Lang::from("zz-ZZ"), Lang::En);
    }

    #[test]
    fn test_from_string() {
        assert_eq!(Lang::from("fr".to_string()), Lang::Fr);
        assert_eq!(Lang::from("de".to_string()), Lang::De);
        assert_eq!(Lang::from("ru".to_string()), Lang::Ru);
    }

    // ── code() ──────────────────────────────────────────────────

    #[test]
    fn test_code_all_langs() {
        assert_eq!(Lang::Fr.code(), "fr");
        assert_eq!(Lang::En.code(), "en");
        assert_eq!(Lang::It.code(), "it");
        assert_eq!(Lang::Es.code(), "es");
        assert_eq!(Lang::De.code(), "de");
        assert_eq!(Lang::Pt.code(), "pt");
        assert_eq!(Lang::Ja.code(), "ja");
        assert_eq!(Lang::Zh.code(), "zh");
        assert_eq!(Lang::Ru.code(), "ru");
    }

    // ── load_json / get() pour toutes les langues ───────────────

    #[test]
    fn test_get_all_languages_required_key() {
        for lang in [
            Lang::It,
            Lang::Es,
            Lang::De,
            Lang::Pt,
            Lang::Ja,
            Lang::Zh,
            Lang::Ru,
        ] {
            let val = lang.get("forms.required");
            assert!(!val.is_empty(), "{:?}.get('forms.required') vide", lang);
        }
    }

    #[test]
    fn test_get_missing_key_fallback_returns_key() {
        let val = Lang::Fr.get("absolutely.nonexistent.key");
        assert_eq!(val, "absolutely.nonexistent.key");
    }

    // ── format() ────────────────────────────────────────────────

    #[test]
    fn test_format_en() {
        assert_eq!(
            Lang::En.format("forms.too_short", &[3u32]),
            "Too short (min 3)"
        );
    }

    #[test]
    fn test_format_no_args() {
        let result = Lang::En.format("forms.required", &[] as &[&str]);
        assert_eq!(result, "This field is required");
    }

    // ── default ─────────────────────────────────────────────────

    #[test]
    fn test_lang_default_is_en() {
        assert_eq!(Lang::default(), Lang::En);
    }

    // ── global : set_lang / current_lang / t / tf ───────────────

    #[test]
    #[serial_test::serial]
    fn test_set_lang_and_current_lang() {
        use runique::utils::trad::switch_lang::{current_lang, set_lang};
        set_lang(Lang::Fr);
        assert_eq!(current_lang(), Lang::Fr);
        set_lang(Lang::En);
    }

    #[test]
    #[serial_test::serial]
    fn test_t_uses_global_lang() {
        use runique::utils::trad::switch_lang::{set_lang, t};
        set_lang(Lang::Fr);
        assert_eq!(t("forms.required"), "Ce champ est obligatoire");
        set_lang(Lang::En);
    }

    #[test]
    #[serial_test::serial]
    fn test_tf_uses_global_lang() {
        use runique::utils::trad::switch_lang::{set_lang, tf};
        set_lang(Lang::En);
        let val = tf("forms.too_short", &[7u32]);
        assert!(val.contains("7"));
        set_lang(Lang::En);
    }

    #[test]
    #[serial_test::serial]
    fn test_set_lang_all_variants() {
        use runique::utils::trad::switch_lang::{current_lang, set_lang};
        for lang in [
            Lang::Fr,
            Lang::It,
            Lang::Es,
            Lang::De,
            Lang::Pt,
            Lang::Ja,
            Lang::Zh,
            Lang::Ru,
        ] {
            set_lang(lang);
            assert_eq!(current_lang(), lang);
        }
        set_lang(Lang::En);
    }
}
