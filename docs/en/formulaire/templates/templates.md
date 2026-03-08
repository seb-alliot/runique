# Template rendering

[← Database errors](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/errors/errors.md)

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
{% if register_form.errors %}
    <div class="alert alert-danger">
        {% for msg in register_form.errors %}
            <p>{{ msg }}</p>
        {% endfor %}
    </div>
{% endif %}
```

---

## Field data as JSON

Forms automatically serialize `data`, `errors`, `html`, `rendered_fields`, `fields` and `js_files`.

---

← [**Database errors**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/errors/errors.md) | [**Full example**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/example/example.md) →
