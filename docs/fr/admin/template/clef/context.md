# Runique Admin — Variables Tera par vue

Ce document liste toutes les variables disponibles dans le contexte Tera lorsqu'un développeur surcharge un template admin.

---

## Variables globales (toutes les routes)

Ces variables sont injectées sur **toutes les routes** par l'extracteur `Request` du framework, avant même que les handlers admin s'exécutent.

| Variable | Type Rust | Description |
| --- | --- | --- |
| `debug` | `bool` | Mode debug activé ou non |
| `csrf_token` | `String` | Token CSRF masqué — à inclure dans les formulaires POST |
| `csp_nonce` | `&str` | Nonce CSP pour les balises `<script>` et `<style>` |
| `static_runique` | `StaticConfig` | Config des assets statiques Runique (voir ci-dessous) |
| `messages` | `Vec<FlashMessage>` | Messages flash de la session courante |
| `current_user` | `CurrentUser` *(optionnel)* | Données de l'utilisateur connecté, absent si non authentifié |

### Champs de `static_runique`

```html
{{ static_runique.static_url }}   {# URL de base des assets, ex: /static #}
{{ static_runique.static_dir }}   {# Répertoire physique sur disque #}
```

---

## Variables injectées par les handlers CRUD

Ces variables sont injectées sur **toutes les vues CRUD admin** via `inject_context`, après l'extracteur.

| Variable | Type | Description |
| --- | --- | --- |
| `lang` | `String` | Code de langue courant (ex: `"fr"`) |
| `site_title` | `String` | Titre du site configuré dans `AdminConfig` |
| `site_url` | `String` | URL de base du site configurée dans `AdminConfig` |
| `resource_key` | `&str` | Clé de la ressource courante (ex: `"users"`) |
| `current_resource` | `&str` | Identique à `resource_key` |
| `resource` | `AdminResource` | Métadonnées complètes de la ressource courante (voir ci-dessous) |
| `resources` | `Vec<AdminResource>` | Toutes les ressources enregistrées dans le registre |
| `registered_roles` | `Vec<String>` | Tous les rôles enregistrés via `register_roles()` |

> Les clés déclarées dans `extra: {}` du bloc `admin!{}` sont également injectées **en tant que variables Tera de premier niveau**.
> Exemple : `extra: { "icon" => "user" }` → `{{ icon }}` (accessible directement) ET `{{ resource.extra_context.icon }}`.

### Structure `AdminResource`

| Champ Tera | Type | Description |
| --- | --- | --- |
| `resource.key` | `&str` | Clé unique de la ressource (`"users"`) |
| `resource.title` | `&str` | Titre lisible (`"Utilisateurs"`) |
| `resource.model_path` | `&str` | Chemin du modèle SeaORM (`"crate::entities::users::Model"`) |
| `resource.permissions.list` | `Vec<String>` | Rôles autorisés pour la liste |
| `resource.permissions.view` | `Vec<String>` | Rôles autorisés pour le détail |
| `resource.permissions.create` | `Vec<String>` | Rôles autorisés pour la création |
| `resource.permissions.edit` | `Vec<String>` | Rôles autorisés pour l'édition |
| `resource.permissions.delete` | `Vec<String>` | Rôles autorisés pour la suppression |
| `resource.display.icon` | `String` *(optionnel)* | Nom d'icône déclaré |
| `resource.display.pagination` | `usize` | Entrées par page (défaut : `25`) |
| `resource.extra_context` | `HashMap<String, String>` | Clés custom déclarées dans `extra: {}` |
| `resource.id_type` | `AdminIdType` | Type de la clé primaire (`I32`, `I64`, `Uuid`) |

---

## Variables i18n globales (toutes les vues CRUD)

Injectées automatiquement via `insert_admin_messages`. Le nom de variable Tera est la clé i18n avec les `.` remplacés par `_`.

### Section `base`

| Variable Tera | Clé i18n |
| --- | --- |
| `admin_base_title` | `admin.base.title` |
| `admin_base_breadcrumb` | `admin.base.breadcrumb` |
| `admin_base_toggle` | `admin.base.toggle` |
| `admin_base_logout_title` | `admin.base.logout_title` |

