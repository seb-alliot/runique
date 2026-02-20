use runique::admin::{AdminConfig, AdminRegistry, AdminResource, DisplayConfig, ResourcePermissions};
use runique::config::{RuniqueConfig, SecurityConfig, ServerConfig};
use runique::flash::{FlashMessage, MessageLevel};
use runique::forms::field::FormField;
use runique::forms::fields::{NumericField, TextField};
use runique::middleware::MiddlewareConfig;
use runique::migration::{ColumnDef, ForeignKeyDef, HooksDef, IndexDef, ModelSchema, PrimaryKeyDef, RelationDef};

#[test]
fn smoke_config_defaults_are_accessible() {
    let server = ServerConfig::default();
    let security = SecurityConfig::from_env();
    let middleware = MiddlewareConfig::default();

    assert!(!server.domain_server.is_empty() || server.port == 0);
    assert!(!security.allowed_hosts.is_empty());
    assert!(middleware.enable_cache);

    let cfg = RuniqueConfig::from_env();
    assert!(!cfg.base_dir.is_empty());
}

#[test]
fn smoke_flash_api() {
    let msg = FlashMessage::success("ok");
    assert!(matches!(msg.level, MessageLevel::Success));
    assert_eq!(msg.level.as_css_class(), "success-message");
}

#[test]
fn smoke_form_fields_validation() {
    let mut text = TextField::text("username").min_length(3, "").max_length(10, "");
    text.set_value("abcd");
    assert!(text.validate());

    let mut num = NumericField::integer("age").min(18.0, "").max(99.0, "");
    num.set_value("25");
    assert!(num.validate());
}

#[test]
fn smoke_admin_registry_and_routes_helpers() {
    let perms = ResourcePermissions::uniform(vec!["admin".to_string()]);
    let res = AdminResource::with_permissions(
        "users",
        "users::Model",
        "RegisterForm",
        "Users",
        perms,
    )
    .display(DisplayConfig::new().icon("user").pagination(30));

    assert_eq!(res.list_route(), "/users/list");
    assert_eq!(res.create_route(), "/users/create");

    let mut registry = AdminRegistry::new();
    registry.register(res);
    assert!(registry.contains("users"));
    assert_eq!(registry.len(), 1);

    let config = AdminConfig::new().site_title("Admin").prefix("/admin");
    assert!(config.enabled);
    assert_eq!(config.prefix, "/admin");
}

#[test]
fn smoke_migration_schema_builders() {
    let schema = ModelSchema::new("User")
        .table_name("users")
        .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
        .column(ColumnDef::new("username").varchar(150).required().unique())
        .column(ColumnDef::new("age").integer().nullable())
        .index(IndexDef::new(vec!["username"]).unique())
        .foreign_key(
            ForeignKeyDef::new("group_id")
                .references("groups")
                .to_column("id"),
        )
        .relation(RelationDef::has_many("posts::Entity"))
        .hooks(HooksDef::new().before_save(10, "crate::hooks::before_save"))
        .build()
        .expect("schema should build");

    assert_eq!(schema.table_name, "users");
    assert_eq!(schema.columns.len(), 2);

    // Ensure SeaQuery generation stays available
    let _stmt = schema.to_migration();
}

#[test]
fn smoke_column_to_form_bridge() {
    let col = ColumnDef::new("email").varchar(255).required();
    let field = col.to_form_field().expect("email should map to form field");
    assert_eq!(field.name(), "email");
    assert!(field.required());
}
