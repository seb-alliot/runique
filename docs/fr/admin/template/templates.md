# Système de templates admin

## Hiérarchie

```
admin/admin_template.html   ← contrat : <head>, meta CSRF, csrf.js, fenêtre modale
        ↓ extends
admin_base.html             ← layout : feuilles de style, barre latérale, barre du haut
        ↓ extends
admin/list.html, admin/create.html …   ← les pages, faites de blocs
```

**`admin/admin_template.html`** : le squelette HTML et ce qui doit toujours être là. Ne se remplace pas.

**`admin_base.html`** : le layout. Il ne se remplace pas d'un bloc : on en change la barre latérale et la barre du haut par leurs blocs (`GlobalBlock::Sidebar`, `GlobalBlock::Topbar`), et ses feuilles de style par les surcharges de CSS.

**Les pages** : chacune est faite de blocs remplaçables (`{% dev_block "nom" %}`). On remplace un bloc, ou la page entière, pour tous les modèles ou pour un seul. Voir [Surcharge](/docs/fr/admin/template-surcharge).

---

## Blocs Tera du squelette

Une page qui étend `admin_base.html` (ou `admin/admin_template.html`) peut redéfinir :

| Bloc | Rôle |
| --- | --- |
| `{% block lang %}` | La balise `<html lang>` |
| `{% block title %}` | Titre de la page (`<title>`) |
| `{% block extra_css %}` | Feuilles de style dans `<head>` : `admin_base.html` y écrit celles de la page, garder `{{ super() }}` |
| `{% block layout %}` | Tout le layout (barre latérale + zone principale) |
| `{% block sidebar %}` | Barre latérale |
| `{% block topbar %}` | Barre du haut (contient `breadcrumb`) |
| `{% block breadcrumb %}` | Fil d'Ariane |
| `{% block messages %}` | Messages flash, contient `{% messages %}` |
| `{% block content %}` | Contenu de la page |
| `{% block extra_js %}` | Scripts avant `</body>` : `admin_base.html` y met ceux de l'admin, garder `{{ super() }}` |
| `{% block footer %}` | Pied de page |

### Hors blocs (toujours présents)

Écrits directement dans `admin/admin_template.html`, aucune surcharge ne les retire (sauf une page qui n'étend rien) :

- `<meta name="csrf-token" content="{{ csrf_token }}">` et `<meta name="robots" content="noindex, nofollow">` dans `<head>`
- `<script {% csp %} src="{% runique_static "js/csrf.js" %}" defer></script>` avant `</body>`
- la fenêtre modale de confirmation

---

## Sous-sections

| Section | Description |
| --- | --- |
| [Clés de contexte](/docs/fr/admin/template-clef) | variables injectées par le backend dans chaque template
| [Surcharge](/docs/fr/admin/template-surcharge) | remplacer un bloc (HTML, CSS ou les deux) ou une page entière |

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin dans un projet existant, créer un superuser |
| [Déclaration](/docs/fr/admin/declaration) | Workflow `AdminStaging`, `AdminSite`, `ModelAdmin` |
| [Permissions](/docs/fr/admin/permission) | Rangs, droits par compte et par groupe, vérification à chaque requête |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution et état aperçu |

## Revenir au menu

- [Sommaire](https://github.com/seb-alliot/runique/blob/main/README.fr.md)