### Section `list`

| Variable Tera | Clé i18n |
| --- | --- |
| `admin_list_breadcrumb_admin` | `admin.list.breadcrumb_admin` |
| `admin_list_entries_count` | `admin.list.entries_count` |
| `admin_list_btn_create` | `admin.list.btn_create` |
| `admin_list_th_id` | `admin.list.th_id` |
| `admin_list_th_actions` | `admin.list.th_actions` |
| `admin_list_bool_true` | `admin.list.bool_true` |
| `admin_list_bool_false` | `admin.list.bool_false` |
| `admin_list_btn_detail` | `admin.list.btn_detail` |
| `admin_list_btn_edit` | `admin.list.btn_edit` |
| `admin_list_btn_delete` | `admin.list.btn_delete` |
| `admin_list_confirm_delete` | `admin.list.confirm_delete` |
| `admin_list_empty_title` | `admin.list.empty_title` |
| `admin_list_empty_desc` | `admin.list.empty_desc` |
| `admin_list_btn_create_first` | `admin.list.btn_create_first` |

### Section `create`

| Variable Tera | Clé i18n |
| --- | --- |
| `admin_create_title` | `admin.create.title` |
| `admin_create_breadcrumb` | `admin.create.breadcrumb` |
| `admin_create_card_info` | `admin.create.card_info` |
| `admin_create_no_fields` | `admin.create.no_fields` |
| `admin_create_btn_cancel` | `admin.create.btn_cancel` |
| `admin_create_btn_submit` | `admin.create.btn_submit` |

### Section `edit`

| Variable Tera | Clé i18n |
| --- | --- |
| `admin_edit_title` | `admin.edit.title` |
| `admin_edit_breadcrumb` | `admin.edit.breadcrumb` |
| `admin_edit_card_info` | `admin.edit.card_info` |
| `admin_edit_no_fields` | `admin.edit.no_fields` |
| `admin_edit_btn_cancel` | `admin.edit.btn_cancel` |
| `admin_edit_btn_submit` | `admin.edit.btn_submit` |

### Section `detail`

| Variable Tera | Clé i18n |
| --- | --- |
| `admin_detail_title` | `admin.detail.title` |
| `admin_detail_breadcrumb` | `admin.detail.breadcrumb` |
| `admin_detail_entry_label` | `admin.detail.entry_label` |
| `admin_detail_btn_list` | `admin.detail.btn_list` |
| `admin_detail_btn_edit` | `admin.detail.btn_edit` |
| `admin_detail_btn_delete` | `admin.detail.btn_delete` |
| `admin_detail_confirm_delete` | `admin.detail.confirm_delete` |

### Section `delete`

| Variable Tera | Clé i18n |
| --- | --- |
| `admin_delete_title` | `admin.delete.title` |
| `admin_delete_breadcrumb` | `admin.delete.breadcrumb` |
| `admin_delete_heading` | `admin.delete.heading` |
| `admin_delete_btn_cancel` | `admin.delete.btn_cancel` |
| `admin_delete_btn_confirm` | `admin.delete.btn_confirm` |
| `admin_delete_warning_title` | `admin.delete.warning.title` |
| `admin_delete_warning_desc` | `admin.delete.warning.desc` |
| `admin_delete_warning_of` | `admin.delete.warning.of` |
| `admin_delete_warning_irreversible` | `admin.delete.warning.irreversible` |

---

## Vue `login`

**Route :** `GET /admin/login`

> `inject_context` n'est **pas** appelé — les variables `resource`, `resources`, `resource_key` ne sont pas disponibles.

