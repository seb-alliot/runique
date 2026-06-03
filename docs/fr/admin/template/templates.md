# Système de templates admin

## Hiérarchie en 3 niveaux

```
admin_template.html   ← niveau 1 : contrat (blocks définis, éléments fixes)
        ↓ extends
admin_base.html            ← niveau 2 : layout visuel par défaut
        ↓ extends
list.html / create.html …  ← niveau 3 : composants CRUD
```

**Niveau 1 — `admin_template.html`** : éléments hors-blocks garantis (CSRF, messages). Ne pas surcharger directement.

**Niveau 2 — `admin_base.html`** : layout par défaut (sidebar, topbar, styles). C'est ce fichier que le dev remplace pour changer l'apparence.

**Niveau 3 — composants** : pages CRUD qui héritent du niveau 2 et remplissent `{% block content %}`.

---

## Blocks disponibles

| Block | Rôle |
| --- | --- |
| `{% block title %}` | Titre de la page (`<title>`) |
| `{% block extra_css %}` | CSS supplémentaires dans `<head>` |
| `{% block layout %}` | Wraps l'ensemble du layout (sidebar + main) |
| `{% block sidebar %}` | Barre latérale de navigation |
| `{% block topbar %}` | Barre supérieure (breadcrumb, logout) |
| `{% block breadcrumb %}` | Fil d'Ariane (défini dans `admin_base`) |
| `{% block messages %}` | Zone de messages flash — contient `{% messages %}` par défaut |
| `{% block content %}` | Contenu principal de la page |
| `{% block extra_js %}` | Scripts JS supplémentaires avant `</body>` |

### Éléments hors-blocks (toujours présents)

Inscrits directement dans `admin_template.html` — **impossibles à supprimer** par surcharge :

- `<meta name="csrf-token" content="{{ csrf_token }}">` dans `<head>`
- `<script src="{{ "js/csrf.js" | runique_static }}" defer></script>` avant `</body>`

---

## Sous-sections

| Section | Description |
| --- | --- |
| [Clés de contexte](/docs/fr/admin/template) | variables injectées par le backend dans chaque template
| [Surcharge](/docs/fr/admin/template) | remplacer le layout ou un composant CRUD
| [CSRF](/docs/fr/admin/template) | token CSRF, `csrf.js`, checklist login custom |

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin dans un projet existant, créer un superuser |
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Permissions](/docs/fr/admin/permission) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution et état bêta |

## Revenir au menu

- [Sommaire](https://github.com/seb-alliot/runique/blob/main/README.fr.md)
