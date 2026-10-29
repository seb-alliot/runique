//! Built-in framework template constants (errors, base, CSRF, admin…).
use crate::tpls;

/// Built-in templates with no dependency on other internal templates —
/// `(tera_name, embedded_content)` pairs, content embedded at compile time via
/// `include_str!` (see the `tpls!` macro). Registered into Tera by
/// `TemplateLoader::internal_templates`.
pub const SIMPLE_TEMPLATES: &[(&str, &str)] = tpls![
    ("base_index.html", "runique_index/base_index.html"),
    ("message.html", "message/message.html"),
    ("400.html", "errors/400.html"),
    ("404.html", "errors/404.html"),
    ("429.html", "errors/429.html"),
    ("500.html", "errors/500.html"),
    ("503.html", "errors/503.html"),
    ("debug.html", "errors/debug_error.html"),
    ("csrf.html", "csrf/csrf.html"),
    ("csp.html", "csp/csp.html"),
    ("js_files.html", "asset/js.html"),
];

/// Partials composing the built-in error page body (`errors/corps-error/`),
/// `(tera_name, embedded_content)` pairs — see [`SIMPLE_TEMPLATES`].
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

/// Per-type field renderers used by the forms renderer (one per `FormField`
/// widget kind), `(tera_name, embedded_content)` pairs — see [`SIMPLE_TEMPLATES`].
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
    ("form_errors.html", "field_html/form_errors.html"),
    ("base_form_name.html", "field_html/base_form_name.html"),
];

/// Built-in authentication pages (forgot/reset password), `(tera_name,
/// embedded_content)` pairs — see [`SIMPLE_TEMPLATES`].
pub const AUTH_TEMPLATES: &[(&str, &str)] = tpls![
    ("auth/forgot_password.html", "auth/forgot_password.html"),
    ("auth/reset_password.html", "auth/reset_password.html"),
];

