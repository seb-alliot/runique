//! The default language, in a process where no language was chosen yet: the
//! only place it can be seen taking effect, hence a file with a single test.
use runique_i18n::{Lang, current_lang, set_default_lang, set_lang};

#[test]
fn a_default_language_is_taken_until_one_is_chosen() {
    assert_eq!(current_lang(), Lang::En, "nothing chosen: English");

    set_default_lang(Lang::Fr);
    assert_eq!(current_lang(), Lang::Fr, "the default is taken");

    set_default_lang(Lang::De);
    assert_eq!(current_lang(), Lang::Fr, "a second default changes nothing");

    set_lang(Lang::It);
    set_default_lang(Lang::De);
    assert_eq!(current_lang(), Lang::It, "nor does one after a choice");
}
