# Template rendering

[← Database errors](/docs/en/formulaire/errors)

---

## Full form

```html
<form method="post">
    {% form.register_form %}
    <button type="submit">Sign up</button>
</form>
```

Automatically renders: all fields, labels, validation errors, CSRF token, and JS scripts.

---

## Field by field

```html
<form method="post">
    {% csrf %}
    <div class="row">
        <div class="col-6">{% form.register_form.username %}</div>
        <div class="col-6">{% form.register_form.email %}</div>
    </div>
    {% form.register_form.password %}
    <button type="submit">Sign up</button>
</form>
```

---

## Global errors

```html
{% if register_form.form_errors %}
    <div class="alert alert-danger">
        {% for msg in register_form.form_errors %}
            <p>{{ msg }}</p>
        {% endfor %}
    </div>
{% endif %}
```

> `form_errors` → `Vec<String>` — errors not tied to a specific field (e.g. "Invalid credentials").
> `errors` → map `{ field_name: message }` — per-field errors + global errors under the `global` key.

---

## Field data as JSON

Forms automatically serialize `errors`, `form_errors`, `html`, `rendered_fields`, `fields` and `js_files`.

---

← [**Database errors**](/docs/en/formulaire/errors) | [**Full example**](/docs/en/formulaire/example) →
