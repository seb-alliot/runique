# Admin — variables Tera par page

Les variables que reçoit chaque page de l'admin, et donc tout bloc ou page qui la remplace (voir [Surcharge](/docs/fr/admin/template-surcharge)). La source qui fait foi est `runique/src/admin/router/context.rs` : chaque page y a sa structure (`ListContext`, `DetailContext`…).

Les liens sont toujours fournis tout faits (`nav.*`, `resource_links.*`, `links.*`, `form_action`, `cancel_url`…) : le préfixe de l'admin et les droits de l'utilisateur y sont déjà appliqués. Un lien à `none` veut dire que l'utilisateur n'a pas le droit d'y aller. Ne construisez pas d'URL à la main.

---

## Communes à toutes les pages (sauf la connexion)

| Variable | Contenu |
| --- | --- |
| `site_title` | Titre déclaré par `.site_title(…)` |
| `lang` | Code de la langue courante (`fr`, `en`…) |
| `admin_prefix` | Adresse de base de l'admin (préfixe de montage + chemin), sans `/` final |
| `stylesheets` | Feuilles de style de la page, chacune `{ href, integrity }` ; écrites par `admin_base.html` |
| `resources` | Modèles que l'utilisateur peut voir, dans l'ordre du menu : `{ key, title, list_url }` |
| `current_resource` | Clé du modèle de la page (chaîne), vide sur le tableau de bord |
| `current_page` | `"dashboard"`, `"list"`, `"detail"`, `"create"`, `"edit"`, `"delete"`, `"history"`, `"rights"`, `"transfer"` |
| `current_user` | Compte connecté : `id`, `username`, `rang`, `groupes`, `droits` |
| `nav.dashboard` | Tableau de bord |
| `nav.history` | Historique, `none` si le compte n'en lit aucun |
| `nav.logout` | Où poste le formulaire de déconnexion (POST + CSRF) |
| `nav.site` | Lien « retour au site » |
| `nav.toggle_template` | Bascule du tableau de bord, voir plus bas |
| `csrf_token` | Token CSRF de la requête |
| `admin_base_*` | Traductions du layout |

Et les variables de tout template Runique (`{% messages %}`, `{% csp %}`, `{% static %}`…), voir [Templates](/docs/fr/template).

---

## Traductions

Chaque page reçoit les traductions de sa section, sous la forme `admin.<section>.<clé>` → `{{ admin_<section>_<clé> }}` (par exemple `admin.list.btn_create` → `{{ admin_list_btn_create }}`). La liste des clés est dans `runique/src/utils/constante/admin_key.rs` (`ADMIN_MESSAGE_KEYS`), les textes dans `runique/src/utils/trad/<langue>.json`.

| Page | Sections |
| --- | --- |
| connexion | `login` |
| tableau de bord | `base`, `dashboard` |
| liste | `base`, `list` |
| détail | `base`, `detail`, `list` |
| création | `base`, `create` |
| édition | `base`, `edit` |
| suppression | `base`, `delete` |
| historique, entrée de l'historique | `base`, `history` |
| droits | `base`, `rights` |
| transfert de groupe | `base`, `transfer` |

---

## Connexion

| Variable | Contenu |
| --- | --- |
| `admin_prefix` | Le formulaire poste sur `{{ admin_prefix }}/login` |
| `csrf_token` | À renvoyer dans le champ caché `csrf_token` |
| `error` | Message d'échec (identifiants, CSRF, compte bloqué, session), absent sinon |
| `site_title`, `site_url`, `lang`, `stylesheets` | — |

Champs attendus par le POST : `username`, `password`, `csrf_token`.

---

## Tableau de bord

| Variable | Contenu |
| --- | --- |
| `resource_counts` | Nombre de lignes par modèle : `resource_counts[resource.key]` ; vide si le comptage a échoué |
| `my_groups` | Groupes du compte : `{ name, staff, rights: [{ table, rights }] }` |
| `admin_has_session_override` | `true` quand la bascule a remis le tableau de bord de Runique |

Quand le tableau de bord est remplacé (`.with_dashboard`), `nav.toggle_template` permet d'afficher celui de Runique et d'en revenir ; le choix est gardé en session.

---

## Liste

Aussi rendue, sans layout, pour les requêtes HTMX (`admin/list_partial.html`).