| Variable | Type | Description |
| --- | --- | --- |
| `site_title` | `String` | Titre du site |
| `lang` | `String` | Code de langue courant |
| `csrf_token` | `String` | Token CSRF (injecté par l'extracteur de base) |
| `admin_login_title` | `String` | i18n `admin.login.title` |
| `admin_login_subtitle` | `String` | i18n `admin.login.subtitle` |
| `admin_login_label_username` | `String` | i18n `admin.login.label_username` |
| `admin_login_label_password` | `String` | i18n `admin.login.label_password` |
| `admin_login_btn_submit` | `String` | i18n `admin.login.btn_submit` |
| `admin_login_error_session` | `String` | i18n `admin.login.error_session` |
| `admin_login_error_credentials` | `String` | i18n `admin.login.error_credentials` |
| `error` *(optionnel)* | `String` | Message d'erreur verbatim — injecté uniquement en cas d'échec POST |

### Clés obligatoires — `login`

```text
csrf_token, site_title, lang
```

> `error` est optionnel — présent uniquement après un échec POST.
> En cas d'échec POST, les messages i18n de la section `base` sont aussi injectés.

**Exemple minimal :**

```html
{% extends "admin/admin_template.html" %}

{% block title %}{{ admin_login_title }}{% endblock %}

{% block content %}
<div class="login-container">
    <h1>{{ admin_login_title }}</h1>
    <p>{{ admin_login_subtitle }}</p>

    {% if error %}
    <div class="alert alert-danger">{{ error }}</div>
    {% endif %}

    <form method="POST" action="/admin/login">
        <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
        <div>
            <label>{{ admin_login_label_username }}</label>
            <input type="text" name="username" required>
        </div>
        <div>
            <label>{{ admin_login_label_password }}</label>
            <input type="password" name="password" required>
        </div>
        <button type="submit">{{ admin_login_btn_submit }}</button>
    </form>
</div>
{% endblock %}
```

> Le template login étend `admin_template.html` (contrat de blocs vides), pas `admin_base.html` — la sidebar et la topbar ne doivent pas apparaître sur la page de connexion.

---

## Vue `dashboard`

**Route :** `GET /admin/`

> `inject_context` n'est **pas** appelé. La variable `current_resource` est explicitement `None`.

| Variable | Type | Description |
| --- | --- | --- |
| `site_title` | `String` | Titre du site |
| `lang` | `String` | Code de langue courant |
| `resources` | `Vec<AdminResource>` | Toutes les ressources enregistrées |
| `resource_counts` | `HashMap<String, u64>` | Nombre d'entrées par ressource (clé = `resource.key`) |
| `current_page` | `&str` | Vaut `"dashboard"` |
| `current_resource` | `None` | Absent — aucune ressource sélectionnée |
| `admin_base_*` | — | Clés i18n section `base` (voir ci-dessus) |
| `admin_dashboard_title` | `String` | i18n `admin.dashboard.title` |
| `admin_dashboard_subtitle` | `String` | i18n `admin.dashboard.subtitle` |
| `admin_dashboard_card_resources` | `String` | i18n `admin.dashboard.card_resources` |
| `admin_dashboard_th_resource` | `String` | i18n `admin.dashboard.th_resource` |
| `admin_dashboard_th_key` | `String` | i18n `admin.dashboard.th_key` |
| `admin_dashboard_th_permissions` | `String` | i18n `admin.dashboard.th_permissions` |
| `admin_dashboard_th_actions` | `String` | i18n `admin.dashboard.th_actions` |
| `admin_dashboard_btn_list` | `String` | i18n `admin.dashboard.btn_list` |
| `admin_dashboard_btn_create` | `String` | i18n `admin.dashboard.btn_create` |
| `admin_dashboard_see_list` | `String` | i18n `admin.dashboard.see_list` |
| `admin_dashboard_empty_title` | `String` | i18n `admin.dashboard.empty_title` |
| `admin_dashboard_empty_desc` | `String` | i18n `admin.dashboard.empty_desc` |

### Clés obligatoires — `dashboard`

```text
resources, resource_counts, current_page
```

**Exemple minimal :**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_dashboard_title }}{% endblock %}

{% block content %}
<h1>{{ admin_dashboard_title }}</h1>
<p>{{ admin_dashboard_subtitle }}</p>

