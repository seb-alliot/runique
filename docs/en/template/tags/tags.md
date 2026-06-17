# Django-like tags

Runique pre-processes templates to transform Django-like syntax into standard Tera syntax.

## {% static %} — Static assets

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">
```

**Transformed into:** `{{ "css/main.css" | static }}` → `/static/css/main.css`

---

## {% media %} — Media files (uploads)

```html
<img src='{% media "avatars/photo.jpg" %}' alt="Profile photo">
```

**Transformed into:** `{{ "avatars/photo.jpg" | media }}` → `/media/avatars/photo.jpg`

---

## {% csrf %} — CSRF protection

```html
<form method="post" action="/signup">
    {% csrf %}
    <button type="submit">Submit</button>
</form>
```

**Transformed into:** `{% include "csrf" %}`

> Not required inside Runique forms (`{% form.xxx %}`) — the CSRF token is injected automatically.

---

## {% messages %} — Flash messages

```html
{% messages %}
```

**Transformed into:** `{% include "message" %}`

---

## {% csp %} — CSP nonce

```html
<script {% csp %}>
    console.log("Script secured with CSP nonce");
</script>
```

**Transformed into:** `{% include "csp" %}`

---

## {% link %} — Named route links

```html
<a href='{% link "index" %}'>Home</a>
<a href='{% link "user_detail" id="42" %}'>User profile</a>
```

**Transformed into:** `{{ link(link='index') }}`

---

## {% form.xxx %} — Full form rendering

```html
<form method="post" action="/signup">
    {% form.signup_form %}
    <button type="submit">Sign up</button>
</form>
```

**Transformed into:** `{{ signup_form | form | safe }}`

Renders the entire form: all HTML fields, validation errors, the CSRF token, and required JS scripts.

---

## {% form.xxx.field %} — Single field rendering

```html
<form method="post" action="/signup">
    <div class="row">
        <div class="col">{% form.signup_form.username %}</div>
        <div class="col">{% form.signup_form.email %}</div>
    </div>
    {% form.signup_form.password %}
    <button type="submit">Sign up</button>
</form>
```

**Transformed into:** `{{ signup_form | form(field='username') | safe }}`

> In field-by-field rendering, the form's JS scripts are **not** auto-injected (unlike full `{% form.xxx %}`). Place them explicitly with `{% form.xxx.js %}` — see below.

---

## {% form.xxx.js %} — Form JS scripts

Emits the form's `<script>` block: one per file declared via `add_js`, with the correct CSP nonce and resolved static URL. Use it **only in field-by-field rendering**, wherever you want the scripts (typically just before `</form>`).

```html
<form method="post" action="/signup">
    <div class="row">
        <div class="col">{% form.signup_form.username %}</div>
        <div class="col">{% form.signup_form.email %}</div>
    </div>
    {% form.signup_form.password %}
    <button type="submit">Sign up</button>

    {% form.signup_form.js %}
</form>
```

**Transformed into:** `{{ signup_form | form(field='js') | safe }}`

> Not needed with `{% form.xxx %}` (full rendering): scripts are already included there, in last position. Don't combine the two — you would get the `<script>` tags twice.

---

## See also

| Section | Description |
| --- | --- |
| [Filters & functions](/docs/en/template/filters) | Low-level Tera filters |
| [Tera syntax](/docs/en/template/syntax) | Inheritance, loops, conditions |

## Back to summary

- [Templates](/docs/en/template)
