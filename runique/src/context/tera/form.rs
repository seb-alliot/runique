//! Tera `form` filter — HTML rendering of a form field by name from context.
// Dans src/tera_function/form_filter.rs
use tera::{Filter, Kwargs, State, TeraResult, Value};

/// Renders a form, or a single field of it, from the serialized form in context.
///
/// Declares `is_safe()`: every string it returns is HTML produced by Runique's own
/// field renderer, never raw user input. This replaces the `| safe` the template
/// preprocessor used to append to `{% form.x %}`.
pub struct FormFilter;

impl Filter<&Value, TeraResult<Value>> for FormFilter {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, value: &Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let Some(field_name) = kwargs.get::<&str>("field")? else {
            // Render full form
            return render_form_html(value);
        };

        // Reserved accessor `{% form.x.js %}` — the form's auto-collected <script>
        // block (real CSP nonce + resolved static URLs), pre-rendered once by
        // `renderer::render_js`. Django `form.media` style: in field-by-field
        // rendering the dev places it explicitly. Full-form rendering already
        // embeds it, so stateless filters never have to guess a last-field anchor.
        if field_name == "js" {
            return Ok(Value::safe_string(
                &render_scripts(value).unwrap_or_default(),
            ));
        }

        let mut output = render_field(value, field_name)?;

        // Inject CSRF before the first field
        if is_first_field_by_index(value, field_name)
            && let Some(csrf) = render_csrf(value)
        {
            output = format!("{}\n{}", csrf, output);
        }

        // Inject honeypot after the last field
        if is_last_field_by_index(value, field_name)
            && let Some(hp) = get_honeypot_html(value)
        {
            output = format!("{}\n{}", output, hp);
        }

        Ok(Value::safe_string(&output))
    }
}

/// Single lookup path for the whole filter: the value it receives is either the
/// serialized form itself, or a struct wrapping it in a `form` field, so every key
/// is tried at both levels. Collapsed into one helper so the two shapes can never
/// drift apart across call sites.
///
/// `get_from_path` splits on `.`, so `key` must be a single, dot-free segment.
/// All call sites pass either a literal or a field name, which the template
/// preprocessor constrains to `[a-zA-Z0-9_]+`.
fn lookup<'v>(value: &'v Value, key: &'v str) -> Option<&'v Value> {
    value.get_from_path(key).or_else(|| {
        value
            .get_from_path("form")
            .and_then(|f| f.get_from_path(key))
    })
}

/// Reads `index` on a serialized field.
fn field_index(field: &Value) -> Option<u64> {
    field.get_from_path("index").and_then(|idx| idx.as_u64())
}

/// Checks if the field is the first by index (excluding csrf_token)
fn is_first_field_by_index(value: &Value, field_name: &str) -> bool {
    let Some(fields) = lookup(value, "fields") else {
        return false;
    };
    let Some(fields_obj) = fields.as_map() else {
        return false;
    };

    // Minimum index excluding csrf_token
    let min_index = fields_obj
        .values()
        .filter(|f| f.get_from_path("name").and_then(|n| n.as_str()) != Some("csrf_token"))
        .filter_map(field_index)
        .min();

    let current_index = fields.get_from_path(field_name).and_then(field_index);

    match (min_index, current_index) {
        (Some(min), Some(current)) => min == current,
        _ => false,
    }
}

/// Checks if the field is the last by index
fn is_last_field_by_index(value: &Value, field_name: &str) -> bool {
    let Some(fields) = lookup(value, "fields") else {
        return false;
    };
    let Some(fields_obj) = fields.as_map() else {
        return false;
    };

    // Find maximum index (last field)
    let max_index = fields_obj.values().filter_map(field_index).max();

    // Get current field index
    let current_index = fields.get_from_path(field_name).and_then(field_index);

    match (max_index, current_index) {
        (Some(max), Some(current)) => max == current,
        _ => false,
    }
}

/// Retrieves the CSRF field HTML from rendered_fields
fn render_csrf(value: &Value) -> Option<String> {
    find_rendered_fields(value)?
        .get_from_path("csrf_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_honeypot_html(value: &Value) -> Option<String> {
    lookup(value, "honeypot_html")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Source of the `{% form.x.js %}` accessor: the form's pre-rendered `<script>`
/// block (real CSP nonce + resolved static URLs), produced once by
/// `renderer::render_js` via the `js.html` template and serialized as `rendered_js`.
/// Read as-is — never rebuilt here with literal `{% csp %}` / `{% static %}` tags:
/// the load-time preprocessor never sees this runtime string, so `| safe` would
/// ship the raw tags to the browser.
fn render_scripts(value: &Value) -> Option<String> {
    lookup(value, "rendered_js")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn render_field(value: &Value, field_name: &str) -> TeraResult<String> {
    if let Some(fields) = find_rendered_fields(value)
        && let Some(html) = fields.get_from_path(field_name).and_then(|v| v.as_str())
    {
        return Ok(html.to_string());
    }

    Err(tera::Error::message(format!(
        "Field '{}' not found in form",
        field_name
    )))
}

fn find_rendered_fields(value: &Value) -> Option<&Value> {
    lookup(value, "rendered_fields")
}

/// Renders the complete form HTML
fn render_form_html(value: &Value) -> TeraResult<Value> {
    let Some(html) = find_html(value) else {
        return Err(tera::Error::message(
            "Cannot render form: no 'html' field found",
        ));
    };

    match get_honeypot_html(value) {
        Some(hp) => Ok(Value::safe_string(&format!("{}\n{}", html, hp))),
        None => Ok(Value::safe_string(&html)),
    }
}

/// Intelligently searches for the "html" field
fn find_html(value: &Value) -> Option<String> {
    // 1. `value.html`, or `value.form.html` when the filter is applied on a struct
    //    wrapping the form
    if let Some(html) = lookup(value, "html").and_then(|v| v.as_str()) {
        return Some(html.to_string());
    }

    // 2. Rebuild from rendered_fields if html is absent
    if let Some(fields) = find_rendered_fields(value)
        && let Some(obj) = fields.as_map()
    {
        let html: Vec<&str> = obj.values().filter_map(|v| v.as_str()).collect();
        if !html.is_empty() {
            return Some(html.join("\n"));
        }
    }

    None
}
