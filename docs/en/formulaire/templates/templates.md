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
    <div class="row">
        <div class="col-6">{% form.register_form.username %}</div>
        <div class="col-6">{% form.register_form.email %}</div>
    </div>
    {% form.register_form.password %}
    <button type="submit">Sign up</button>
</form>
```

---

The form's **first declared field** carries the hidden fields along (CSRF token and form name), wherever it sits in the template. Render them all, or at least that first field: without it, the submission is refused (CSRF) or ignored (no name).

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

Forms automatically serialize `errors`, `form_errors`, `html`, `rendered_fields`, `fields`, `js_files`, `rendered_js` (the `<script>` block already rendered, with the CSP nonce) and `honeypot_html` (the anti-bot trap field, empty without it).

---

← [**Database errors**](/docs/en/formulaire/errors) | [**Full example**](/docs/en/formulaire/example) →
