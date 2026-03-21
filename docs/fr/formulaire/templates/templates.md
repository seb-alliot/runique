# Rendu dans les templates

[← Erreurs de base de données](/docs/fr/formulaire/erreurs)

---

## Formulaire complet

```html
<form method="post">
  {% form.inscription_form %}
  <button type="submit">S'inscrire</button>
</form>
```

Rend automatiquement : tous les champs, les labels, les erreurs de validation, le token CSRF et les scripts JS.

---

## Champ par champ

```html
<form method="post">
  {% csrf %}
  <div class="row">
    <div class="col-6">{% form.inscription_form.username %}</div>
    <div class="col-6">{% form.inscription_form.email %}</div>
  </div>
  {% form.inscription_form.password %}
  <button type="submit">S'inscrire</button>
</form>
```

---

## Erreurs globales

```html
{% if inscription_form.form_errors %}
<div class="alert alert-danger">
  {% for msg in inscription_form.form_errors %}
  <p>{{ msg }}</p>
  {% endfor %}
</div>
{% endif %}
```

> `form_errors` → `Vec<String>` — erreurs non liées à un champ (ex: "Identifiants invalides").
> `errors` → map `{ field_name: message }` — erreurs par champ + erreurs globales sous la clé `global`.

---

## Données de champ en JSON

Les formulaires sérialisent automatiquement `errors`, `form_errors`, `html`, `rendered_fields`, `fields` et `js_files`.

---

← [**Erreurs de base de données**](/docs/fr/formulaire/erreurs) | [**Exemple complet**](/docs/fr/formulaire/exemple) →