| Variable | Contenu |
| --- | --- |
| `resource` | `{ key, title, display: { list_filter: [(colonne, libellé)] } }` |
| `resource_links` | `{ list, create, bulk, rights }` ; `create`, `bulk`, `rights` à `none` sans le droit |
| `entries` | Lignes de la page : `{ id, values, links: { detail, edit, delete }, taken_by }` |
| `visible_columns` | Colonnes affichées, dans l'ordre (sans la clé) |
| `sortable_columns` | Colonnes triables (déclarées `.sortable()`) |
| `column_labels` | En-tête de chaque colonne, par nom |
| `rich_fields` | Colonnes contenant du HTML, affichées en texte brut |
| `group_actions` | Actions groupées proposées : `{ index, label }` |
| `bulk_delete` | La barre d'actions groupées propose la suppression |
| `return_qs` | Requête courante (recherche, tri, filtres, page), pour revenir au même endroit |
| `filter_values` | Valeurs de chaque filtre, par colonne |
| `active_filters` | Filtres appliqués, colonne → valeur |
| `filter_qs` | Filtres appliqués en query string (`&filter_col=valeur…`) |
| `search` | Recherche en cours |
| `sort_by`, `sort_dir`, `sort_dir_toggle` | Tri en cours (`asc` / `desc`) et sens inverse |
| `total` | Lignes correspondantes, toutes pages |
| `page`, `page_count`, `has_prev`, `has_next`, `prev_page`, `next_page` | Pagination |

`entry.values[colonne]` donne la valeur d'une colonne affichée.

---

## Détail

| Variable | Contenu |
| --- | --- |
| `resource`, `resource_key` | Le modèle (`resource_key` = `resource.key`) |
| `object_id` | Clé de la ligne |
| `entry` | La ligne : une entrée par colonne lisible |
| `rich_fields` | Colonnes HTML, affichées assainies |
| `list_url` | Retour à la liste |
| `links` | `{ detail, edit, delete }` |
| `reset_password_url` | Sur un compte, avec le droit de le modifier |
| `rank_choices`, `current_rank`, `rank_url` | Sur un compte dont on peut changer le rang : rangs proposés `(valeur, libellé)`, rang actuel, où poster |
| `rights_url` | Grille de droits d'un admin, si on peut la régler |
| `memberships`, `memberships_url` | Sur un compte : groupes gérés `(nom du champ, nom du groupe, coché)`, où poster |
| `transfer_receivers`, `transfer_url` | Sur un groupe, pour un dirigeant : comptes à qui le confier `(id, nom)`, page de transfert |
| `taken_by` | Groupe repris après un départ : par qui |

---

## Création et édition

| Variable | Contenu |
| --- | --- |
| `resource`, `resource_key` | Le modèle |
| `object_id` | Clé de la ligne (édition seulement) |
| `list_url` | Retour à la liste |
| `form_fields` | Le formulaire : `html` (champs, CSRF et nom du formulaire inclus), `errors`, `form_errors`, `fields` |
| `form_action` | Où poste le formulaire |
| `cancel_url` | « Annuler » : la ligne en édition, la liste en création |
| `cancel_label`, `submit_label` | Textes des boutons |
| `no_fields_label` | Affiché quand le formulaire n'a aucun champ |

Écrivez `{{ form_fields.html }}` : au chargement, Runique le réécrit en `{{ form_fields.html | safe }}` (le HTML du formulaire est produit par Runique, ses valeurs déjà échappées). Les blocs `form_fields` et `form_actions` sont inclus dans la balise `<form>`.

---

## Suppression

| Variable | Contenu |
| --- | --- |
| `resource`, `resource_key` | Le modèle |
| `object_id` | Clé de la ligne |
| `list_url` | Retour à la liste |
| `form_action` | Où poste la confirmation |
| `form_fields` | Formulaire de confirmation, sans champ : `html` (token CSRF et nom), `errors`, `form_errors` |
| `cancel_url` | Retour à la ligne |

---

## Historique

| Variable | Contenu |
| --- | --- |
| `history_url` | Historique complet : où postent les filtres, et leur remise à zéro |
| `entries` | Entrées, les plus récentes d'abord (voir ci-dessous) |
| `history_resources` | Modèles dont le compte lit l'historique : `{ key, title }` |
| `filter_resource`, `filter_object`, `filter_action`, `filter_user` | Filtres en cours |
| `page`, `total_pages` | Pagination |

Une entrée : `{ id, created_at, username, resource_key, resource_title, object_pk, object_label, action, has_changes, links: { user, object, entry } }`. `action` vaut `create`, `update` ou `delete` ; `created_at` est en UTC (`AAAA-MM-JJ HH:MM:SS`) ; `links.entry` est `none` sans changement lisible.

## Entrée de l'historique

| Variable | Contenu |
| --- | --- |
| `history_url` | Retour à l'historique |
| `entry` | L'entrée (même forme que ci-dessus) |
| `changes` | Colonnes changées et lisibles : `{ column, old, new }` |

---

## Droits et transfert

Ces deux pages ne se remplacent pas ; seul le bloc `rights_table` de la page des droits se remplace. Leur contexte (`RightsContext`, `TransferContext`) est décrit dans `context.rs`.

---

## Sous-sections

| Section | Description |
| --- | --- |
| [Surcharge](/docs/fr/admin/template-surcharge) | remplacer un bloc (HTML, CSS ou les deux) ou une page entière |

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
| [Sommaire](/docs/fr/admin) | Sommaire admin |
