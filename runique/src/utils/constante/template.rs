// templates.rs
pub const SIMPLE_TEMPLATES: &[(&str, &str)] = &[
    (
        "base_index",
        include_str!("../../../templates/runique_index/base_index.html"),
    ),
    (
        "message",
        include_str!("../../../templates/message/message.html"),
    ),
    ("404", include_str!("../../../templates/errors/404.html")),
    ("500", include_str!("../../../templates/errors/500.html")),
    (
        "debug",
        include_str!("../../../templates/errors/debug_error.html"),
    ),
    ("csrf", include_str!("../../../templates/csrf/csrf.html")),
    ("csp", include_str!("../../../templates/csp/csp.html")),
];

pub const ERROR_CORPS: &[(&str, &str)] = &[
    (
        "errors/corps-error/header-error.html",
        include_str!("../../../templates/errors/corps-error/header-error.html"),
    ),
    (
        "errors/corps-error/template-info.html",
        include_str!("../../../templates/errors/corps-error/template-info.html"),
    ),
    (
        "errors/corps-error/stack-trace-error.html",
        include_str!("../../../templates/errors/corps-error/stack-trace-error.html"),
    ),
    (
        "errors/corps-error/request-info.html",
        include_str!("../../../templates/errors/corps-error/request-info.html"),
    ),
    (
        "errors/corps-error/environment-info.html",
        include_str!("../../../templates/errors/corps-error/environment-info.html"),
    ),
    (
        "errors/corps-error/footer-error.html",
        include_str!("../../../templates/errors/corps-error/footer-error.html"),
    ),
];

pub const FIELD_TEMPLATES: &[(&str, &str)] = &[
    (
        "base_boolean",
        include_str!("../../../templates/field_html/base_boolean.html"),
    ),
    (
        "base_checkbox",
        include_str!("../../../templates/field_html/base_checkbox.html"),
    ),
    (
        "base_color",
        include_str!("../../../templates/field_html/base_color.html"),
    ),
    (
        "base_datetime",
        include_str!("../../../templates/field_html/base_datetime.html"),
    ),
    (
        "base_file",
        include_str!("../../../templates/field_html/base_file.html"),
    ),
    (
        "base_number",
        include_str!("../../../templates/field_html/base_number.html"),
    ),
    (
        "base_radio",
        include_str!("../../../templates/field_html/base_radio.html"),
    ),
    (
        "base_select",
        include_str!("../../../templates/field_html/base_select.html"),
    ),
    (
        "base_special",
        include_str!("../../../templates/field_html/base_special.html"),
    ),
    (
        "base_string",
        include_str!("../../../templates/field_html/base_string.html"),
    ),
];
