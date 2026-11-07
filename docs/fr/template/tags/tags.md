# Tags Django-like

Runique pré-traite les templates pour transformer une syntaxe Django-like en syntaxe Tera standard.

## {% static %} — Assets statiques

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">
```

**Transformé en :** `{{ "css/main.css" | static }}` → `/static/css/main.css`

En production, la balise ajoute aussi l'empreinte du fichier : `integrity="sha384-…" crossorigin="anonymous"`, calculée au démarrage sur le contenu de `STATICFILES_DIRS`. En développement (`RUNIQUE_ENV=dev`), pas d'empreinte : un fichier modifié est servi sans redémarrer.

> La balise doit se trouver **entre les guillemets** de l'attribut (`href='{% static "…" %}'`) : ces guillemets font partie de la règle de remplacement. Écrite hors d'un attribut, elle n'est pas transformée.

---

## {% runique_static %} — Fichiers statiques de Runique

```html
<link rel="stylesheet" href='{% runique_static "css/auth.css" %}'>
```

Comme `{% static %}`, mais pour les fichiers fournis par Runique (admin, pages d'authentification, `js/csrf.js`) : l'empreinte est lue dans le dossier statique de Runique.

---

## {% media %} — Fichiers médias (uploads)

```html
<img src='{% media "avatars/photo.jpg" %}' alt="Photo de profil">
```

**Transformé en :** `{{ "avatars/photo.jpg" | media }}` → `/media/avatars/photo.jpg`

### Avec une variable

`{% static %}` et `{% media %}` acceptent aussi une **variable Tera**, sans guillemets — typiquement le chemin d'un fichier envoyé, enregistré en base :

```html
<img src="{% media user.avatar %}" alt="Avatar">
```

**Transformé en :** `{{ user.avatar | media }}`. Avec guillemets, la valeur est un chemin littéral ; sans guillemets, c'est le nom d'une variable du contexte.

---

## {% csrf %} — Protection CSRF

```html
<form method="post" action="/inscription">
    {% csrf %}
    <button type="submit">Envoyer</button>
</form>
```

**Transformé en :** `{% include "csrf.html" %}`

> Non nécessaire dans les formulaires Runique (`{% form.xxx %}`) — le token CSRF est injecté automatiquement.

---

## {% messages %} — Flash messages

```html
{% messages %}
```

**Transformé en :** `{% include "message.html" %}`

---

## {% csp %} — Nonce CSP

```html
<script {% csp %}>
    console.log("Script sécurisé avec nonce CSP");
</script>
```

**Transformé en :** `{% include "csp.html" %}`

---

## {% link %} — Liens vers des routes nommées

```html
<a href='{% link "index" %}'>Accueil</a>
<a href='{% link "user_detail" id="42" %}'>Profil utilisateur</a>
```

**Transformé en :** `{{ link(link='index') }}`, et pour une route à paramètres `{{ link(link='user_detail', id='42') }}`. Seules les routes déclarées avec `name = "…"` dans `urlpatterns!` sont résolues ; un nom inconnu fait échouer le rendu.

---

## {% form.xxx %} — Rendu de formulaire complet

```html
<form method="post" action="/inscription">
    {% form.inscription_form %}
    <button type="submit">S'inscrire</button>
</form>
```

**Transformé en :** `{{ inscription_form | form }}` (le filtre `form` déclare `is_safe()` — pas besoin de `| safe`)

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

**Transformé en :** `{{ inscription_form | form(field='username') }}`

> En rendu champ par champ, les scripts JS du formulaire ne sont **pas** injectés automatiquement (contrairement à `{% form.xxx %}` complet). Place-les explicitement avec `{% form.xxx.js %}` — voir ci-dessous.

---

## {% form.xxx.js %} — Scripts JS du formulaire

Émet le bloc `<script>` du formulaire : un par fichier déclaré via `add_js`, avec le nonce CSP de la requête, le hash SRI du fichier et l'URL statique résolue. À utiliser **uniquement en rendu champ par champ**, là où tu veux les scripts (typiquement juste avant `</form>`).

```html
<form method="post" action="/inscription">
    <div class="row">
        <div class="col">{% form.inscription_form.username %}</div>
        <div class="col">{% form.inscription_form.email %}</div>
    </div>
    {% form.inscription_form.password %}
    <button type="submit">S'inscrire</button>

    {% form.inscription_form.js %}
</form>
```

**Transformé en :** `{{ inscription_form | form(field='js') }}`

> Inutile avec `{% form.xxx %}` (rendu complet) : les scripts y sont déjà inclus, en dernière position. Ne combine pas les deux — tu obtiendrais les `<script>` en double.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Filtres & fonctions](/docs/fr/template/filtres) | Filtres Tera bas niveau |
| [Syntaxe Tera](/docs/fr/template/syntaxe) | Héritage, boucles, conditions |

## Retour au sommaire

- [Templates](/docs/fr/template)
