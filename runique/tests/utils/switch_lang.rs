mod switch_lang_utils {
	#[test]
	fn test_lang_enum() {
		use runique::utils::trad::switch_lang::Lang;
		let l = Lang::Fr;
		assert_eq!(format!("{:?}", l), "Fr");
	}
}