/// Built-in admin panel templates (login, dashboard, CRUD views, history,
/// notification emails…), `(tera_name, embedded_content)` pairs — see
/// [`SIMPLE_TEMPLATES`].
pub const ADMIN_TEMPLATES: &[(&str, &str)] = tpls![
    // Layout: the page frame every admin page extends.
    (
        "admin/admin_template.html",
        "admin/layout/admin_template.html"
    ),
    ("admin_base.html", "admin/layout/admin_base.html"),
    // Pages: what a route renders, assembled from blocks.
    ("admin/login.html", "admin/pages/login.html"),
    ("admin/dashboard.html", "admin/pages/dashboard.html"),
    ("admin/list.html", "admin/pages/list.html"),
    ("admin/create.html", "admin/pages/create.html"),
    ("admin/edit.html", "admin/pages/edit.html"),
    ("admin/detail.html", "admin/pages/detail.html"),
    ("admin/delete.html", "admin/pages/delete.html"),
    ("admin/history.html", "admin/pages/history.html"),
    ("admin/history_diff.html", "admin/pages/history_diff.html"),
    ("admin/rights.html", "admin/pages/rights.html"),
    ("admin/transfer.html", "admin/pages/transfer.html"),
    // Blocks: named by their path, so a project replaces one by putting a
    // file at the same path in its own templates.
    (
        "admin/blocks/layout/sidebar.html",
        "admin/blocks/layout/sidebar.html"
    ),
    (
        "admin/blocks/layout/stylesheets.html",
        "admin/blocks/layout/stylesheets.html"
    ),
    (
        "admin/blocks/layout/topbar_actions.html",
        "admin/blocks/layout/topbar_actions.html"
    ),
    (
        "admin/list_partial.html",
        "admin/blocks/list/list_partial.html"
    ),
    (
        "admin/blocks/list/search.html",
        "admin/blocks/list/search.html"
    ),
    ("admin/blocks/list/bulk.html", "admin/blocks/list/bulk.html"),
    (
        "admin/blocks/list/table.html",
        "admin/blocks/list/table.html"
    ),
    (
        "admin/blocks/list/pagination.html",
        "admin/blocks/list/pagination.html"
    ),
    (
        "admin/blocks/list/filters.html",
        "admin/blocks/list/filters.html"
    ),
    (
        "admin/blocks/detail/actions.html",
        "admin/blocks/detail/actions.html"
    ),
    (
        "admin/blocks/detail/fields.html",
        "admin/blocks/detail/fields.html"
    ),
    (
        "admin/blocks/detail/groups.html",
        "admin/blocks/detail/groups.html"
    ),
    (
        "admin/blocks/detail/transfer.html",
        "admin/blocks/detail/transfer.html"
    ),
    (
        "admin/blocks/dashboard/table.html",
        "admin/blocks/dashboard/table.html"
    ),
    (
        "admin/blocks/dashboard/my_groups.html",
        "admin/blocks/dashboard/my_groups.html"
    ),
    (
        "admin/blocks/form/fields.html",
        "admin/blocks/form/fields.html"
    ),
    (
        "admin/blocks/form/actions.html",
        "admin/blocks/form/actions.html"
    ),
    (
        "admin/blocks/delete/warning.html",
        "admin/blocks/delete/warning.html"
    ),
    (
        "admin/blocks/delete/actions.html",
        "admin/blocks/delete/actions.html"
    ),
    (
        "admin/blocks/history/action_badge.html",
        "admin/blocks/history/action_badge.html"
    ),
    (
        "admin/blocks/history/filters.html",
        "admin/blocks/history/filters.html"
    ),
    (
        "admin/blocks/history/table.html",
        "admin/blocks/history/table.html"
    ),
    (
        "admin/blocks/rights/table.html",
        "admin/blocks/rights/table.html"
    ),
    (
        "admin/blocks/common/kebab.html",
        "admin/blocks/common/kebab.html"
    ),
    // Emails sent by the admin.
    (
        "admin/reset_password_email.html",
        "admin/emails/reset_password_email.html"
    ),
    (
        "admin/user_created_email.html",
        "admin/emails/user_created_email.html"
    ),
];

/// The admin blocks a developer may replace: the name a `{% dev_block %}`
/// gives, and Runique's template for it.
pub const ADMIN_BLOCKS: &[(&str, &str)] = &[
    ("list_table", "admin/blocks/list/table.html"),
    ("list_filters", "admin/blocks/list/filters.html"),
    ("list_search", "admin/blocks/list/search.html"),
    ("list_pagination", "admin/blocks/list/pagination.html"),
    ("list_bulk", "admin/blocks/list/bulk.html"),
    ("kebab", "admin/blocks/common/kebab.html"),
    ("detail_fields", "admin/blocks/detail/fields.html"),
    ("detail_actions", "admin/blocks/detail/actions.html"),
    ("detail_groups", "admin/blocks/detail/groups.html"),
    ("detail_transfer", "admin/blocks/detail/transfer.html"),
    ("form_fields", "admin/blocks/form/fields.html"),
    ("form_actions", "admin/blocks/form/actions.html"),
    ("delete_warning", "admin/blocks/delete/warning.html"),
    ("delete_actions", "admin/blocks/delete/actions.html"),
    ("sidebar", "admin/blocks/layout/sidebar.html"),
    ("topbar", "admin/blocks/layout/topbar_actions.html"),
    ("dashboard_table", "admin/blocks/dashboard/table.html"),
    ("dashboard_groups", "admin/blocks/dashboard/my_groups.html"),
    ("history_table", "admin/blocks/history/table.html"),
    ("history_filters", "admin/blocks/history/filters.html"),
    (
        "history_action_badge",
        "admin/blocks/history/action_badge.html",
    ),
    ("rights_table", "admin/blocks/rights/table.html"),
];

/// Runique's template for the admin block named `name`.
pub fn admin_block(name: &str) -> Option<&'static str> {
    ADMIN_BLOCKS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, path)| *path)
}
