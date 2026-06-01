//! Built-in framework template constants (errors, base, CSRF, admin…).
use crate::tpls;

pub const SIMPLE_TEMPLATES: &[(&str, &str)] = tpls![
    ("base_index.html", "runique_index/base_index.html"),
    ("message.html", "message/message.html"),
    ("404.html", "errors/404.html"),
    ("429.html", "errors/429.html"),
    ("500.html", "errors/500.html"),
    ("debug.html", "errors/debug_error.html"),
    ("csrf.html", "csrf/csrf.html"),
    ("csp.html", "csp/csp.html"),
    ("js_files.html", "asset/js.html"),
];

pub const ERROR_CORPS: &[(&str, &str)] = tpls![
    ("header-error.html", "errors/corps-error/header-error.html"),
    (
        "template-info.html",
        "errors/corps-error/template-info.html"
    ),
    (
        "trace-error.html",
        "errors/corps-error/stack-trace-error.html"
    ),
    ("request-info.html", "errors/corps-error/request-info.html"),
    ("zone-info.html", "errors/corps-error/zone-info.html"),
    ("footer-error.html", "errors/corps-error/footer-error.html"),
];

pub const FIELD_TEMPLATES: &[(&str, &str)] = tpls![
    ("base_boolean.html", "field_html/base_boolean.html"),
    ("base_checkbox.html", "field_html/base_checkbox.html"),
    ("base_color.html", "field_html/base_color.html"),
    ("base_datetime.html", "field_html/base_datetime.html"),
    ("base_file.html", "field_html/base_file.html"),
    ("base_number.html", "field_html/base_number.html"),
    ("base_radio.html", "field_html/base_radio.html"),
    ("base_select.html", "field_html/base_select.html"),
    ("base_special.html", "field_html/base_special.html"),
    ("base_string.html", "field_html/base_string.html"),
    ("base_hidden.html", "field_html/base_hidden.html"),
    ("base_honeypot.html", "field_html/base_honeypot.html"),
];

pub const AUTH_TEMPLATES: &[(&str, &str)] = tpls![
    ("auth/forgot_password.html", "auth/forgot_password.html"),
    ("auth/reset_password.html", "auth/reset_password.html"),
];

pub const ADMIN_TEMPLATES: &[(&str, &str)] = tpls![
    ("admin/admin_template.html", "admin/admin_template.html"),
    ("admin_base.html", "admin/admin_base.html"),
    ("admin/login.html", "admin/composant/login.html"),
    ("admin/dashboard.html", "admin/composant/dashboard.html"),
    ("admin/list.html", "admin/composant/list.html"),
    (
        "admin/list_partial.html",
        "admin/composant/list_partial.html"
    ),
    ("admin/create.html", "admin/composant/create.html"),
    ("admin/edit.html", "admin/composant/edit.html"),
    ("admin/detail.html", "admin/composant/detail.html"),
    ("admin/delete.html", "admin/composant/delete.html"),
    ("admin/kebab.html", "admin/composant/kebab.html"),
    ("admin/history.html", "admin/composant/history.html"),
    (
        "admin/history_diff.html",
        "admin/composant/history_diff.html"
    ),
    (
        "admin/history_timeline.html",
        "admin/composant/history_timeline.html"
    ),
    (
        "admin/history_batch.html",
        "admin/composant/history_batch.html"
    ),
    ("admin/bulk_edit.html", "admin/composant/bulk_edit.html"),
    (
        "admin/reset_password_email.html",
        "admin/reset_password_email.html"
    ),
    (
        "admin/user_created_email.html",
        "admin/user_created_email.html"
    ),
];
