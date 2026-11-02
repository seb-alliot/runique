# Surcharge des templates admin

L'apparence de l'admin se change à trois niveaux, du plus léger au plus lourd :

1. **Le thème** : les variables CSS (couleurs, rayons, tailles), voir [Référence des blocs](/docs/fr/admin/template-surcharge-blocks).
2. **Un bloc** : un morceau de page (le tableau de la liste, les boutons du formulaire, la barre latérale…). Vous remplacez son HTML, sa feuille de style, ou les deux.
3. **Une page entière** : la liste, le formulaire de création, le détail…

Tout se déclare dans le builder. Au démarrage :

- un template de bloc absent refuse le boot (Tera vérifie chaque `{% include %}`) ;
- un chemin de template qui ne finit pas par `.html` / `.xml`, ou qui contient `"`, `{`, `}`, `%`, `#`, `\` ou un caractère de contrôle, refuse le boot ;
- un template de **page** absent n'est pas détecté au démarrage : l'erreur arrive au premier affichage de la page ;
- une feuille de style absente n'est pas détectée : la page se charge sans elle (404 sur le fichier).

---

## Comment une page est faite

```
admin/admin_template.html   ← contrat : <head>, meta CSRF, csrf.js, fenêtre modale
        ↓ extends
admin_base.html             ← layout : feuilles de style, barre latérale, barre du haut
        ↓ extends
admin/list.html, admin/create.html …   ← les pages
```

Chaque page est composée de blocs. Dans les templates de Runique, un bloc remplaçable s'écrit `{% dev_block "nom" %}`. Au démarrage, Runique remplace chaque `{% dev_block %}` par un `{% include %}` de votre template si vous en avez déclaré un, sinon par le sien.

---

## Où déclarer

**Pour tous les modèles** : dans `with_admin`, via `.templates(|t| …)` (type `AdminTemplate`).

```rust
use runique::admin::template::{Block, GlobalBlock, Page};

