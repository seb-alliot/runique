//! Keys of `forms`.
pub mod error {
    pub const AEGIS_JSON: &str = "forms.error.aegis_json";
    pub const AEGIS_QUERY: &str = "forms.error.aegis_query";
    pub const BODY: &str = "forms.error.body";
    pub const BODY_TOO_LARGE: &str = "forms.error.body_too_large";
    pub const CHECKBOX_REQUIRED: &str = "forms.error.checkbox_required";
    pub const CHOICE_INVALID: &str = "forms.error.choice_invalid";
    pub const CHOICE_INVALID_VALUE: &str = "forms.error.choice_invalid_value";
    pub const COLOR_BAD_HEX: &str = "forms.error.color_bad_hex";
    pub const COLOR_INVALID: &str = "forms.error.color_invalid";
    pub const COLOR_NO_HASH: &str = "forms.error.color_no_hash";
    pub const CONFIG_NOT_FOUND: &str = "forms.error.config_not_found";
    pub const DATA_INVALID: &str = "forms.error.data_invalid";
    pub const DATA_PASSWORD: &str = "forms.error.data_password";
    pub const DATA_REQUIRED: &str = "forms.error.data_required";
    pub const DATE_INVALID: &str = "forms.error.date_invalid";
    pub const DATE_TOO_FAR: &str = "forms.error.date_too_far";
    pub const DATE_TOO_OLD: &str = "forms.error.date_too_old";
    pub const DATETIME_TOO_FAR: &str = "forms.error.datetime_too_far";
    pub const DATETIME_TOO_OLD: &str = "forms.error.datetime_too_old";
    pub const DB: &str = "forms.error.db";
    pub const DB_CHECK: &str = "forms.error.db_check";
    pub const DB_FOREIGN_KEY: &str = "forms.error.db_foreign_key";
    pub const DB_NOT_NULL: &str = "forms.error.db_not_null";
    pub const DELEGATED_NO_HASH: &str = "forms.error.delegated_no_hash";
    pub const DELETE_LINKED: &str = "forms.error.delete_linked";
    pub const DURATION_INVALID: &str = "forms.error.duration_invalid";
    pub const DURATION_TOO_LONG: &str = "forms.error.duration_too_long";
    pub const DURATION_TOO_SHORT: &str = "forms.error.duration_too_short";
    pub const EMAIL_INVALID: &str = "forms.error.email_invalid";
    pub const FIELD_LOOKS_HASHED: &str = "forms.error.field_looks_hashed";
    pub const FIELDS_REFUSED: &str = "forms.error.fields_refused";
    pub const FILE_CREATE: &str = "forms.error.file_create";
    pub const FILE_EXTENSION_BLOCKED: &str = "forms.error.file_extension_blocked";
    pub const FILE_INVALID_AUDIO: &str = "forms.error.file_invalid_audio";
    pub const FILE_INVALID_IMAGE: &str = "forms.error.file_invalid_image";
    pub const FILE_INVALID_VIDEO: &str = "forms.error.file_invalid_video";
    pub const FILE_MAX_COUNT: &str = "forms.error.file_max_count";
    pub const FILE_REQUIRED: &str = "forms.error.file_required";
    pub const FILE_TOO_LARGE: &str = "forms.error.file_too_large";
    pub const FILE_UNREADABLE: &str = "forms.error.file_unreadable";
    pub const FILE_WRITE: &str = "forms.error.file_write";
    pub const FINALIZE: &str = "forms.error.finalize";
    pub const HASH: &str = "forms.error.hash";
    pub const IMAGE_TOO_TALL: &str = "forms.error.image_too_tall";
    pub const IMAGE_TOO_WIDE: &str = "forms.error.image_too_wide";
    pub const INTEGER_DIGITS_MAX: &str = "forms.error.integer_digits_max";
    pub const INTEGER_REQUIRED: &str = "forms.error.integer_required";
    pub const IP_INVALID: &str = "forms.error.ip_invalid";
    pub const IPV4_ONLY: &str = "forms.error.ipv4_only";
    pub const IPV6_ONLY: &str = "forms.error.ipv6_only";
    pub const JSON_INVALID: &str = "forms.error.json_invalid";
    pub const MAX_SIZE_ABOVE_MODEL: &str = "forms.error.max_size_above_model";
    pub const MAX_SIZE_UNSUPPORTED: &str = "forms.error.max_size_unsupported";
    pub const MAX_VALUE: &str = "forms.error.max_value";
    pub const MEDIA_ROOT_FAILED: &str = "forms.error.media_root_failed";
    pub const MIN_VALUE: &str = "forms.error.min_value";
    pub const MULTIPART: &str = "forms.error.multipart";
    pub const MULTIPART_STREAM: &str = "forms.error.multipart_stream";
    pub const MUST_BE_CHECKED: &str = "forms.error.must_be_checked";
    pub const NUMBER_INVALID: &str = "forms.error.number_invalid";
    pub const NUMBER_REQUIRED: &str = "forms.error.number_required";
    pub const PASSWORD_EMPTY: &str = "forms.error.password_empty";
    pub const PASSWORD_EMPTY_REFUSED: &str = "forms.error.password_empty_refused";
    pub const PASSWORD_LOOKS_HASHED: &str = "forms.error.password_looks_hashed";
    pub const PHONE_INVALID: &str = "forms.error.phone_invalid";
    pub const PRECISION_MAX: &str = "forms.error.precision_max";
    pub const PRECISION_MIN: &str = "forms.error.precision_min";
    pub const PUBLISH_FAILED: &str = "forms.error.publish_failed";
    pub const PUBLISH_FAILED_AT: &str = "forms.error.publish_failed_at";
    pub const REMOVE_STAGED_FAILED: &str = "forms.error.remove_staged_failed";
    pub const RENDER_ERRORS: &str = "forms.error.render_errors";
    pub const RENDER_JS: &str = "forms.error.render_js";
    pub const REQUIRED: &str = "forms.error.required";
    pub const SELECT_REQUIRED: &str = "forms.error.select_required";
    pub const SLUG_INVALID: &str = "forms.error.slug_invalid";
    pub const SLUG_NO_DASH: &str = "forms.error.slug_no_dash";
    pub const SLUG_UNICODE_INVALID: &str = "forms.error.slug_unicode_invalid";
    pub const STAGED_UPLOAD_FAILED: &str = "forms.error.staged_upload_failed";
    pub const TEMPLATE_MISSING: &str = "forms.error.template_missing";
    pub const TERA_NOT_CONFIGURED: &str = "forms.error.tera_not_configured";
    pub const TEXT_FIELD_TOO_LARGE: &str = "forms.error.text_field_too_large";
    pub const TIME_INVALID: &str = "forms.error.time_invalid";
    pub const TIME_TOO_FAR: &str = "forms.error.time_too_far";
    pub const TIME_TOO_OLD: &str = "forms.error.time_too_old";
    pub const TOO_LONG: &str = "forms.error.too_long";
    pub const TOO_SHORT: &str = "forms.error.too_short";
    pub const UNIQUE_CONSTRAINT_VIOLATED: &str = "forms.error.unique_constraint_violated";
    pub const UNIQUE_FIELD_TAKEN: &str = "forms.error.unique_field_taken";
    pub const UNIQUE_VALUE_TAKEN: &str = "forms.error.unique_value_taken";
    pub const UPLOAD_DESTINATION_INVALID: &str = "forms.error.upload_destination_invalid";
    pub const UPLOAD_DIR: &str = "forms.error.upload_dir";
    pub const UPLOAD_DIR_FAILED: &str = "forms.error.upload_dir_failed";
    pub const UPLOAD_ESCAPES_ROOT: &str = "forms.error.upload_escapes_root";
    pub const UPLOAD_FAILED: &str = "forms.error.upload_failed";
    pub const UPLOAD_NO_LONGER_STAGED: &str = "forms.error.upload_no_longer_staged";
    pub const UPLOAD_NOT_A_DIRECTORY: &str = "forms.error.upload_not_a_directory";
    pub const UPLOAD_NOT_REAL_DIRECTORY: &str = "forms.error.upload_not_real_directory";
    pub const UPLOAD_NOT_REGULAR_FILE: &str = "forms.error.upload_not_regular_file";
    pub const UPLOAD_NOT_STAGED: &str = "forms.error.upload_not_staged";
    pub const UPLOAD_TO_INVALID: &str = "forms.error.upload_to_invalid";
    pub const UPLOAD_TOO_LARGE: &str = "forms.error.upload_too_large";
    pub const URL_INVALID: &str = "forms.error.url_invalid";
    pub const UUID_INVALID: &str = "forms.error.uuid_invalid";
    pub const VALIDATION_OVERFLOW: &str = "forms.error.validation_overflow";
    pub const VALUE_UNCONVERTIBLE: &str = "forms.error.value_unconvertible";
}
pub mod hint {
    pub const IP: &str = "forms.hint.ip";
    pub const IP_V4: &str = "forms.hint.ip_v4";
    pub const IP_V6: &str = "forms.hint.ip_v6";
    pub const JSON: &str = "forms.hint.json";
    pub const SLUG: &str = "forms.hint.slug";
    pub const UUID: &str = "forms.hint.uuid";
}
pub mod log {
    pub const ANTI_BOT_FILLED: &str = "forms.log.anti_bot_filled";
    pub const ANTI_BOT_READ_NAME: &str = "forms.log.anti_bot_read_name";
    pub const BINARY_FIELD_STAGED_UPLOAD_REMOVAL_FAILED: &str =
        "forms.log.binary_field_staged_upload_removal_failed";
    pub const BINARY_FIELD_STAGED_UPLOAD_SIZE_UNREADABLE: &str =
        "forms.log.binary_field_staged_upload_size_unreadable";
    pub const BINARY_FIELD_SUBMITTED_VALUE_NOT_STAGED: &str =
        "forms.log.binary_field_submitted_value_not_staged";
    pub const BINARY_FIELD_VALUE_NO_LONGER_STAGED: &str =
        "forms.log.binary_field_value_no_longer_staged";
    pub const BOOLEAN_FIELD_REFUSED: &str = "forms.log.boolean_field_refused";
    pub const CLEANED_CONVERSION_FAILED: &str = "forms.log.cleaned_conversion_failed";
    pub const CLEANUP_FILES_REMOVE_FAILED: &str = "forms.log.cleanup_files_remove_failed";
    pub const FIELD_REGISTERED: &str = "forms.log.field_registered";
    pub const FIELD_RENDER_FAILED: &str = "forms.log.field_render_failed";
    pub const FILE_FIELD_SUBMITTED_VALUE_NOT_STAGED: &str =
        "forms.log.file_field_submitted_value_not_staged";
    pub const FILE_FIELD_UPLOAD_SIZE_UNREADABLE_REFUSED: &str =
        "forms.log.file_field_upload_size_unreadable_refused";
    pub const FILE_SENT_FIELD_THAT_TAKES_NONE: &str = "forms.log.file_sent_field_that_takes_none";
    pub const FINALIZE_ERROR: &str = "forms.log.finalize_error";
    pub const FINALIZE_OK: &str = "forms.log.finalize_ok";
    pub const FORM_DATABASE_ERROR: &str = "forms.log.form_database_error";
    pub const FORM_RENDER_FAILED: &str = "forms.log.form_render_failed";
    pub const FORM_WRITE_FAILED: &str = "forms.log.form_write_failed";
    pub const IMAGE_FIELD_DIMENSION_READ_TASK_FAILED: &str =
        "forms.log.image_field_dimension_read_task_failed";
    pub const IMAGE_FIELD_DIMENSIONS_UNREADABLE_REFUSED: &str =
        "forms.log.image_field_dimensions_unreadable_refused";
    pub const MEDIA_FIELD_SUBMITTED_VALUE_NOT_STAGED: &str =
        "forms.log.media_field_submitted_value_not_staged";
    pub const PUBLISHED_UPLOAD_ROLLBACK_FAILED: &str = "forms.log.published_upload_rollback_failed";
    pub const REMOVE_OVERSIZED: &str = "forms.log.remove_oversized";
    pub const REMOVE_REPLACED: &str = "forms.log.remove_replaced";
    pub const RENDER_ERROR: &str = "forms.log.render_error";
    pub const RENDER_START: &str = "forms.log.render_start";
    pub const RENDERED_OK: &str = "forms.log.rendered_ok";
    pub const SET_VALUE: &str = "forms.log.set_value";
    pub const SET_VALUE_CHECKBOX_ABSENT_FALSE: &str = "forms.log.set_value_checkbox_absent_false";
    pub const SET_VALUE_HIDDEN: &str = "forms.log.set_value_hidden";
    pub const SKIPPING_JS_FILE: &str = "forms.log.skipping_js_file";
    pub const STAGED_UPLOAD_DIRECTORY_NOT_REMOVED: &str =
        "forms.log.staged_upload_directory_not_removed";
    pub const UNSUPPORTED_COLUMN: &str = "forms.log.unsupported_column";
    pub const UPLOAD_NOT_WRITTEN: &str = "forms.log.upload_not_written";
    pub const VALIDATE: &str = "forms.log.validate";
    pub const VALIDATE_FIELDS_RESULT: &str = "forms.log.validate_fields_result";
    pub const VALIDATION_FAILED: &str = "forms.log.validation_failed";
}
