# Runique Admin — Tera Variables per View

This document lists all variables available in the **Tera context** when a developer overrides an admin template.

---

## Global Variables (All Views)

These variables are injected into **every admin view** via `inject_common_context`.

| Variable           | Type                | Description                                  |
| ------------------ | ------------------- | -------------------------------------------- |
| `site_title`       | `String`            | Site title configured in `AdminConfig`       |
| `resource_key`     | `&str`              | Key of the current resource (e.g. `"user"`)  |
| `current_resource` | `&str`              | Same as `resource_key`                       |
| `resource`         | `ResourceMeta`      | Metadata of the current resource (see below) |
| `resources`        | `Vec<ResourceMeta>` | List of all registered resources             |
| `csrf_token`       | `String`            | CSRF token to include in POST forms          |
| `static_runique`   | `String`            | URL for Runique static assets                |
| `lang`             | `String`            | Current language code (e.g. `"fr"`)          |
| `debug`            | `bool`              | Whether debug mode is enabled                |

### `ResourceMeta` Structure

| Field                       | Type                     | Description                            |
| --------------------------- | ------------------------ | -------------------------------------- |
| `resource.key`              | `&str`                   | Unique resource key                    |
| `resource.title`            | `&str`                   | Human-readable resource title          |
| `resource.permissions.list` | `Vec<String>`            | Roles required to access this resource |
| `resource.extra_context`    | `HashMap<String, Value>` | Custom context declared in `admin!{}`  |

---

## Global i18n Variables (All Views)

Automatically injected via `insert_admin_messages`. All keys follow the pattern `admin_{section}_{key}`.

### `base` Section

| Tera Variable             | i18n Key                  |
| ------------------------- | ------------------------- |
| `admin_base_title`        | `admin.base.title`        |
| `admin_base_breadcrumb`   | `admin.base.breadcrumb`   |
| `admin_base_toggle`       | `admin.base.toggle`       |
| `admin_base_logout_title` | `admin.base.logout_title` |

### `list` Section

| Tera Variable                 | i18n Key                      |
| ----------------------------- | ----------------------------- |
| `admin_list_breadcrumb_admin` | `admin.list.breadcrumb_admin` |
| `admin_list_entries_count`    | `admin.list.entries_count`    |
| `admin_list_btn_create`       | `admin.list.btn_create`       |
| `admin_list_th_id`            | `admin.list.th_id`            |
| `admin_list_th_actions`       | `admin.list.th_actions`       |
| `admin_list_bool_true`        | `admin.list.bool_true`        |
| `admin_list_bool_false`       | `admin.list.bool_false`       |
| `admin_list_btn_detail`       | `admin.list.btn_detail`       |
| `admin_list_btn_edit`         | `admin.list.btn_edit`         |
| `admin_list_btn_delete`       | `admin.list.btn_delete`       |
| `admin_list_confirm_delete`   | `admin.list.confirm_delete`   |
| `admin_list_empty_title`      | `admin.list.empty_title`      |
| `admin_list_empty_desc`       | `admin.list.empty_desc`       |
| `admin_list_btn_create_first` | `admin.list.btn_create_first` |

### `create` Section

| Tera Variable             | i18n Key                  |
| ------------------------- | ------------------------- |
| `admin_create_title`      | `admin.create.title`      |
| `admin_create_breadcrumb` | `admin.create.breadcrumb` |
| `admin_create_card_info`  | `admin.create.card_info`  |
| `admin_create_no_fields`  | `admin.create.no_fields`  |
| `admin_create_btn_cancel` | `admin.create.btn_cancel` |
| `admin_create_btn_submit` | `admin.create.btn_submit` |

### `edit` Section

| Tera Variable           | i18n Key                |
| ----------------------- | ----------------------- |
| `admin_edit_title`      | `admin.edit.title`      |
| `admin_edit_breadcrumb` | `admin.edit.breadcrumb` |
| `admin_edit_card_info`  | `admin.edit.card_info`  |
| `admin_edit_no_fields`  | `admin.edit.no_fields`  |
| `admin_edit_btn_cancel` | `admin.edit.btn_cancel` |
| `admin_edit_btn_submit` | `admin.edit.btn_submit` |

### `detail` Section

| Tera Variable                 | i18n Key                      |
| ----------------------------- | ----------------------------- |
| `admin_detail_title`          | `admin.detail.title`          |
| `admin_detail_breadcrumb`     | `admin.detail.breadcrumb`     |
| `admin_detail_entry_label`    | `admin.detail.entry_label`    |
| `admin_detail_btn_list`       | `admin.detail.btn_list`       |
| `admin_detail_btn_edit`       | `admin.detail.btn_edit`       |
| `admin_detail_btn_delete`     | `admin.detail.btn_delete`     |
| `admin_detail_confirm_delete` | `admin.detail.confirm_delete` |

### `delete` Section

| Tera Variable                       | i18n Key                            |
| ----------------------------------- | ----------------------------------- |
| `admin_delete_title`                | `admin.delete.title`                |
| `admin_delete_breadcrumb`           | `admin.delete.breadcrumb`           |
| `admin_delete_heading`              | `admin.delete.heading`              |
| `admin_delete_btn_cancel`           | `admin.delete.btn_cancel`           |
| `admin_delete_btn_confirm`          | `admin.delete.btn_confirm`          |
| `admin_delete_warning_title`        | `admin.delete.warning.title`        |
| `admin_delete_warning_desc`         | `admin.delete.warning.desc`         |
| `admin_delete_warning_of`           | `admin.delete.warning.of`           |
| `admin_delete_warning_irreversible` | `admin.delete.warning.irreversible` |

