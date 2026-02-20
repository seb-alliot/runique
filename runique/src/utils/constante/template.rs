use crate::tpls;

pub const SIMPLE_TEMPLATES: &[(&str, &str)] = tpls![
    ("base_index", "runique_index/base_index.html"),
    ("message", "message/message.html"),
    ("404", "errors/404.html"),
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
];

pub const ADMIN_TEMPLATES: &[(&str, &str)] = tpls![
    ("admin_base.html", "admin/admin_base.html"),
    ("login", "admin/composant/admin_login.html"),
    ("dashboard", "admin/composant/admin_dashboard.html"),
    ("list", "admin/composant/admin_list.html"),
    ("form", "admin/composant/admin_form.html"),
    ("detail", "admin/composant/admin_detail.html"),
    ("delete", "admin/composant/admin_delete.html"),
];
