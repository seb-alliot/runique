mod relation_kind_migration {
	#[test]
	fn test_relation_kind_enum() {
		use runique::migration::relation::RelationKind;
		let kind = RelationKind::HasOne;
		assert_eq!(format!("{:?}", kind), "HasOne");
		let kind = RelationKind::HasMany;
		assert_eq!(format!("{:?}", kind), "HasMany");
		let kind = RelationKind::BelongsTo { from: "a".to_string(), to: "b".to_string() };
		assert!(format!("{:?}", kind).contains("BelongsTo"));
		let kind = RelationKind::ManyToMany { via: "pivot".to_string() };
		assert!(format!("{:?}", kind).contains("ManyToMany"));
	}
}