---

## `login` View

**Route:** `GET /admin/login`

| Variable                        | Type     | Description                          |
| ------------------------------- | -------- | ------------------------------------ |
| `admin_login_title`             | `String` | i18n `admin.login.title`             |
| `admin_login_subtitle`          | `String` | i18n `admin.login.subtitle`          |
| `admin_login_label_username`    | `String` | i18n `admin.login.label_username`    |
| `admin_login_label_password`    | `String` | i18n `admin.login.label_password`    |
| `admin_login_btn_submit`        | `String` | i18n `admin.login.btn_submit`        |
| `admin_login_error_session`     | `String` | i18n `admin.login.error_session`     |
| `admin_login_error_credentials` | `String` | i18n `admin.login.error_credentials` |
| `csrf_token`                    | `String` | CSRF token to include in the form    |

⚠️ The login view **does not have access** to the variables `resource`, `resources`, or `resource_key` — `inject_common_context` is not called.

---

## `dashboard` View

**Route:** `GET /admin/`

| Variable                         | Type                   | Description                                     |
| -------------------------------- | ---------------------- | ----------------------------------------------- |
| `resources`                      | `Vec<ResourceMeta>`    | All registered resources                        |
| `resource_counts`                | `HashMap<String, i64>` | Entry count per resource (key = `resource.key`) |
| `current_page`                   | `&str`                 | Value `"dashboard"`                             |
| `admin_dashboard_title`          | `String`               | i18n `admin.dashboard.title`                    |
| `admin_dashboard_subtitle`       | `String`               | i18n `admin.dashboard.subtitle`                 |
| `admin_dashboard_card_resources` | `String`               | i18n `admin.dashboard.card_resources`           |
| `admin_dashboard_th_resource`    | `String`               | i18n `admin.dashboard.th_resource`              |
| `admin_dashboard_th_key`         | `String`               | i18n `admin.dashboard.th_key`                   |
| `admin_dashboard_th_permissions` | `String`               | i18n `admin.dashboard.th_permissions`           |
| `admin_dashboard_th_actions`     | `String`               | i18n `admin.dashboard.th_actions`               |
| `admin_dashboard_btn_list`       | `String`               | i18n `admin.dashboard.btn_list`                 |
| `admin_dashboard_btn_create`     | `String`               | i18n `admin.dashboard.btn_create`               |
| `admin_dashboard_see_list`       | `String`               | i18n `admin.dashboard.see_list`                 |
| `admin_dashboard_empty_title`    | `String`               | i18n `admin.dashboard.empty_title`              |
| `admin_dashboard_empty_desc`     | `String`               | i18n `admin.dashboard.empty_desc`               |

---

## `list` View

**Route:** `GET /admin/{resource}/list`

| Variable       | Type         | Description                        |
| -------------- | ------------ | ---------------------------------- |
| `entries`      | `Vec<Value>` | List of records serialized as JSON |
| `total`        | `usize`      | Total number of entries            |
| `current_page` | `&str`       | Value `"list"`                     |

The i18n variables for the `list` section are defined in the global variables above.

---

## `create` View

**Route:** `GET /admin/{resource}/create`

| Variable      | Type         | Description                                                |
| ------------- | ------------ | ---------------------------------------------------------- |
| `form_fields` | `FormOutput` | Form generated by Prisme — accessed via `form_fields.html` |
| `is_edit`     | `bool`       | Value `false`                                              |

The i18n variables for the `create` section are defined in the global variables above.

---

## `edit` View

**Route:** `GET /admin/{resource}/{id}/edit`

| Variable      | Type         | Description                        |
| ------------- | ------------ | ---------------------------------- |
| `form_fields` | `FormOutput` | Form pre-filled with existing data |
| `is_edit`     | `bool`       | Value `true`                       |
| `object_id`   | `i32`        | ID of the entry being edited       |

The i18n variables for the `edit` section are defined in the global variables above.

---

## `detail` View

**Route:** `GET /admin/{resource}/{id}/detail`

| Variable    | Type    | Description                                                      |
| ----------- | ------- | ---------------------------------------------------------------- |
| `entry`     | `Value` | Record serialized as JSON (absent if `get_fn` is not configured) |
| `object_id` | `i32`   | Entry ID                                                         |

The i18n variables for the `detail` section are defined in the global variables above.

---

## `delete` View

**Route:** `GET /admin/{resource}/{id}/delete`

| Variable    | Type    | Description                                                      |
| ----------- | ------- | ---------------------------------------------------------------- |
| `entry`     | `Value` | Record serialized as JSON (absent if `get_fn` is not configured) |
| `object_id` | `i32`   | ID of the entry to delete                                        |

The i18n variables for the `delete` section are defined in the global variables above.

---

## Recommended Pattern for i18n Variables

All i18n variables should be used with the following pattern in Tera:

```html
{% if admin_create_title %}{{ admin_create_title }}{% endif %}
```

Do not use `| default(value="...")` — i18n variables are always present when a language is configured.

---

## Overriding a Template

Using `.templates()` in the builder replaces the template for **all resources**:

```rust
RuniqueApp::builder(config)
    .with_admin(|a| a
        .templates(|t| t
            .with_list("my_theme/list")
            .with_create("my_theme/create")
            .with_edit("my_theme/edit")
            .with_detail("my_theme/detail")
            .with_delete("my_theme/delete")
            .with_dashboard("my_theme/dashboard")
            .with_login("my_theme/login")
            .with_base("my_theme/admin_base")
        )
    )
    .build().await?
```

Overridden templates have access to the **same variables as the default templates**.
