# Exemples de surcharge admin

Cas complets, du plus léger au plus lourd. Les déclarations se font dans `main.rs` (pour tous) ou sur le `ModelAdmin` du modèle (pour un seul). Voir [Surcharge](/docs/fr/admin/template-surcharge) pour les règles, et la [référence des blocs](/docs/fr/admin/template-surcharge-blocks) pour les clés.

```rust
use runique::admin::template::{Block, GlobalBlock, Page};
```

---

## 1. Le CSS seul : restyler le tableau de toutes les listes

Le HTML du tableau reste celui de Runique. `ListTable` a une feuille chez Runique (`css/admin/blocks/list.css`) : la vôtre la remplace, partez donc d'une copie de celle de Runique.

```rust
.with_admin(|a| a
    .templates(|t| t.block_stylesheet(Block::ListTable, "css/admin/list.css"))
    .site(site)
)
```

```css
/* static/css/admin/list.css : copie de css/admin/blocks/list.css, modifiée */
.admin-list__cards { gap: 0.5em; }
```

---

## 2. Le template seul, pour un modèle : les boutons du formulaire d'article

Seuls les formulaires de création et d'édition des articles changent ; les autres modèles gardent ceux de Runique.

```rust
fn article() -> ModelAdmin<article::Entity> {
    ModelAdmin::new()
        .templates(|t| t.block(Block::FormActions, "articles/form_actions.html"))
}
```

```html
{# templates/articles/form_actions.html : inclus dans le <form> de Runique #}
<div class="admin-form__actions">
  <p class="admin-page__subtitle">Un article enregistré est publié tout de suite.</p>
  <a href="{{ cancel_url }}" class="btn btn-secondary">{{ cancel_label }}</a>
  <button type="submit" class="btn btn-primary">{{ submit_label }}</button>
</div>
```

Le bloc est inclus à l'intérieur du `<form>` de la page : le token CSRF et les champs sont déjà là, le bouton `submit` suffit.

---

## 3. Les deux : une barre latérale à vous

```rust
.templates(|t| t
    .global_block(GlobalBlock::Sidebar, "mon_theme/sidebar.html")
    .global_block_stylesheet(GlobalBlock::Sidebar, "css/admin/sidebar.css")
)
```

```html
{# templates/mon_theme/sidebar.html #}
<aside class="admin-sidebar" id="adminSidebar">
  <nav class="mon-nav">
    <a href="{{ nav.dashboard }}">{{ admin_base_dashboard }}</a>
    {% for resource in resources %}
      <a href="{{ resource.list_url }}"
         {% if current_resource == resource.key %}class="active"{% endif %}>
        {{ resource.title }}
      </a>
    {% endfor %}
    {% if nav.history %}<a href="{{ nav.history }}">{{ admin_base_history }}</a>{% endif %}
  </nav>
</aside>
```

`resources` ne contient que les modèles que l'utilisateur connecté a le droit de voir : pas de filtre à écrire. Gardez `id="adminSidebar"` si vous voulez que le bouton du menu mobile continue d'ouvrir la barre (`admin.js` la cherche par cet id).

---

## 4. Une page qui étend celle de Runique

Pour changer l'en-tête de la liste des articles et garder tout le reste, la page étend `admin/list.html` et redéfinit son bloc Tera `list_header` :

```rust
ModelAdmin::new()
    .templates(|t| t.page(Page::List, "articles/list.html"))
```

```html
{# templates/articles/list.html #}
{% extends "admin/list.html" %}

{% block list_header %}
<div class="admin-page__header">
  <h1 class="admin-page__title">{{ resource.title }}</h1>
  <p class="admin-page__subtitle">{{ total }} article(s), les plus récents en premier.</p>
  {% if resource_links.create %}
    <a href="{{ resource_links.create }}" class="btn btn-primary">{{ admin_list_btn_create }}</a>
  {% endif %}
</div>
{% endblock %}
```

Les blocs `dev_block` de la page (tableau, filtres, pagination…) suivent toujours vos surcharges de blocs.

---

## 5. Une page entière sur le layout de Runique

La page étend `admin_base.html` : feuilles de style, barre latérale, barre du haut et CSRF restent ceux de Runique, le contenu est entièrement le vôtre.

```rust
.templates(|t| t
    .with_dashboard("mon_theme/dashboard.html")
    .page_stylesheet(Page::Dashboard, "css/admin/dashboard.css")
)
```

```html
{# templates/mon_theme/dashboard.html #}
{% extends "admin_base.html" %}

{% block title %}{{ site_title }}{% endblock %}

{% block content %}
<h1>{{ site_title }}</h1>
<ul class="mon-tableau">
  {% for resource in resources %}
    <li><a href="{{ resource.list_url }}">{{ resource.title }}</a></li>
  {% endfor %}
</ul>
{% endblock %}
```

Avec `page_stylesheet`, la page charge les variables de Runique puis `css/admin/dashboard.css`, et plus aucune autre feuille de Runique. Sans, elle garde les feuilles de Runique.

---

## 6. Une page autonome (sans héritage)

Aucun `{% extends %}` : utile pour une intégration avec un autre outil frontend. Ce que `admin/admin_template.html` apportait est alors à écrire à la main : la meta CSRF, `csrf.js` (les actions en POST en ont besoin) et l'attribut `{% csp %}` sur chaque script.

```html
{# templates/mon_theme/list.html #}
<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
  <meta charset="UTF-8">
  <meta name="robots" content="noindex, nofollow">
  <meta name="csrf-token" content="{{ csrf_token }}">
  <title>{{ resource.title }} — {{ site_title }}</title>
  <link rel="stylesheet" href="{% static "css/admin/mon_admin.css" %}">
</head>
<body>
  <nav>
    {% for res in resources %}
      <a href="{{ res.list_url }}">{{ res.title }}</a>
    {% endfor %}
  </nav>
  <main>
    {% messages %}
    <h1>{{ resource.title }}</h1>
    {% for entry in entries %}
      <p><a href="{{ entry.links.detail }}">#{{ entry.id }}</a></p>
    {% endfor %}
  </main>
  <script {% csp %} src="{% runique_static "js/csrf.js" %}" defer></script>
</body>
</html>
```

---

## Récapitulatif

| Besoin | Déclaration |
| --- | --- |
| Changer les couleurs | feuille de thème, voir [Thème CSS](/docs/fr/admin/template-surcharge-blocks) |
| Restyler un bloc | `.block_stylesheet(Block::…, css)` / `.global_block_stylesheet(GlobalBlock::…, css)` |
| Changer le HTML d'un bloc | `.block(Block::…, html)` / `.global_block(GlobalBlock::…, html)` |
| Le même, pour un seul modèle | `.templates(…)` sur le `ModelAdmin` du modèle |
| Changer une partie d'une page | page qui étend celle de Runique (`{% extends "admin/list.html" %}`) |
| Refaire une page | `.with_list(…)` etc., ou `.page(Page::…, …)` pour un modèle, avec `page_stylesheet` si besoin |

---

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Surcharge](/docs/fr/admin/template-surcharge) | Règles et déclarations |
| [Clés de contexte](/docs/fr/admin/template-clef) | Variables disponibles par vue |