{% if resources %}
<h2>{{ admin_dashboard_card_resources }}</h2>
<table class="table">
    <thead>
        <tr>
            <th>{{ admin_dashboard_th_resource }}</th>
            <th>{{ admin_dashboard_th_key }}</th>
            <th>{{ admin_dashboard_th_permissions }}</th>
            <th>{{ admin_dashboard_th_actions }}</th>
        </tr>
    </thead>
    <tbody>
        {% for res in resources %}
        <tr>
            <td>{{ res.title }}</td>
            <td><code>{{ res.key }}</code></td>
            <td>{{ res.permissions.list | join(sep=", ") }}</td>
            <td>
                <a href="/admin/{{ res.key }}/list">
                    {{ admin_dashboard_btn_list }}
                    {% if resource_counts[res.key] %}({{ resource_counts[res.key] }}){% endif %}
                </a>
                <a href="/admin/{{ res.key }}/create">{{ admin_dashboard_btn_create }}</a>
            </td>
        </tr>
        {% endfor %}
    </tbody>
</table>
{% else %}
<p>{{ admin_dashboard_empty_title }}</p>
<p>{{ admin_dashboard_empty_desc }}</p>
{% endif %}
{% endblock %}
```

---

## Vue `list`

**Route :** `GET /admin/{resource}/list`

### Pagination

| Variable | Type | Description |
| --- | --- | --- |
| `entries` | `Vec<Value>` | Enregistrements de la page courante, sérialisés en JSON |
| `total` | `u64` | Nombre total d'entrées |
| `page` | `u64` | Page courante (commence à 1) |
| `page_count` | `u64` | Nombre total de pages |
| `has_prev` | `bool` | Il existe une page précédente |
| `has_next` | `bool` | Il existe une page suivante |
| `prev_page` | `u64` | Numéro de la page précédente |
| `next_page` | `u64` | Numéro de la page suivante |
| `current_page` | `&str` | Vaut `"list"` |

### Colonnes

| Variable | Type | Description |
| --- | --- | --- |
| `visible_columns` | `Vec<String>` | Noms des colonnes à afficher (depuis `list_display` ou toutes sauf `id`/`password`) |
| `column_labels` | `HashMap<String, String>` | Label par colonne — vide si `list_display` non configuré, sinon `{ "col" => "Label" }` |

### Tri

| Variable | Type | Description |
| --- | --- | --- |
| `sort_by` | `String` | Colonne de tri active — chaîne vide si aucun tri |
| `sort_dir` | `String` | Direction : `"asc"` ou `"desc"` |
| `sort_dir_toggle` | `String` | Direction opposée à `sort_dir` — pratique pour les liens d'en-tête |

### Recherche

| Variable | Type | Description |
| --- | --- | --- |
| `search` | `String` | Terme de recherche courant — chaîne vide si aucune recherche |

### Filtres sidebar

| Variable | Type | Description |
| --- | --- | --- |
| `filter_values` | `HashMap<String, Vec<String>>` | Valeurs distinctes par colonne de filtre (depuis `list_filter`) |
| `active_filters` | `HashMap<String, String>` | Filtre actif par colonne — `""` si aucun filtre actif sur cette colonne |
| `filter_qs` | `String` | Fragment query string des filtres actifs — à inclure dans les liens de pagination |
| `filter_meta` | `HashMap<String, Object>` | Pagination sidebar par colonne — voir structure ci-dessous |

> **Note :** `active_filters` est pré-rempli pour **toutes** les colonnes de `list_filter` (valeur `""` si inactif). Tera lève une erreur si on accède à une clé absente — cette pré-initialisation l'évite.

### Structure de `filter_meta[colonne]`

| Champ | Type | Description |
| --- | --- | --- |
| `current_page` | `u64` | Page courante dans la sidebar filtre (commence à 0) |
| `total_pages` | `u64` | Nombre total de pages de valeurs distinctes |
| `has_prev` | `bool` | Il existe une page précédente |
| `has_next` | `bool` | Il existe une page suivante |
| `prev_qs` | `String` | Query string complet pour la page précédente — `href="?{{ filter_meta[col].prev_qs }}"` |
| `next_qs` | `String` | Query string complet pour la page suivante |

### Clés obligatoires — `list`

Référencées via `runique::utils::constante::admin_ctx::list::REQUIRED` :

```text
entries, total, page, page_count, has_prev, has_next,
prev_page, next_page, visible_columns,
sort_by, sort_dir, sort_dir_toggle, search
```

> Les variables i18n de la section `list` sont listées dans les variables globales ci-dessus.

**Exemple minimal :**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ resource.title }}{% endblock %}

{% block content %}
<div class="d-flex justify-content-between align-items-center mb-3">
    <h1>{{ resource.title }}
        <small class="text-muted fs-6">{{ total }} {{ admin_list_entries_count }}</small>
    </h1>
    <a href="/admin/{{ resource_key }}/create" class="btn btn-primary">
        {{ admin_list_btn_create }}
    </a>
</div>

{% if entries %}
<table class="table">
    <thead>
        <tr>
            <th>{{ admin_list_th_id }}</th>
            {# ajouter les colonnes selon le modèle #}
            <th>{{ admin_list_th_actions }}</th>
        </tr>
    </thead>
    <tbody>
        {% for entry in entries %}
        <tr>
            <td>{{ entry.id }}</td>
            <td>
                <a href="/admin/{{ resource_key }}/{{ entry.id }}/detail">{{ admin_list_btn_detail }}</a>
                <a href="/admin/{{ resource_key }}/{{ entry.id }}/edit">{{ admin_list_btn_edit }}</a>
                <a href="/admin/{{ resource_key }}/{{ entry.id }}/delete">{{ admin_list_btn_delete }}</a>
            </td>
        </tr>
        {% endfor %}
    </tbody>
</table>
{% else %}
<p>{{ admin_list_empty_title }}</p>
<p>{{ admin_list_empty_desc }}</p>
<a href="/admin/{{ resource_key }}/create">{{ admin_list_btn_create_first }}</a>
{% endif %}
{% endblock %}
```

