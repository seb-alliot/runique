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
            <a href="/admin/{{ res.key }}/list"
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
    <form method="POST" action="/admin/logout">
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
RuniqueApp::new()
    .with_admin(|a| a
        .templates(|t| t
            .with_base("mon_theme/admin_base")
        )
    )
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

Déclaration dans `src/admin.rs` :

```rust
admin! {
    users: users::Model => AdminForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        template_list: "mon_theme/users_list",
    }
}
```

---

## Blocks disponibles pour la surcharge

| Block | Contenu par défaut | Peut être surchargé |
| --- | --- | --- |
| `{% block title %}` | Titre de la page | Oui |
| `{% block extra_css %}` | CSS Runique admin | Oui |
| `{% block layout %}` | Tout le layout (sidebar + main) | Oui (avancé) |
| `{% block sidebar %}` | Sidebar avec navigation ressources | Oui |
| `{% block topbar %}` | Topbar avec breadcrumb + logout | Oui |
| `{% block breadcrumb %}` | Fil d'Ariane | Oui (depuis admin_base) |
| `{% block messages %}` | `{% messages %}` | Oui — conserver `{% messages %}` |
| `{% block content %}` | Contenu de la page CRUD | Oui |
| `{% block extra_js %}` | `admin.js` | Oui — utiliser `{{ super() }}` pour cumuler |

---

## Points d'attention

- `current_resource` est une **String** (la clé, ex: `"users"`), pas un objet. Utiliser `resource.key` et `resource.title` pour les métadonnées.
- Si `{% block extra_js %}` est surchargé, appeler `{{ super() }}` pour ne pas perdre `admin.js`.
- Les éléments hors-blocks (`<meta csrf-token>`, `<script csrf.js>`) sont garantis par `admin_template` — ils ne peuvent pas être supprimés par surcharge.
- Si `{% block messages %}` est redéfini, **conserver** `{% messages %}` dedans.
