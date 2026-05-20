# Surcharge des templates admin

L'admin Runique permet de remplacer n'importe quel template par un template personnalisé,
tout en conservant les éléments contractuels (CSRF, messages flash).

---

## Principe : 3 niveaux d'héritage

```
admin_template.html   ← niveau 1 : contrat Runique (ne pas toucher)
        ↓ extends
admin_base.html            ← niveau 2 : layout par défaut (peut être remplacé)
        ↓ extends
list.html / create.html …  ← niveau 3 : composants CRUD (peuvent être remplacés)
```

Le dev peut remplacer le niveau 2 (layout global) et/ou le niveau 3 (pages individuelles).

---

## Surcharger le layout global (`admin_base.html`)

Créer un fichier qui hérite de `admin_template` et remplit les blocks de layout.

### `templates/mon_theme/admin_base.html`

```html
{% extends "admin/admin_template" %}

{% block extra_css %}
    <link rel="stylesheet" href="{{ "css/mon_theme.css" | runique_static }}">
{% endblock %}

{% block sidebar %}
<nav class="mon-sidebar">
    <h2>{{ site_title }}</h2>
    <ul>
    {% for res in resources %}
        <li>
            <a href="{{ admin_prefix }}/{{ res.key }}/list"
               {% if res.key == current_resource %}class="active"{% endif %}>
                {{ res.title }}
            </a>
        </li>
    {% endfor %}
    </ul>
</nav>
{% endblock %}

{% block topbar %}
<header class="mon-topbar">
    {% block breadcrumb %}{% endblock %}
    <form method="POST" action="{{ admin_prefix }}/logout">
        {% csrf %}
        <button type="submit">{{ current_user.username }} — Déconnexion</button>
    </form>
</header>
{% endblock %}

{% block extra_js %}
    <script src="{{ "js/mon_admin.js" | runique_static }}" defer></script>
{% endblock %}
```

> **Note** : `admin/admin_template` est la clé Tera, pas un chemin de fichier.

---

## Déclarer le template custom dans la config

```rust
RuniqueApp::builder(config)
    .with_admin(|a| a
        .templates(|t| t
            .with_list("mon_theme/list")
            .with_create("mon_theme/create")
            .with_edit("mon_theme/edit")
            .with_detail("mon_theme/detail")
            .with_delete("mon_theme/delete")
            .with_dashboard("mon_theme/dashboard")
            .with_login("mon_theme/login")
            .with_base("mon_theme/admin_base")
        )
    )
    .build().await?
```

---

## Surcharger un composant CRUD spécifique

Pour remplacer uniquement la page de liste des utilisateurs :

### `templates/mon_theme/users_list.html`

```html
{% extends "mon_theme/admin_base" %}

{% block content %}
<h1>{{ resource.title }}</h1>
<p>{{ total }} entrée(s)</p>

{% for entry in entries %}
    <div class="user-card">
        <span>#{{ entry.id }}</span>
        <span>{{ entry.username }}</span>
    </div>
{% endfor %}
{% endblock %}
```

---

## Blocks disponibles pour la surcharge

Voir la [référence complète des blocks](/docs/fr/admin/template/surcharge/blocks) — liste exhaustive par template avec les variables CSS surchargeables.

---

## Points d'attention

- `current_resource` est une **String** (la clé, ex: `"users"`), pas un objet. Utiliser `resource.key` et `resource.title` pour les métadonnées.
- Si `{% block extra_js %}` est surchargé, appeler `{{ super() }}` pour ne pas perdre `admin.js`.
- Si `{% block extra_css %}` est surchargé, appeler `{{ super() }}` pour conserver le CSS Runique.
- Les éléments hors-blocks (`<meta csrf-token>`, `<script csrf.js>`) sont garantis par `admin_template` — ils ne peuvent pas être supprimés par surcharge.
- Si `{% block messages %}` est redéfini, **conserver** `{% messages %}` dedans.

## Sous-sections

| Section | Description |
| --- | --- |
| [Référence des blocks](/docs/fr/admin/template/surcharge/blocks) | Liste complète des blocks par template + variables CSS |
| [Exemples](/docs/fr/admin/template) | 3 approches : héritage Runique, layout custom, HTML autonome |
| [Clés de contexte](/docs/fr/admin/template/clef/context) | Variables injectées par le backend dans chaque template |
| [CSRF](/docs/fr/admin/template) | Token CSRF, `csrf.js`, checklist login custom |

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
| [Sommaire](/docs/fr/admin) | Sommaire admin |
