# Clés de contexte Tera — Admin

Variables injectées par le backend dans chaque template admin.

---

## Disponibles partout (middleware auth)

| Variable | Type | Description |
| --- | --- | --- |
| `csrf_token` | `String` | Token CSRF de la session (masqué) |
| `current_user.username` | `String` | Nom d'utilisateur authentifié |
| `current_user.is_staff` | `bool` | Accès à l'admin |
| `current_user.is_superuser` | `bool` | Accès total |
| `current_user.roles` | `Vec<String>` | Rôles personnalisés |

---

## Disponibles sur toutes les pages CRUD

| Variable | Type | Description |
| --- | --- | --- |
| `site_title` | `String` | Titre du site (depuis `AdminConfig`) |
| `resource` | `AdminResource` | Métadonnées de la ressource active |
| `resource.key` | `String` | Clé URL (ex: `"users"`) |
| `resource.title` | `String` | Titre affiché (ex: `"Utilisateurs"`) |
| `resources` | `Vec<AdminResource>` | Toutes les ressources enregistrées |
| `current_resource` | `String` | Clé de la ressource active (= `resource.key`) |

---

## Dashboard

| Variable | Type | Description |
| --- | --- | --- |
| `resource_counts` | `HashMap<String, u64>` | Nombre d'entrées par ressource |
| `current_page` | `String` | Vaut `"dashboard"` |

---

## Login

| Variable | Type | Description |
| --- | --- | --- |
| `error` | `String` (optionnel) | Message d'erreur en cas d'échec |

---

## List (`list.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `entries` | `Vec<JSON Object>` | Enregistrements sérialisés depuis le modèle |
| `total` | `usize` | Nombre total d'entrées |
| `current_page` | `String` | Vaut `"list"` |

---

## Create (`create.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `form_fields` | `Forms` | Formulaire avec HTML pré-rendu et erreurs |
| `is_edit` | `bool` | Vaut `false` |

---

## Edit (`edit.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `form_fields` | `Forms` | Formulaire pré-rempli avec les valeurs existantes |
| `is_edit` | `bool` | Vaut `true` |
| `object_id` | `i32` | Identifiant de l'enregistrement |

---

## Detail & Delete (`detail.html`, `delete.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `entry` | `JSON Object` (optionnel) | Enregistrement sérialisé |
| `object_id` | `i32` | Identifiant de l'enregistrement |

## Sous-sections

| Section | Description |
| --- | --- |
| [Surcharge](../surcharge/surcharge.md) | remplacer le layout ou un composant CRUD
| [CSRF](../csrf/csrf.md) | token CSRF, `csrf.js`, checklist login custom

## Revenir au sommaire

| --- | --- |
| [Sommaire template](../template.md) | Admin
| [Sommaire](../../11-Admin.md) | Sommaire template