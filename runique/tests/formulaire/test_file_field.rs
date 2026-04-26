//! Tests — forms/fields/file.rs
//! Couvre : AllowedExtensions, FileUploadConfig, IntoUploadPath, FileField builders, FileField::validate()

use runique::config::StaticConfig;
use runique::forms::base::FormField;
use runique::forms::fields::file::{AllowedExtensions, FileField, FileUploadConfig};

// ═══════════════════════════════════════════════════════════════
// AllowedExtensions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_allowed_extensions_images() {
    let ext = AllowedExtensions::images();
    assert!(ext.extensions.contains(&"jpg".to_string()));
    assert!(ext.extensions.contains(&"png".to_string()));
    assert!(ext.extensions.contains(&"webp".to_string()));
}

#[test]
fn test_allowed_extensions_documents() {
    let ext = AllowedExtensions::documents();
    assert!(ext.extensions.contains(&"pdf".to_string()));
    assert!(ext.extensions.contains(&"docx".to_string()));
}

#[test]
fn test_allowed_extensions_any_empty_list() {
    let ext = AllowedExtensions::any();
    assert!(ext.extensions.is_empty());
}

#[test]
fn test_allowed_extensions_new_custom() {
    let ext = AllowedExtensions::new(vec!["rs", "toml"]);
    assert!(ext.extensions.contains(&"rs".to_string()));
    assert!(ext.extensions.contains(&"toml".to_string()));
}

#[test]
fn test_is_allowed_svg_always_blocked() {
    let ext = AllowedExtensions::any();
    assert!(!ext.is_allowed("photo.svg"));
    assert!(!ext.is_allowed("icon.SVG"));
}

#[test]
fn test_is_allowed_empty_list_allows_all() {
    let ext = AllowedExtensions::any();
    assert!(ext.is_allowed("file.jpg"));
    assert!(ext.is_allowed("file.pdf"));
    assert!(ext.is_allowed("file.rs"));
}

#[test]
fn test_is_allowed_matched_extension() {
    let ext = AllowedExtensions::images();
    assert!(ext.is_allowed("photo.jpg"));
    assert!(ext.is_allowed("photo.PNG"));
}

#[test]
fn test_is_allowed_unmatched_extension() {
    let ext = AllowedExtensions::images();
    assert!(!ext.is_allowed("document.pdf"));
}

#[test]
fn test_is_allowed_no_extension() {
    let ext = AllowedExtensions::images();
    assert!(!ext.is_allowed("Makefile"));
}

// ═══════════════════════════════════════════════════════════════
// FileUploadConfig
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_file_upload_config_default() {
    let cfg = FileUploadConfig::default();
    assert!(cfg.upload_to.is_none());
    assert!(cfg.max_size.is_some());
}

#[test]
fn test_file_upload_config_new() {
    let cfg = FileUploadConfig::new();
    assert!(cfg.upload_to.is_none());
}

#[test]
fn test_file_upload_config_max_size() {
    let cfg = FileUploadConfig::new().max_size(5 * 1024 * 1024);
    assert_eq!(cfg.max_size, Some(5 * 1024 * 1024));
}

#[test]
fn test_file_upload_config_upload_to() {
    let cfg = FileUploadConfig::new().upload_to("media/avatars".to_string());
    let f = cfg.upload_to.unwrap();
    assert_eq!(f("avatar"), "media/avatars");
}

// ═══════════════════════════════════════════════════════════════
// IntoUploadPath
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_into_upload_path_str() {
    let field = FileField::image("avatar").upload_to("media/photos");
    let f = field.upload_config.upload_to.unwrap();
    assert_eq!(f("avatar"), "media/photos");
}

#[test]
fn test_into_upload_path_string() {
    let path = "media/docs".to_string();
    let field = FileField::document("doc").upload_to(path);
    let f = field.upload_config.upload_to.unwrap();
    assert_eq!(f("doc"), "media/docs");
}

#[test]
fn test_into_upload_path_static_config() {
    let mut config = StaticConfig::default();
    config.media_root = "media".to_string();
    let field = FileField::image("pic").upload_to(&config);
    let f = field.upload_config.upload_to.unwrap();
    assert_eq!(f("pic"), "media");
}

// ═══════════════════════════════════════════════════════════════
// FileField builders
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_file_field_image() {
    let f = FileField::image("photo");
    assert_eq!(f.base.name, "photo");
    assert!(f.allowed_extensions.extensions.contains(&"jpg".to_string()));
}

#[test]
fn test_file_field_document() {
    let f = FileField::document("cv");
    assert_eq!(f.base.name, "cv");
    assert!(f.allowed_extensions.extensions.contains(&"pdf".to_string()));
}