---

## Vue `create`

**Route :** `GET /admin/{resource}/create`

| Variable | Type | Description |
| --- | --- | --- |
| `form_fields` | `Forms` | Formulaire généré par Prisme — rendu via `{% form.field_name %}` ou `form_fields.html` |
| `is_edit` | `bool` | Vaut `false` |

### Clés obligatoires — `create`

```text
form_fields, is_edit
```

> Les variables i18n de la section `create` sont listées dans les variables globales ci-dessus.

**Exemple minimal :**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_create_title }} — {{ resource.title }}{% endblock %}

{% block content %}
<h1>{{ admin_create_title }} — {{ resource.title }}</h1>
<p class="text-muted">{{ admin_create_card_info }}</p>

<form method="POST" action="/admin/{{ resource_key }}/create">
    {# csrf.js gère le token automatiquement pour les formulaires admin #}
    {% if form_fields.html %}
        {{ form_fields.html | safe }}
    {% else %}
        <p>{{ admin_create_no_fields }}</p>
    {% endif %}
    <button type="submit" class="btn btn-primary">{{ admin_create_btn_submit }}</button>
    <a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_create_btn_cancel }}</a>
</form>
{% endblock %}
```

---

## Vue `edit`

**Route :** `GET /admin/{resource}/{id}/edit`

| Variable | Type | Description |
| --- | --- | --- |
| `form_fields` | `Forms` | Formulaire pré-rempli avec les données existantes |
| `is_edit` | `bool` | Vaut `true` |
| `object_id` | `String` | ID de l'entrée en cours d'édition |

### Clés obligatoires — `edit`

```text
form_fields, is_edit, object_id
```

> Les variables i18n de la section `edit` sont listées dans les variables globales ci-dessus.

**Exemple minimal :**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_edit_title }} — {{ resource.title }} #{{ object_id }}{% endblock %}

{% block content %}
<h1>{{ admin_edit_title }} — {{ resource.title }} <small>#{{ object_id }}</small></h1>
<p class="text-muted">{{ admin_edit_card_info }}</p>

<form method="POST" action="/admin/{{ resource_key }}/{{ object_id }}/edit">
    {% if form_fields.html %}
        {{ form_fields.html | safe }}
    {% else %}
        <p>{{ admin_edit_no_fields }}</p>
    {% endif %}
    <button type="submit" class="btn btn-primary">{{ admin_edit_btn_submit }}</button>
    <a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_edit_btn_cancel }}</a>
</form>
{% endblock %}
```

