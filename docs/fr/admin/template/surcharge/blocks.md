# Référence des blocks Tera — admin

Chaque template admin expose des `{% block %}` que le dev peut surcharger dans son propre template.  
Utiliser `{{ super() }}` pour conserver le contenu par défaut et ajouter autour.

---

## Blocks de layout — `admin_base.html`

Définis dans le layout global, disponibles depuis tout template qui étend `admin_base`.

| Block | Contenu par défaut | Note |
| --- | --- | --- |
| `title` | Titre de la page | — |
| `extra_css` | CSS Runique admin (7 fichiers) | `{{ super() }}` pour cumuler |
| `layout` | Sidebar + zone principale | Avancé — remplace tout |
| `sidebar` | Navigation ressources + historique | — |
| `topbar` | Breadcrumb + bouton site + logout | — |
| `breadcrumb` | Fil d'Ariane (depuis `admin_base`) | — |
| `messages` | Flash messages | Conserver `{% messages %}` |
| `content` | Contenu de la page CRUD | — |
| `extra_js` | `admin.js` | `{{ super() }}` pour cumuler |

---

## `list.html`

| Block | Contenu |
| --- | --- |
| `list_header` | En-tête de page : titre, compteur, bouton Créer |

---

## `list_partial.html` *(swapé par HTMX)*

| Block | Contenu |
| --- | --- |
| `list_search` | Barre de recherche + champs cachés tri/filtres |
| `list_group_action` | Barre d'actions groupées (sélection + actions bulk) |
| `list_table` | Tableau principal + état vide |
| `list_pagination` | Contrôles de pagination |
| `list_filters` | Sidebar des filtres par colonne |

---

## `create.html`

| Block | Contenu |
| --- | --- |
| `create_header` | En-tête de page |
| `create_form` | Card complète avec formulaire |
| `create_form_fields` | Grille des champs + champs M2M |
| `create_form_actions` | Boutons Annuler / Créer |
| `create_denied` | Message accès refusé |

---

## `edit.html`

| Block | Contenu |
| --- | --- |
| `edit_header` | En-tête de page |
| `edit_form` | Card complète avec formulaire |
| `edit_form_fields` | Grille des champs + champs M2M |
| `edit_form_actions` | Boutons Annuler / Enregistrer |
| `edit_denied` | Message accès refusé |

---

## `detail.html`

| Block | Contenu |
| --- | --- |
| `detail_header` | En-tête de page (inclut `detail_actions`) |
| `detail_actions` | Boutons Éditer / Supprimer / Reset mdp + menu mobile |
| `detail_table` | Card avec tableau clé → valeur |

---

## `delete.html`

| Block | Contenu |
| --- | --- |
| `delete_header` | En-tête de page |
| `delete_warning` | Bandeau d'avertissement |
| `delete_actions` | Boutons Annuler / Confirmer la suppression |
| `delete_denied` | Message accès refusé |

---

## `bulk_edit.html`

| Block | Contenu |
| --- | --- |
| `group_edit_header` | En-tête de page |
| `group_edit_fields` | Section des champs non-booléens |
| `group_edit_permissions` | Section des permissions booléennes (remplie par JS) |
| `group_edit_actions` | Boutons Annuler / Appliquer |
| `group_edit_denied` | Message accès refusé |

---

## `dashboard.html`

| Block | Contenu |
| --- | --- |
| `dashboard_header` | En-tête de page |
| `dashboard_stats` | Grille des stat-cards par ressource |
| `dashboard_table` | Tableau récapitulatif des ressources |

---

## Thème CSS — custom properties

Pour modifier les couleurs et espacements sans réécrire de HTML, surcharger les variables dans `{% block extra_css %}` :

```html
{% block extra_css %}
{{ super() }}
<style>
  :root {
    --accent:       #e11d48;
    --accent-hover: #be123c;
    --bg-main:      #fafafa;
    --bg-card:      #ffffff;
    --bg-sidebar:   #1e1e2e;
    --text-main:    #111827;
  }
</style>
{% endblock %}
```

| Variable | Rôle |
| --- | --- |
| `--bg-main` | Fond principal |
| `--bg-card` | Fond des cards |
| `--bg-sidebar` | Fond de la sidebar |
| `--bg-input` | Fond des champs |
| `--bg-hover` | Fond au survol |
| `--bg-active` | Fond de l'élément actif |
| `--text-main` | Texte principal |
| `--text-muted` | Texte secondaire |
| `--text-sidebar` | Texte de la sidebar |
| `--accent` | Couleur d'accentuation (boutons, liens actifs) |
| `--accent-hover` | Accentuation au survol |
| `--accent-light` | Accentuation translucide |
| `--border` | Bordure standard |
| `--border-light` | Bordure claire |
| `--success` / `--danger` / `--warning` | Couleurs sémantiques |
| `--sidebar-width` | Largeur sidebar déployée |
| `--sidebar-collapsed` | Largeur sidebar réduite |
| `--topbar-height` | Hauteur de la topbar |
| `--radius` / `--radius-lg` | Rayons de bordure |
| `--shadow` | Ombre des cards |
| `--transition` | Durée/easing des transitions |

> Les classes CSS par block sont documentées dans la [référence des classes CSS](/docs/fr/admin/template/surcharge/classes). Le renommage BEM est prévu en v2.2.

---

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Surcharge templates](/docs/fr/admin/template/surcharge/surcharge) | Principe, niveaux d'héritage, exemples |
| [Clés de contexte](/docs/fr/admin/template/clef/context) | Variables injectées par le backend |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