#[test]
fn test_file_field_any() {
    let f = FileField::any("attachment");
    assert_eq!(f.base.name, "attachment");
    assert!(f.allowed_extensions.extensions.is_empty());
}

#[test]
fn test_file_field_label() {
    let f = FileField::image("pic").label("Profile photo");
    assert_eq!(f.base.label, "Profile photo");
}

#[test]
fn test_file_field_max_size() {
    let f = FileField::image("pic").max_size(2 * 1024 * 1024);
    assert_eq!(f.upload_config.max_size, Some(2 * 1024 * 1024));
}

#[test]
fn test_file_field_max_files() {
    let f = FileField::any("gallery").max_files(5);
    assert_eq!(f.max_files, Some(5));
    assert!(f.base.html_attributes.contains_key("multiple"));
}

#[test]
fn test_file_field_max_files_single_no_multiple_attr() {
    let f = FileField::any("file").max_files(1);
    assert_eq!(f.max_files, Some(1));
    assert!(!f.base.html_attributes.contains_key("multiple"));
}

#[test]
fn test_file_field_required() {
    let f = FileField::image("photo").required();
    assert!(f.base.is_required.choice);
}

#[test]
fn test_file_field_max_dimensions() {
    let f = FileField::image("thumb").max_dimensions(800, 600);
    assert_eq!(f.max_width, Some(800));
    assert_eq!(f.max_height, Some(600));
}

#[test]
fn test_file_field_allowed_extensions_override() {
    let f = FileField::any("file").allowed_extensions(vec!["zip", "tar"]);
    assert!(f.allowed_extensions.extensions.contains(&"zip".to_string()));
    assert!(f.allowed_extensions.extensions.contains(&"tar".to_string()));
}

#[test]
fn test_file_field_upload_to_env() {
    let f = FileField::image("avatar").upload_to_env();
    assert!(f.upload_config.upload_to.is_some());
}

// ═══════════════════════════════════════════════════════════════
// FileField::validate()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_file_field_validate_required_empty_fails() {
    let mut f = FileField::image("photo").required();
    f.set_value("");
    assert!(!f.validate());
    assert!(f.error().is_some());
}

#[test]
fn test_file_field_validate_not_required_empty_passes() {
    let mut f = FileField::image("photo");
    f.set_value("");
    assert!(f.validate());
    assert!(f.error().is_none());
}

#[test]
fn test_file_field_validate_extension_blocked() {
    let mut f = FileField::image("photo");
    // .svg is always blocked
    f.set_value("photo.svg");
    assert!(!f.validate());
    assert!(f.error().is_some());
}

#[test]
fn test_file_field_validate_custom_extension_blocked() {
    let mut f = FileField::any("file").allowed_extensions(vec!["pdf"]);
    f.set_value("script.exe");
    assert!(!f.validate());
    assert!(f.error().is_some());
}

#[test]
fn test_file_field_validate_max_files_exceeded() {
    let mut f = FileField::any("gallery").max_files(2);
    // 3 files, max is 2 → fail
    f.set_value("a.jpg,b.jpg,c.jpg");
    assert!(!f.validate());
    assert!(f.error().is_some());
}

#[test]
fn test_file_field_validate_max_files_exact() {
    let mut f = FileField::document("docs").max_files(2);
    // Extension "pdf" is allowed, 2 files exactly at limit
    // Files don't exist → image validation (FileFieldType::Document) skips is_valid_path
    // Size check uses metadata → Err (file not found) → skipped
    f.set_value("a.pdf,b.pdf");
    // Should pass extension + count, and no image check for Document type
    assert!(f.validate());
}

#[test]
fn test_file_field_validate_image_not_found_fails() {
    let mut f = FileField::image("photo");
    // jpg extension passes, but file doesn't exist → is_valid_path returns false
    f.set_value("nonexistent_file_abc.jpg");
    assert!(!f.validate());
    assert!(f.error().is_some());
}

// ═══════════════════════════════════════════════════════════════
// FileField::finalize()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_file_field_finalize_no_upload_to_ok() {
    let mut f = FileField::any("file");
    f.set_value("some.pdf");
    assert!(f.finalize().is_ok());
}

#[test]
fn test_file_field_finalize_empty_value_ok() {
    let mut f = FileField::any("file").upload_to("media/files");
    f.set_value("");
    assert!(f.finalize().is_ok());
}

#[test]
fn test_file_field_finalize_nonexistent_file_keeps_path() {
    let mut f = FileField::any("file").upload_to("media/files");
    f.set_value("nonexistent_xyz.pdf");
    // File doesn't exist → path kept as-is, no error
    assert!(f.finalize().is_ok());
    assert!(f.value().contains("nonexistent_xyz.pdf"));
}
