use runique_dsl::snake_case;

#[test]
fn a_pascal_case_name_gets_one_underscore_per_word() {
    assert_eq!(snake_case("Article"), "article");
    assert_eq!(snake_case("BlogPost"), "blog_post");
    assert_eq!(snake_case("EihwazUsers"), "eihwaz_users");
}

#[test]
fn a_name_already_in_snake_case_is_left_as_written() {
    assert_eq!(snake_case("blog_post"), "blog_post");
    assert_eq!(snake_case("eihwaz_users"), "eihwaz_users");
}

#[test]
fn the_first_letter_never_gets_an_underscore() {
    assert_eq!(snake_case("A"), "a");
    assert_eq!(snake_case("a"), "a");
    assert_eq!(snake_case(""), "");
}

#[test]
fn an_underscore_already_written_is_not_doubled() {
    assert_eq!(snake_case("Blog_Post"), "blog_post");
    assert_eq!(snake_case("_Blog"), "_blog");
}

#[test]
fn underscores_the_dev_wrote_are_kept() {
    assert_eq!(snake_case("blog__post"), "blog__post");
    assert_eq!(snake_case("Blog__Post"), "blog__post");
}

#[test]
fn accented_capitals_are_lowered_like_the_others() {
    assert_eq!(snake_case("École"), "école");
    assert_eq!(snake_case("MenuÉté"), "menu_été");
}

#[test]
fn each_capital_of_an_acronym_starts_a_word() {
    assert_eq!(snake_case("HTTPServer"), "h_t_t_p_server");
}

#[test]
fn digits_do_not_start_a_word() {
    assert_eq!(snake_case("Menu2"), "menu2");
    assert_eq!(snake_case("Menu2Jour"), "menu2_jour");
}
