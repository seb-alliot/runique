# Tags Django-like

Runique pré-traite les templates pour transformer une syntaxe Django-like en syntaxe Tera standard.

## {% static %} — Assets statiques

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">
```

**Transformé en :** `{{ "css/main.css" | static }}` → `/static/css/main.css`

---

## {% media %} — Fichiers médias (uploads)

```html
<img src='{% media "avatars/photo.jpg" %}' alt="Photo de profil">
```

**Transformé en :** `{{ "avatars/photo.jpg" | media }}` → `/media/avatars/photo.jpg`

---

## {% csrf %} — Protection CSRF

```html
<form method="post" action="/inscription">
    {% csrf %}
    <button type="submit">Envoyer</button>
</form>
```

**Transformé en :** `{% include "csrf/csrf_field.html" %}`

> Non nécessaire dans les formulaires Runique (`{% form.xxx %}`) — le token CSRF est injecté automatiquement.

---

## {% messages %} — Flash messages

```html
{% messages %}
```

**Transformé en :** `{% include "message/message_include.html" %}`

---

## {% csp_nonce %} — Nonce CSP

```html
<script {% csp_nonce %}>
    console.log("Script sécurisé avec nonce CSP");
</script>
```

**Transformé en :** `{% include "csp/csp_nonce.html" %}`

---

## {% link %} — Liens vers des routes nommées

```html
<a href='{% link "index" %}'>Accueil</a>
<a href='{% link "user_detail" id="42" %}'>Profil utilisateur</a>
```

**Transformé en :** `{{ link(link='index') }}`

---

## {% form.xxx %} — Rendu de formulaire complet

```html
<form method="post" action="/inscription">
    {% form.inscription_form %}
    <button type="submit">S'inscrire</button>
</form>
```

**Transformé en :** `{{ inscription_form | form | safe }}`

Rend l'intégralité du formulaire : tous les champs HTML, les erreurs de validation, le token CSRF, et les scripts JS nécessaires.

---

## {% form.xxx.champ %} — Rendu d'un champ isolé

```html
<form method="post" action="/inscription">
    <div class="row">
        <div class="col">{% form.inscription_form.username %}</div>
        <div class="col">{% form.inscription_form.email %}</div>
    </div>
    {% form.inscription_form.password %}
    <button type="submit">S'inscrire</button>
</form>
```

**Transformé en :** `{{ inscription_form | form(field='username') | safe }}`

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Filtres & fonctions](/docs/fr/template/filtres) | Filtres Tera bas niveau |
| [Syntaxe Tera](/docs/fr/template/syntaxe) | Héritage, boucles, conditions |

## Retour au sommaire

- [Templates](/docs/fr/template)