.with_admin(|a| a
    .templates(|t| t
        .block(Block::ListTable, "mon_theme/table.html")
        .global_block(GlobalBlock::Sidebar, "mon_theme/sidebar.html")
    )
    .site(site)
)
```

**Pour un seul modèle** : sur son `ModelAdmin`, via `.templates(|t| …)` (type `ModelTemplates`).

```rust
fn article() -> ModelAdmin<article::Entity> {
    ModelAdmin::new()
        .templates(|t| t.block(Block::ListTable, "articles/table.html"))
}
```

Priorité : ce qui est déclaré sur le modèle, puis ce qui est déclaré pour tous, puis Runique. Le HTML et la feuille de style d'un bloc se résolvent **séparément** : un modèle peut remplacer le HTML d'un bloc et garder la feuille de style déclarée pour tous.

Les chemins :

- un **template** est relatif à votre dossier de templates (`TEMPLATES_DIR`) et doit finir par `.html` ou `.xml` (Tera n'échappe que ces extensions) ;
- une **feuille de style** est relative à votre dossier static (`STATICFILES_DIRS`), servie comme `{% static %}` la sert, avec son empreinte d'intégrité.

---

## Surcharger un bloc

Les blocs sont de deux sortes :

| Sorte | Type | Remplaçable | Exemples |
| --- | --- | --- | --- |
| Bloc des pages d'un modèle | `Block` | pour tous **ou** pour un modèle | `ListTable`, `FormActions`, `DeleteWarning` |
| Bloc du layout, ou d'une page sans modèle | `GlobalBlock` | pour tous seulement | `Sidebar`, `Topbar`, `DashboardTable` |

La liste complète des clés, avec la page qui affiche chaque bloc et le template de Runique à copier comme point de départ, est dans la [référence des blocs](/docs/fr/admin/template-surcharge-blocks).

### Le template seul

Le bloc prend votre HTML, la feuille de style de Runique reste chargée.

```rust
.templates(|t| t.block(Block::DeleteWarning, "mon_theme/delete_warning.html"))
```

```html
{# templates/mon_theme/delete_warning.html #}
<div class="admin-warning">
  <p>
    <strong>{{ admin_delete_warning_title }}</strong>
    {{ resource.title }} #{{ object_id }} sera supprimé définitivement.
  </p>
</div>
```

Le template d'un bloc reçoit le contexte de la page qui l'inclut (ici celui de la page de suppression : `resource`, `object_id`, les traductions `admin_delete_*`…). Voir [Clés de contexte](/docs/fr/admin/template-clef). Le plus sûr est de partir du template de Runique (`runique/templates/admin/blocks/…`) et de le modifier.

### Le CSS seul

Le bloc garde le HTML de Runique et charge votre feuille de style.

```rust
.templates(|t| t.block_stylesheet(Block::ListTable, "css/admin/table.css"))
```

Deux cas :

- le bloc a une feuille de style chez Runique (`ListTable`, `DetailFields`, `DetailGroups`, `DashboardTable`) : **la vôtre la remplace**. Elle doit donc tout redéfinir pour ce bloc, pas seulement ce que vous changez ;
- le bloc n'en a pas : la vôtre **s'ajoute**, après celles de Runique.

### Les deux

Les deux déclarations se cumulent :

```rust
.templates(|t| t
    .block(Block::ListTable, "mon_theme/table.html")
    .block_stylesheet(Block::ListTable, "css/admin/table.css")
)
```

Pour un bloc du layout ou d'une page sans modèle, mêmes trois cas avec `global_block` et `global_block_stylesheet` :

```rust
.templates(|t| t
    .global_block(GlobalBlock::Sidebar, "mon_theme/sidebar.html")
    .global_block_stylesheet(GlobalBlock::Sidebar, "css/admin/sidebar.css")
)
```

---

## Surcharger une page entière

### Pour tous les modèles

| Méthode | Page |
| --- | --- |
| `.with_dashboard(chemin)` | tableau de bord |
| `.with_list(chemin)` | liste |
| `.with_htmx(chemin)` | fragment de liste renvoyé aux requêtes HTMX (recherche, tri, pagination) |
| `.with_create(chemin)` | création |
| `.with_edit(chemin)` | édition |
| `.with_detail(chemin)` | détail |
| `.with_delete(chemin)` | confirmation de suppression |
| `.with_login(chemin)` | connexion |
| `.with_history(chemin)` | historique |
| `.with_history_diff(chemin)` | une entrée de l'historique |

La page des droits et celle du transfert de groupe ne se remplacent pas.

`.with_htmx` ne touche que la réponse HTMX : la page de liste complète continue d'inclure le fragment de Runique. Remplacer l'un sans l'autre donne deux rendus différents selon que la liste est chargée ou rafraîchie.

### Pour un seul modèle

```rust
ModelAdmin::new()
    .templates(|t| t.page(Page::List, "articles/list.html"))
```

Seules `Page::List`, `Page::Create`, `Page::Edit`, `Page::Detail` et `Page::Delete` se remplacent par modèle. Une autre page refuse le boot. La page de détail peut aussi être remplacée par une route ([routes admin déclarées](/docs/fr/admin/declaration)) ; les deux à la fois refusent le boot : la route gagnerait et le template ne serait jamais rendu.

### Le template d'une page

Étendre `admin_base.html` garde le layout de Runique (feuilles de style, barre latérale, barre du haut, CSRF) :

```html
{# templates/articles/list.html #}
{% extends "admin_base.html" %}

{% block title %}{{ resource.title }}{% endblock %}

{% block content %}
<h1>{{ resource.title }} ({{ total }})</h1>
{% for entry in entries %}
  <p><a href="{{ entry.links.detail }}">#{{ entry.id }}</a></p>
{% endfor %}
{% if resource_links.create %}
  <a href="{{ resource_links.create }}" class="btn btn-primary">{{ admin_list_btn_create }}</a>
{% endif %}
{% endblock %}
```

Une page remplacée peut toujours inclure les blocs de Runique avec `{% dev_block "nom" %}` : ils suivent alors les surcharges de blocs déclarées.

### La feuille de style d'une page remplacée

Par défaut, votre page reçoit les feuilles de Runique (layout + blocs de la page). Avec `page_stylesheet`, elle ne charge plus que les **variables** de Runique, puis la vôtre :

```rust
.templates(|t| t
    .with_list("mon_theme/list.html")
    .page_stylesheet(Page::List, "css/admin/list.css")
)
// ou pour un modèle :
ModelAdmin::new().templates(|t| t
    .page(Page::List, "articles/list.html")
    .page_stylesheet(Page::List, "css/admin/articles_list.css")
)
```

`page_stylesheet` n'est lu que si la page est remplacée.

---

## Points d'attention

- Les feuilles de style sont écrites par `{% block extra_css %}` d'`admin_base.html`. Si votre page redéfinit `extra_css`, appelez `{{ super() }}`, sinon plus aucune feuille n'est chargée. Même chose pour `extra_js` (`admin.js`, la modale, le thème).
- La meta CSRF et `csrf.js` sont posés par `admin/admin_template.html`. Une page qui n'étend rien doit les écrire elle-même (voir [Exemples](/docs/fr/admin/template-surcharge-exemples)).
- Les liens viennent du contexte (`nav.*`, `resource_links.*`, `entry.links.*`, `form_action`, `cancel_url`) : ne construisez pas les URL à la main, le préfixe de l'admin et les droits y sont déjà appliqués. Un lien absent veut dire que l'utilisateur n'a pas le droit ; la porte refait la vérification sur la page visée de toute façon.
- `{{ var }}` est échappé, `{{ var | safe }}` ne l'est pas : ne mettez `safe` que sur du HTML que vous produisez vous-même.
- Un fichier de vos templates qui porte le même nom qu'un template de Runique (par exemple `admin/blocks/list/table.html`) le remplace pour tous, sans déclaration ni vérification. Rangez vos templates admin sous un autre dossier (`mon_theme/…`).

## Limites actuelles

- Pas de feuille de thème globale : pour changer les variables CSS sur toutes les pages, il faut passer par un bloc présent partout, par exemple `.global_block_stylesheet(GlobalBlock::Topbar, "css/admin/theme.css")` (toutes les pages sauf la connexion). Voir [Thème CSS](/docs/fr/admin/template-surcharge-blocks).
- `.with_htmx` est global : pas de fragment de liste par modèle.
- Un template de page ou une feuille de style absents ne sont pas signalés au démarrage (voir plus haut).

## Sous-sections

| Section | Description |
| --- | --- |
| [Référence des blocs](/docs/fr/admin/template-surcharge-blocks) | Clés des blocs, pages qui les affichent, variables CSS |
| [Exemples](/docs/fr/admin/template-surcharge-exemples) | Cas complets : bloc, CSS, page, page autonome |
| [Classes CSS](/docs/fr/admin/template-surcharge-classes) | Classes utilisées par les templates de Runique |
| [Clés de contexte](/docs/fr/admin/template-clef) | Variables injectées par le backend dans chaque template |

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
| [Sommaire](/docs/fr/admin) | Sommaire admin |