---

## Vue `detail`

**Route :** `GET /admin/{resource}/{id}/detail`

| Variable | Type | Description |
| --- | --- | --- |
| `entry` | `Value` *(optionnel)* | Enregistrement sérialisé en JSON — absent si `get_fn` non configurée |
| `object_id` | `String` | ID de l'entrée |

### Clés obligatoires — `detail`

```text
object_id
```

> `entry` est optionnel — absent si `get_fn` n'est pas configurée sur la ressource.
> Les variables i18n de la section `detail` sont listées dans les variables globales ci-dessus.

**Exemple minimal :**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_detail_title }} — {{ resource.title }} #{{ object_id }}{% endblock %}

{% block content %}
<h1>{{ admin_detail_title }} — {{ resource.title }} <small>#{{ object_id }}</small></h1>

{% if entry %}
<dl class="row">
    {# itération dynamique sur les champs de l'entrée #}
    {% for key, value in entry %}
    <dt class="col-sm-3">{{ key }}</dt>
    <dd class="col-sm-9">{{ value }}</dd>
    {% endfor %}
</dl>
{% endif %}

<a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_detail_btn_list }}</a>
<a href="/admin/{{ resource_key }}/{{ object_id }}/edit" class="btn btn-primary">{{ admin_detail_btn_edit }}</a>
<a href="/admin/{{ resource_key }}/{{ object_id }}/delete" class="btn btn-danger">{{ admin_detail_btn_delete }}</a>
{% endblock %}
```

---

## Vue `delete`

**Route :** `GET /admin/{resource}/{id}/delete`

| Variable | Type | Description |
| --- | --- | --- |
| `entry` | `Value` *(optionnel)* | Enregistrement sérialisé en JSON — absent si `get_fn` non configurée |
| `object_id` | `String` | ID de l'entrée à supprimer |

### Clés obligatoires — `delete`

```text
object_id
```

> `entry` est optionnel — absent si `get_fn` n'est pas configurée sur la ressource.
> Les variables i18n de la section `delete` sont listées dans les variables globales ci-dessus.

**Exemple minimal :**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_delete_title }} — {{ resource.title }} #{{ object_id }}{% endblock %}

{% block content %}
<h1>{{ admin_delete_heading }}</h1>

<div class="alert alert-danger">
    <strong>{{ admin_delete_warning_title }}</strong>
    <p>{{ admin_delete_warning_desc }}</p>
    {% if entry %}
    <p>{{ admin_delete_warning_of }} <strong>#{{ object_id }}</strong></p>
    {% endif %}
    <p>{{ admin_delete_warning_irreversible }}</p>
</div>

<form method="POST" action="/admin/{{ resource_key }}/{{ object_id }}/delete">
    <button type="submit" class="btn btn-danger">{{ admin_delete_btn_confirm }}</button>
    <a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_delete_btn_cancel }}</a>
</form>
{% endblock %}
```

---

## Pattern recommandé pour les variables i18n

```html
{{ admin_create_title }}
```

> Ne pas utiliser `| default(value="...")` — les variables i18n sont toujours présentes si la langue est configurée.

---

## Surcharger un template

Via `.templates()` dans le builder, remplace le template pour **toutes** les ressources :

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

> Les templates surchargés ont accès aux mêmes variables que les templates par défaut.

## Sous-sections

| Section | Description |
| --- | --- |
| [Surcharge](/docs/fr/admin/template) | remplacer le layout ou un composant CRUD |
| [CSRF](/docs/fr/admin/template) | token CSRF, `csrf.js`, checklist login custom |

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
| [Sommaire](/docs/fr/admin) | Sommaire admin |
