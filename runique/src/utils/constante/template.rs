//! Built-in framework template constants (errors, base, CSRF, admin…).
use crate::tpls;

pub const SIMPLE_TEMPLATES: &[(&str, &str)] = tpls![
    ("base_index", "runique_index/base_index.html"),
    ("message", "message/message.html"),
    ("404", "errors/404.html"),
    ("429", "errors/429.html"),
    ("500", "errors/500.html"),
    ("debug", "errors/debug_error.html"),
    ("csrf", "csrf/csrf.html"),
    ("csp", "csp/csp.html"),
    ("js_files", "asset/js.html"),
];

pub const ERROR_CORPS: &[(&str, &str)] = tpls![
    ("header-error", "errors/corps-error/header-error.html"),
    ("template-info", "errors/corps-error/template-info.html"),
    ("trace-error", "errors/corps-error/stack-trace-error.html"),
    ("request-info", "errors/corps-error/request-info.html"),
    ("zone-info", "errors/corps-error/zone-info.html"),
    ("footer-error", "errors/corps-error/footer-error.html"),
];

pub const FIELD_TEMPLATES: &[(&str, &str)] = tpls![
    ("base_boolean", "field_html/base_boolean.html"),
    ("base_checkbox", "field_html/base_checkbox.html"),
    ("base_color", "field_html/base_color.html"),
    ("base_datetime", "field_html/base_datetime.html"),
    ("base_file", "field_html/base_file.html"),
    ("base_number", "field_html/base_number.html"),
    ("base_radio", "field_html/base_radio.html"),
    ("base_select", "field_html/base_select.html"),
    ("base_special", "field_html/base_special.html"),
    ("base_string", "field_html/base_string.html"),
    ("base_hidden", "field_html/base_hidden.html"),
];

pub const AUTH_TEMPLATES: &[(&str, &str)] = tpls![
    ("auth/forgot_password.html", "auth/forgot_password.html"),
    ("auth/reset_password.html", "auth/reset_password.html"),
];

pub const ADMIN_TEMPLATES: &[(&str, &str)] = tpls![
    ("admin/admin_template", "admin/admin_template.html"),
    ("admin_base", "admin/admin_base.html"),
    ("admin/login", "admin/composant/login.html"),
    ("admin/dashboard", "admin/composant/dashboard.html"),
    ("admin/list", "admin/composant/list.html"),
    ("admin/list_partial", "admin/composant/list_partial.html"),
    ("admin/create", "admin/composant/create.html"),
    ("admin/edit", "admin/composant/edit.html"),
    ("admin/detail", "admin/composant/detail.html"),
    ("admin/delete", "admin/composant/delete.html"),
    ("admin/kebab", "admin/composant/kebab.html"),
    ("admin/history", "admin/composant/history.html"),
    ("admin/history_diff", "admin/composant/history_diff.html"),
    (
        "admin/history_timeline",
        "admin/composant/history_timeline.html"
    )
];
