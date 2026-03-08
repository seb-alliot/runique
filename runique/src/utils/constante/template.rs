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
    ("admin/admin_base_contract", "admin/admin_base_contract.html"),
    ("admin_base", "admin/admin_base.html"),

    ("login", "admin/composant/login.html"),
    ("dashboard", "admin/composant/dashboard.html"),
    ("list", "admin/composant/list.html"),
    ("create", "admin/composant/create.html"),
    ("edit", "admin/composant/edit.html"),
    ("detail", "admin/composant/detail.html"),
    ("delete", "admin/composant/delete.html"),
];
