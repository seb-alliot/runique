# Renommage des clés de traduction : proposition

Produit le 2026-11-23. **Proposition à relire : rien n'est appliqué**, ni dans le code ni dans les fichiers de traduction. Les noms viennent de règles automatiques (section 1) ; ceux marqués « à valider » sont ceux où la règle a hésité.

Voir `tableau_i18n_cles.md` pour l'état actuel et les usages détaillés.

## 1. La règle

`sujet.nature.clé` — le sujet nomme le fichier, la nature la sous-section.

| Nature | Ce que c'est |
|---|---|
| `error` | ce que voit l'utilisateur quand quelque chose est refusé ou échoue |
| `label` | titres, boutons, libellés, en-têtes de colonnes |
| `hint` | aide de saisie, ou ce qu'il faut faire ensuite |
| `log` | ligne écrite dans le journal |
| `info` | **ajoutée par rapport à ta liste** : message qui annonce ce qui vient de se passer (« Projet créé », « Entrée enregistrée », « Serveur lancé ») ; ni un refus, ni un libellé |

Pour `admin` (253 clés), un niveau de plus : `admin.écran.nature.clé`, un fichier par écran (`fr/admin/login.json`). Sans ce niveau, `title` existerait 10 fois sous `admin.label`. Les lignes de journal de l'admin vont dans `admin.log`.

Dans le nom de la clé, ce qui répète la nature est retiré : `hint_slug` → `hint.slug`, `upload_dir_error` → `error.upload_dir`.

## 2. Ce que deviennent les thèmes

| Thème actuel | Devient | Remarque |
|---|---|---|
| `TemplateMessage` | `debug` (30) | fusionné avec `debug` ; préfixe `debug_` retiré |
| `admin` | `admin` (222), `superuser` (31) | l'assistant du superuser (31 clés, utilisé par `runique-cli`) sort dans `superuser` |
| `atomic` | `atomic` (16) |  |
| `auth` | `auth` (2) |  |
| `build` | `build` (28) |  |
| `cli` | `cli` (14) |  |
| `csrf` | `csrf` (5) |  |
| `database` | `database` (12) |  |
| `debug` | `debug` (2) | reçoit `TemplateMessage` ; ses 2 clés étaient des doublons |
| `error` | `error` (22) | donne `error.error.…` : le nom du sujet est à revoir |
| `flash` | `flash` (6) | 6 clés sans aucune référence : à supprimer ? |
| `forms` | `forms` (95) |  |
| `html` | `http` (16) | renommé `http` (pages 400, 404, 500…) |
| `log` | `csrf` (17), `admin` (14), `session` (6), `forms` (5), `server` (4), `auth` (3), `build` (2), `mailer` (1), `database` (1) | **disparaît** : chaque ligne rejoint son sujet |
| `makemigrations` | `makemigrations` (23) |  |
| `middleware` | `middleware` (2) |  |
| `migrate` | `migrate` (12) |  |
| `reset` | `reset` (18) |  |
| `runique_test` | `runique_test` (22) |  |
| `server` | `server` (9) |  |
| `test_runner` | `test_runner` (16) |  |
| `title` | `error` (1) | **disparaît** : sa clé unique rejoint `error` |

## 3. Les sujets obtenus

| Sujet | Clés | error | label | hint | log | info | Utilisé par |
|---|---:|---:|---:|---:|---:|---:|---|
| `admin` | 236 | 27 | 184 |  | 14 | 11 | runique |
| `atomic` | 16 | 16 |  |  |  |  | runique_atomic |
| `auth` | 5 | 2 |  |  | 3 |  | runique |
| `build` | 30 | 17 | 2 | 9 | 2 |  | runique |
| `cli` | 14 | 7 |  | 3 |  | 4 | runique-cli |
| `csrf` | 22 | 5 |  |  | 17 |  | runique |
| `database` | 13 | 11 |  |  | 1 | 1 | runique |
| `debug` | 32 |  | 32 |  |  |  |  |
| `error` | 23 | 15 | 8 |  |  |  | runique |
| `flash` | 6 | 2 |  |  |  | 4 |  |
| `forms` | 100 | 89 |  | 6 | 5 |  | runique, runique_i18n |
| `http` | 16 | 8 | 8 |  |  |  | runique |
| `mailer` | 1 |  |  |  | 1 |  | runique |
| `makemigrations` | 23 | 16 |  | 1 |  | 6 | runique-cli |
| `middleware` | 2 | 2 |  |  |  |  | runique |
| `migrate` | 12 | 8 |  |  |  | 4 | runique, runique-cli |
| `reset` | 18 | 7 | 10 |  |  | 1 | runique |
| `runique_test` | 22 | 20 | 2 |  |  |  | runique |
| `server` | 13 | 1 |  | 1 | 4 | 7 | runique |
| `session` | 6 |  |  |  | 6 |  | runique |
| `superuser` | 31 | 8 | 14 |  |  | 9 | runique-cli |
| `test_runner` | 16 | 16 |  |  |  |  | runique-cli |

22 thèmes aujourd'hui, 22 sujets ensuite.

## 4. Clés fusionnées par la règle

Deux anciennes clés qui reçoivent le même nouveau nom **et** ont le même texte en français et en anglais, à la ponctuation près : une seule clé suffit.

| Nouvelle clé | Anciennes clés | Texte (fr) |
|---|---|---|
| `debug.label.stack_trace_title` | `TemplateMessage.debug_stack_trace_title`<br>`debug.stack_trace_title` | Trace d'erreur détaillée |

## 5. Clés sans référence dans le code

Proposées à la suppression (voir la section 6 de `tableau_i18n_cles.md`). À vérifier une par une.

| Clé actuelle | Texte (fr) |
|---|---|
| `csrf.forbidden` | Token CSRF invalide |
| `csrf.session_write_error` | Erreur d'écriture de session |
| `error.context_occurred` | Erreur AppError survenue |
| `error.title.database` | Erreur base de données |
| `error.title.forbidden` | Accès interdit |
| `error.title.template` | Erreur de rendu de template |
| `error.title.validation` | Erreur de validation |
| `flash.error_demo` | Ceci est un message d'erreur. |
| `flash.fix_errors` | Veuillez corriger les erreurs |
| `flash.info_demo` | Ceci est un message d'information. |
| `flash.success_demo` | Ceci est un message de succès. |
| `flash.warning_demo` | Ceci est un message d'avertissement. |
| `flash.welcome` | Bienvenue {}, votre compte est créé ! |
| `forms.aegis_urlencoded_error` | Échec parsing urlencoded : {} |
| `forms.boolean_required` | Vous devez accepter ce champ |
| `forms.decimal_required` | Doit être au format décimal |
| `forms.filter_invalid_value` | Le filtre static/media a reçu une valeur invalide ({}) au lieu d'un c… |
| `reset.invalid_link` | Lien invalide. |

## 6. Clés fabriquées à l'exécution

Le code construit leur nom : le renommage se fait dans le modèle, à la main.

| Modèle actuel | Fichier | Exemple : avant → après |
|---|---|---|
| `TemplateMessage.{key}` | runique/src/middleware/errors/error.rs | `TemplateMessage.debug_info_title` → `debug.label.info_title` |
| `admin.rank.{rank}` | runique/src/admin/router/account/summary.rs | `admin.rank.superuser` → `admin.rank.label.superuser` |
| `admin.rank.{value}` | runique/src/admin/router/permission/rank.rs | `admin.rank.superuser` → `admin.rank.label.superuser` |
| `admin.rights.col_{}` | runique/src/admin/router/permission/transfer.rs | `admin.rights.col_read` → `admin.rights.label.col_read` |
| `admin.rights.right_{key}` | runique/src/admin/router/admin_router.rs | `admin.rights.right_create` → `admin.rights.label.right_create` |
| `admin.rights.right_{}` | runique/src/admin/router/permission/transfer.rs | `admin.rights.right_create` → `admin.rights.label.right_create` |
| `admin.{section}.` | runique/src/admin/trad/mod.rs | toute une section de `admin` est envoyée aux templates d'un coup |
| `html.{code}_text` | runique/src/middleware/errors/error.rs | `html.400_title` → `http.label.400` |
| `html.{code}_title` | runique/src/middleware/errors/error.rs | `html.400_title` → `http.label.400` |

## 7. À valider en priorité

Clés où aucune règle de nature n'a tranché (la nature par défaut du thème a été prise), ou dont le nom a dû être gardé pour éviter une confusion.

| Clé actuelle | Proposée | Texte (fr) | Pourquoi |
|---|---|---|---|
| `TemplateMessage.debug_stack_trace_tip` | `debug.label.debug_stack_trace_tip` | L'erreur la plus haute (niveau 0) est généra… | nom gardé : deux textes différents auraient eu le même nom |
| `build.subscriber_already_installed` | `build.error.subscriber_already_installed` | Un subscriber tracing est déjà installé ; la… | nature par défaut du thème |
| `build.external_outputs_ignored` | `build.error.external_outputs_ignored` | `with_log(\|l\| l.external())` est défini ; … | nature par défaut du thème |
| `database.url_sqlite_default_file` | `database.error.url_sqlite_default_file` | DATABASE_URL demande SQLite sans nommer de f… | nature par défaut du thème |
| `debug.stack_trace_tip` | `debug.label.stack_trace_tip` | L'erreur la plus haute (niveau 0) est généra… | nom gardé : deux textes différents auraient eu le même nom |
| `forms.data_password` | `forms.error.data_password` | Le mot de passe de « {} » n'a pas pu être ha… | nature par défaut du thème |
| `makemigrations.project_pk_cargo` | `makemigrations.error.project_pk_cargo` | Impossible de lire les features du projet av… | nature par défaut du thème |
| `makemigrations.project_pk_no_runique` | `makemigrations.error.project_pk_no_runique` | `runique` n'est pas une dépendance du projet… | nature par défaut du thème |
| `makemigrations.destructive_detected` | `makemigrations.error.destructive_detected` | Changements destructifs détectés : | nature par défaut du thème |
| `makemigrations.destructive_require_force` | `makemigrations.error.destructive_require_force` | Utilisez --force pour appliquer ces changeme… | nature par défaut du thème |
| `makemigrations.long_identifier_detected` | `makemigrations.error.long_identifier_detected` | Identifiants SQL générés trop longs (limite … | nature par défaut du thème |
| `makemigrations.type_change_write_by_hand` | `makemigrations.error.type_change_write_by_hand` | Écrivez à la main le up() et le down() de ce… | nature par défaut du thème |
| `makemigrations.database_unreachable` | `makemigrations.error.database_unreachable` | Connexion à la base impossible : {} | nature par défaut du thème |
| `migrate.sqlite_key_change` | `migrate.error.sqlite_key_change` | {} : cette migration ajoute ou retire une cl… | nature par défaut du thème |
| `migrate.renamed_from_kept` | `migrate.error.renamed_from_kept` | {} : les `renamed_from` appliqués n'ont pas … | nature par défaut du thème |
| `runique_test.env_line_malformed` | `runique_test.error.env_line_malformed` | impossible d'analyser {} : une ligne est mal… | nature par défaut du thème |
| `runique_test.transaction_ended` | `runique_test.error.transaction_ended` | la transaction du test a été terminée depuis… | nature par défaut du thème |
| `server.acme_add_feature` | `server.info.acme_add_feature` | Ajoute la feature `acme` à ton Cargo.toml. | nature par défaut du thème |
| `server.test_utils_feature_enabled` | `server.info.test_utils_feature_enabled` | Feature `test-utils` compilée dans ce build … | nature par défaut du thème |

## 8. Toutes les clés : avant → après

Colonne « Usages » : appels dans le code Rust + templates où la variable apparaît. `exéc.` = nom fabriqué à l'exécution. `—` = aucune référence.

### `TemplateMessage` (30 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `debug_info_title` | `debug.label.info_title` | label | Informations de debug | exéc. |
| `debug_field_title` | `debug.label.field_title` | label | Titre | exéc. |
| `debug_field_count` | `debug.label.field_count` | label | Nombre de champs | exéc. |
| `debug_error_count` | `debug.label.error_count` | label | Erreurs | exéc. |
| `debug_main_message` | `debug.label.main_message` | label | Message principal | exéc. |
| `debug_root_cause` | `debug.label.root_cause` | label | Cause racine | exéc. |
| `debug_details` | `debug.label.details` | label | Détails supplémentaires | exéc. |
| `debug_request_info` | `debug.label.request_info` | label | Informations de la requête | exéc. |
| `debug_http_method` | `debug.label.http_method` | label | Méthode HTTP | exéc. |
| `debug_path` | `debug.label.path` | label | Chemin | exéc. |
| `debug_query_params` | `debug.label.query_params` | label | Paramètres de requête | exéc. |
| `debug_http_headers` | `debug.label.http_headers` | label | En-têtes HTTP | exéc. |
| `debug_template_info` | `debug.label.template_info` | label | Informations du template | exéc. |
| `debug_template_name` | `debug.label.template_name` | label | Nom du template | exéc. |
| `debug_error_line` | `debug.label.error_line` | label | Ligne de l'erreur | exéc. |
| `debug_template_source` | `debug.label.template_source` | label | Code source du template | exéc. |
| `debug_available_templates` | `debug.label.available_templates` | label | Templates disponibles | exéc. |
| `debug_stack_trace_title` | `debug.label.stack_trace_title` | label | Trace d'erreur détaillée | exéc. |
| `debug_stack_trace_tip` | `debug.label.debug_stack_trace_tip` | label | L'erreur la plus haute (niveau 0) est généralem… | exéc. |
| `debug_environment` | `debug.label.environment` | label | Environnement | exéc. |
| `debug_debug_mode` | `debug.label.debug_mode` | label | Mode debug | exéc. |
| `debug_enabled` | `debug.label.enabled` | label | Activé | exéc. |
| `debug_disabled` | `debug.label.disabled` | label | Désactivé | exéc. |
| `debug_rust_version` | `debug.label.rust_version` | label | Version Rust | exéc. |
| `debug_app_version` | `debug.label.app_version` | label | Version App | exéc. |
| `debug_runique_version` | `debug.label.runique_version` | label | Version Runique | exéc. |
| `debug_occurred_at` | `debug.label.occurred_at` | label | Erreur survenue le : | exéc. |
| `debug_status_code` | `debug.label.status_code` | label | Code de statut | exéc. |
| `debug_header_name` | `debug.label.header_name` | label | Nom | exéc. |
| `debug_header_value` | `debug.label.header_value` | label | Valeur | exéc. |

### `admin` (253 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `group` | `admin.common.label.group` | label | Groupe | section admin |
| `groups` | `admin.common.label.groups` | label | Groupes | 1 tpl |
| `resource` | `admin.common.label.resource` | label | Ressource | section admin |
| `resources` | `admin.common.label.resources` | label | Ressources | section admin |
| `name` | `admin.common.label.name` | label | Nom | section admin |
| `username` | `admin.common.label.username` | label | Nom d'utilisateur | 1 code |
| `email` | `admin.common.label.email` | label | Email | section admin |
| `staff` | `admin.common.label.staff` | label | Staff | section admin |
| `superuser` | `admin.common.label.superuser` | label | Superutilisateur | section admin |
| `active` | `admin.common.label.active` | label | Actif | section admin |
| `user.username_too_short` | `admin.user.error.username_too_short` | error | Le nom d'utilisateur est trop court (min 3 car.) | section admin |
| `user.invalid_email` | `admin.user.error.invalid_email` | error | Format d'adresse email invalide | section admin |
| `user.not_activated` | `admin.user.error.not_activated` | error | Ce compte n'a pas encore été activé par son uti… | section admin |
| `login.title` | `admin.login.label.title` | label | Connexion | 1 tpl |
| `login.subtitle` | `admin.login.label.subtitle` | label | Connectez-vous pour accéder au panneau d'admini… | 1 tpl |
| `login.label_username` | `admin.login.label.label_username` | label | Nom d'utilisateur | 1 tpl |
| `login.label_password` | `admin.login.label.label_password` | label | Mot de passe | 1 tpl |
| `login.btn_submit` | `admin.login.label.btn_submit` | label | Se connecter | 1 tpl |
| `login.back_to_site` | `admin.login.label.back_to_site` | label | ← Retour au site | 1 tpl |
| `login.error_session` | `admin.login.error.session` | error | Erreur lors de l'ouverture de session. | 1 code |
| `login.error_credentials` | `admin.login.error.credentials` | error | Identifiants incorrects ou droits insuffisants. | 1 code |
| `login.error_locked` | `admin.login.error.locked` | error | Trop de tentatives. Compte temporairement verro… | 1 code |
| `logout.success` | `admin.logout.info.success` | info | Déconnexion réussie. | liste admin |
| `access.insufficient_rights` | `admin.access.error.insufficient_rights` | error | Droits insuffisants pour accéder à l'administra… | liste admin |
| `access.superuser_only` | `admin.access.error.superuser_only` | error | Action non autorisée | section admin |
| `access.action_not_allowed` | `admin.access.error.action_not_allowed` | error | Action non autorisée | 6 code |
| `access.still_responsable` | `admin.access.error.still_responsable` | error | Ce compte répond encore de groupes : transférez… | 2 code |
| `dashboard.subtitle` | `admin.dashboard.label.subtitle` | label | Vue d'ensemble de l'application | 1 tpl |
| `dashboard.card_resources` | `admin.dashboard.label.card_resources` | label | Ressources | 1 tpl |
| `dashboard.th_resource` | `admin.dashboard.label.th_resource` | label | Ressource | liste admin |
| `dashboard.th_key` | `admin.dashboard.label.th_key` | label | Clé | liste admin |
| `dashboard.th_permissions` | `admin.dashboard.label.th_permissions` | label | Permissions | liste admin |
| `dashboard.th_actions` | `admin.dashboard.label.th_actions` | label | Actions | liste admin |
| `dashboard.btn_list` | `admin.dashboard.label.btn_list` | label | Liste | 1 tpl |
| `dashboard.btn_create` | `admin.dashboard.label.btn_create` | label | Créer | liste admin |
| `dashboard.see_list` | `admin.dashboard.label.see_list` | label | Voir la liste → | liste admin |
| `dashboard.empty_title` | `admin.dashboard.label.empty_title` | label | Aucune ressource enregistrée | 1 tpl |
| `dashboard.empty_desc` | `admin.dashboard.label.empty_desc` | label | Enregistrez vos modèles dans src/admin.rs avec … | 1 tpl |
| `dashboard.kpi_resources` | `admin.dashboard.label.kpi_resources` | label | Ressources | liste admin |
| `dashboard.kpi_entries` | `admin.dashboard.label.kpi_entries` | label | Entrées totales | liste admin |
| `dashboard.kpi_largest` | `admin.dashboard.label.kpi_largest` | label | Plus grande | liste admin |
| `dashboard.th_count` | `admin.dashboard.label.th_count` | label | Entrées | liste admin |
| `dashboard.toggle_template` | `admin.dashboard.label.toggle_template` | label | ← Template configuré | 1 tpl |
| `dashboard.my_groups_title` | `admin.dashboard.label.my_groups_title` | label | Mes groupes | 1 tpl |
| `dashboard.my_groups_staff` | `admin.dashboard.label.my_groups_staff` | label | staff | 1 tpl |
| `dashboard.my_groups_none` | `admin.dashboard.label.my_groups_none` | label | Ce groupe ne donne aucun droit sur les tables. | 1 tpl |
| `list.rank_label` | `admin.list.label.rank_label` | label | Rang | 2 tpl |
| `list.groups_label` | `admin.list.label.groups_label` | label | Groupes | 2 tpl |
| `list.entries_count_one` | `admin.list.label.entries_count_one` | label | entrée | 1 tpl |
| `list.entries_count_many` | `admin.list.label.entries_count_many` | label | entrées | 1 tpl |
| `list.btn_create` | `admin.list.label.btn_create` | label | Créer | 2 tpl |
| `list.th_id` | `admin.list.label.th_id` | label | id | liste admin |
| `list.th_actions` | `admin.list.label.th_actions` | label | Actions | liste admin |
| `list.bool_true` | `admin.list.label.bool_true` | label | Oui | 2 tpl |
| `list.bool_false` | `admin.list.label.bool_false` | label | Non | 2 tpl |
| `list.btn_detail` | `admin.list.label.btn_detail` | label | Détail | 1 tpl |
| `list.btn_edit` | `admin.list.label.btn_edit` | label | Modifier | 1 tpl |
| `list.btn_delete` | `admin.list.label.btn_delete` | label | Supprimer | 1 tpl |
| `list.confirm_delete` | `admin.list.label.confirm_delete` | label | Vraiment supprimer ? | liste admin |
| `list.empty_title` | `admin.list.label.empty_title` | label | Aucune entrée | 1 tpl |
| `list.empty_desc` | `admin.list.label.empty_desc` | label | Commencez par créer un élément. | 1 tpl |
| `list.btn_create_first` | `admin.list.label.btn_create_first` | label | Créer le premier | 1 tpl |
| `list.search_placeholder` | `admin.list.label.search_placeholder` | label | Rechercher... | 1 tpl |
| `list.per_page` | `admin.list.label.per_page` | label | Par page | 1 tpl |
| `list.filter_per_page` | `admin.list.label.filter_per_page` | label | Valeurs par page | 1 tpl |
| `list.bulk_selected` | `admin.list.label.bulk_selected` | label | sélectionné(s) | 1 tpl |
| `list.bulk_delete` | `admin.list.label.bulk_delete` | label | Supprimer | 1 tpl |
| `list.filters` | `admin.list.label.filters` | label | Filtres | 2 tpl |
| `list.filters_hide` | `admin.list.label.filters_hide` | label | Masquer les filtres | 1 tpl |
| `list.filters_clear` | `admin.list.label.filters_clear` | label | ✕ Tout | 1 tpl |
| `list.select_all` | `admin.list.label.select_all` | label | Tout sélectionner | 1 tpl |
| `list.row_details` | `admin.list.label.row_details` | label | Détails | liste admin |
| `list.btn_rights` | `admin.list.label.btn_rights` | label | Droits | 1 tpl |
| `list.action_activate` | `admin.list.label.action_activate` | label | Activer | 1 code |
| `list.action_deactivate` | `admin.list.label.action_deactivate` | label | Désactiver | 1 code |
| `list.responsable_pending` | `admin.list.label.responsable_pending` | label | Responsable à nommer | 2 tpl |
| `list.taken_by` | `admin.list.label.taken_by` | label | repris par | 2 tpl |
| `create.title` | `admin.create.label.title` | label | Créer | 1 tpl |
| `create.breadcrumb` | `admin.create.label.breadcrumb` | label | Créer | 1 tpl |
| `create.card_info` | `admin.create.label.card_info` | label | Informations | 1 tpl |
| `create.no_fields` | `admin.create.label.no_fields` | label | Aucun champ disponible pour le moment. | 1 code |
| `create.btn_cancel` | `admin.create.label.btn_cancel` | label | Annuler | 1 code |
| `create.btn_submit` | `admin.create.label.btn_submit` | label | Créer | 1 code |
| `create.success` | `admin.create.info.success` | info | Entrée créée avec succès ! | 1 code |
| `edit.title` | `admin.edit.label.title` | label | Modifier | 1 tpl |
| `edit.breadcrumb` | `admin.edit.label.breadcrumb` | label | Modifier | 1 tpl |
| `edit.card_info` | `admin.edit.label.card_info` | label | Informations | 1 tpl |
| `edit.no_fields` | `admin.edit.label.no_fields` | label | Aucun champ disponible pour le moment. | 1 code |
| `edit.btn_cancel` | `admin.edit.label.btn_cancel` | label | Annuler | 1 code |
| `edit.btn_submit` | `admin.edit.label.btn_submit` | label | Enregistrer | 1 code |
| `edit.success` | `admin.edit.info.success` | info | Entrée mise à jour avec succès ! | 4 code |
| `detail.title` | `admin.detail.label.title` | label | Détail | 1 tpl |
| `detail.breadcrumb` | `admin.detail.label.breadcrumb` | label | Détail | 1 tpl |
| `detail.entry_label` | `admin.detail.label.entry_label` | label | Entrée | 1 tpl |
| `detail.btn_list` | `admin.detail.label.btn_list` | label | ← Liste | 1 tpl |
| `detail.btn_edit` | `admin.detail.label.btn_edit` | label | Modifier | 1 tpl |
| `detail.btn_delete` | `admin.detail.label.btn_delete` | label | Supprimer | 1 tpl |
| `detail.confirm_delete` | `admin.detail.label.confirm_delete` | label | Supprimer cette entrée ? | liste admin |
| `detail.btn_reset_password` | `admin.detail.label.btn_reset_password` | label | Réinitialiser le mot de passe | 1 tpl |
| `detail.btn_reset_password_short` | `admin.detail.label.btn_reset_password_short` | label | Réinitialiser | liste admin |
| `detail.btn_rights` | `admin.detail.label.btn_rights` | label | Droits | 1 tpl |
| `detail.btn_rank` | `admin.detail.label.btn_rank` | label | Changer le rang | 1 tpl |
| `detail.groups_title` | `admin.detail.label.groups_title` | label | Groupes de ce compte | 1 tpl |
| `detail.btn_save_groups` | `admin.detail.label.btn_save_groups` | label | Attribuer | 1 tpl |
| `detail.transfer_title` | `admin.detail.label.transfer_title` | label | Responsable du groupe | 1 tpl |
| `detail.btn_transfer` | `admin.detail.label.btn_transfer` | label | Transférer | 1 tpl |
| `reset_password.btn` | `admin.reset_password.label.btn` | label | Réinitialiser le mot de passe | section admin |
| `reset_password.success_email` | `admin.reset_password.info.success_email` | info | Lien de réinitialisation envoyé. | 1 code |
| `reset_password.success_link` | `admin.reset_password.info.success_link` | info | Lien de réinitialisation (mailer non configuré)… | section admin |
| `reset_password.error_no_email` | `admin.reset_password.error.no_email` | error | Aucun champ email trouvé pour cet utilisateur. | section admin |
| `reset_password.error_no_mailer` | `admin.reset_password.error.no_mailer` | error | L'envoi de courriel n'est pas configuré sur ce … | 2 code |
| `reset_password.error_unavailable` | `admin.reset_password.error.unavailable` | error | Réinitialisation du mot de passe impossible pou… | 2 code |
| `reset_password.error_send` | `admin.reset_password.error.send` | error | Erreur lors de l'envoi de l'email : {} | section admin |
| `reset_password.email_subject` | `admin.reset_password.label.email_subject` | label | Réinitialisation de votre mot de passe | 1 code |
| `reset_password.email_title` | `admin.reset_password.label.email_title` | label | Réinitialisation du mot de passe | 1 code |
| `reset_password.email_greeting` | `admin.reset_password.label.email_greeting` | label | Bonjour, | 1 code |
| `reset_password.email_body` | `admin.reset_password.label.email_body` | label | Cliquez sur le lien ci-dessous pour réinitialis… | 1 code |
| `reset_password.email_btn` | `admin.reset_password.label.email_btn` | label | Réinitialiser mon mot de passe | 1 code |
| `reset_password.email_ignore` | `admin.reset_password.label.email_ignore` | label | Si vous n'avez pas demandé cette réinitialisati… | 1 code |
| `user_created.email_subject` | `admin.user_created.label.email_subject` | label | Bienvenue — définissez votre mot de passe | 1 code |
| `user_created.email_sent` | `admin.user_created.info.email_sent` | info | Email de bienvenue envoyé. | 1 code |
| `user_created.email_greeting` | `admin.user_created.label.email_greeting` | label | Bienvenue, | 1 code |
| `user_created.email_body` | `admin.user_created.label.email_body` | label | Un compte a été créé pour vous. Cliquez sur le … | 1 code |
| `user_created.email_btn` | `admin.user_created.label.email_btn` | label | Définir mon mot de passe | 1 code |
| `user_created.email_validity` | `admin.user_created.label.email_validity` | label | Ce lien est valide {} min. Si vous n'attendiez … | 1 code |
| `delete.title` | `admin.delete.label.title` | label | Supprimer | 1 tpl |
| `delete.breadcrumb` | `admin.delete.label.breadcrumb` | label | Supprimer | 1 tpl |
| `delete.heading` | `admin.delete.label.heading` | label | Confirmer la suppression | 1 tpl |
| `delete.btn_cancel` | `admin.delete.label.btn_cancel` | label | ← Annuler | 1 tpl |
| `delete.btn_confirm` | `admin.delete.label.btn_confirm` | label | Oui, supprimer définitivement | 1 tpl |
| `delete.success` | `admin.delete.info.success` | info | Entrée supprimée avec succès ! | 1 code |
| `delete.not_found` | `admin.delete.error.not_found` | error | Ressource introuvable. | liste admin |
| `delete.warning.title` | `admin.delete.label.warning_title` | label | Attention ! | 1 tpl |
| `delete.warning.desc` | `admin.delete.label.warning_desc` | label | Vous êtes sur le point de supprimer l'entrée | 1 tpl |
| `delete.warning.of` | `admin.delete.label.warning_of` | label | de la ressource | 1 tpl |
| `delete.warning.irreversible` | `admin.delete.label.warning_irreversible` | label | Cette action est irréversible. | 1 tpl |
| `bulk.no_selection` | `admin.bulk.error.no_selection` | error | Aucun élément sélectionné. | 1 code |
| `bulk.no_field_selected` | `admin.bulk.error.no_field_selected` | error | Aucun champ modifié. | section admin |
| `bulk.delete_success` | `admin.bulk.info.delete_success` | info | élément(s) supprimé(s). | 1 code |
| `bulk.update_success` | `admin.bulk.info.update_success` | info | élément(s) mis à jour. | 1 code |
| `base.title` | `admin.base.label.title` | label | Administration | 2 tpl |
| `base.breadcrumb` | `admin.base.label.breadcrumb` | label | Admin | 1 tpl |
| `base.toggle` | `admin.base.label.toggle` | label | Réduire le menu | 1 tpl |
| `base.logout_title` | `admin.base.label.logout_title` | label | Déconnexion | 1 tpl |
| `base.theme_toggle` | `admin.base.label.theme_toggle` | label | Basculer le thème | 1 tpl |
| `base.back_to_site` | `admin.base.label.back_to_site` | label | ← Site | 1 tpl |
| `base.dashboard` | `admin.base.label.dashboard` | label | Tableau de bord | 2 tpl |
| `base.history` | `admin.base.label.history` | label | Historique | 3 tpl |
| `superuser_wizard.title` | `superuser.label.title` | label | === Créer un superutilisateur === | 1 code |
| `superuser_wizard.algo_prompt` | `superuser.label.algo_prompt` | label | [1/5] Algorithme de hachage | 1 code |
| `superuser_wizard.algo_argon2` | `superuser.label.algo_argon2` | label | Argon2 (recommandé) | 1 code |
| `superuser_wizard.algo_bcrypt` | `superuser.label.algo_bcrypt` | label | Bcrypt | 1 code |
| `superuser_wizard.algo_scrypt` | `superuser.label.algo_scrypt` | label | Scrypt | 1 code |
| `superuser_wizard.algo_custom` | `superuser.label.algo_custom` | label | Fournisseur personnalisé | 1 code |
| `superuser_wizard.provider_path` | `superuser.label.provider_path` | label | Chemin du fournisseur | 1 code |
| `superuser_wizard.invalid_path` | `superuser.error.invalid_path` | error | Chemin invalide. | 1 code |
| `superuser_wizard.username_prompt` | `superuser.label.username_prompt` | label | [2/5] Nom d'utilisateur | 1 code |
| `superuser_wizard.username_empty` | `superuser.error.username_empty` | error | Le nom d'utilisateur ne peut pas être vide. | 1 code |
| `superuser_wizard.username_taken` | `superuser.error.username_taken` | error | Un utilisateur avec ce nom existe déjà. | 1 code |
| `superuser_wizard.email_prompt` | `superuser.label.email_prompt` | label | [3/5] Email | 1 code |
| `superuser_wizard.email_invalid` | `superuser.error.email_invalid` | error | Email invalide. | 1 code |
| `superuser_wizard.email_taken` | `superuser.error.email_taken` | error | Un utilisateur avec cet email existe déjà. | 1 code |
| `superuser_wizard.password_prompt` | `superuser.label.password_prompt` | label | [4/5] Mot de passe | 1 code |
| `superuser_wizard.password_too_short` | `superuser.error.password_too_short` | error | Le mot de passe doit contenir au moins 12 carac… | 3 code |
| `superuser_wizard.password_weak` | `superuser.error.password_weak` | error | Le mot de passe doit contenir au moins une minu… | 2 code |
| `superuser_wizard.confirm_prompt` | `superuser.label.confirm_prompt` | label | [5/5] Confirmer le mot de passe | 1 code |
| `superuser_wizard.password_mismatch` | `superuser.error.password_mismatch` | error | Les mots de passe ne correspondent pas. Veuille… | 1 code |
| `superuser_wizard.review_algo` | `superuser.info.review_algo` | info |   Algorithme  : {} | 1 code |
| `superuser_wizard.review_username` | `superuser.info.review_username` | info |   Utilisateur : {} | 1 code |
| `superuser_wizard.review_email` | `superuser.info.review_email` | info |   Email       : {} | 1 code |
| `superuser_wizard.review_password` | `superuser.info.review_password` | info |   Mot de passe : •••••••• | 1 code |
| `superuser_wizard.review_action` | `superuser.info.review_action` | info | Action | 1 code |
| `superuser_wizard.confirm_create` | `superuser.label.confirm_create` | label | Confirmer et créer | 1 code |
| `superuser_wizard.change_algo` | `superuser.label.change_algo` | label | Changer d'algorithme | 1 code |
| `superuser_wizard.modify_password` | `superuser.label.modify_password` | label | Modifier le mot de passe | 1 code |
| `superuser_wizard.success` | `superuser.info.success` | info | ✓ Superutilisateur créé avec succès ! | 1 code |
| `superuser_wizard.id_line` | `superuser.info.id_line` | info |   ID          : {} | 1 code |
| `superuser_wizard.username_line` | `superuser.info.username_line` | info |   Utilisateur : {} | 1 code |
| `superuser_wizard.email_line` | `superuser.info.email_line` | info |   Email       : {} | 1 code |
| `builtin.invalid_id` | `admin.builtin.error.invalid_id` | error | Identifiant invalide. | section admin |
| `builtin.droit_resource_required` | `admin.builtin.error.droit_resource_required` | error | Veuillez sélectionner au moins une ressource. | section admin |
| `builtin.droit_exists` | `admin.builtin.error.droit_exists` | error | Le droit "{}" existe déjà. | section admin |
| `builtin.groupe_exists` | `admin.builtin.error.groupe_exists` | error | Un groupe nommé « {} » existe déjà. | section admin |
| `builtin.user_username_exists` | `admin.builtin.error.user_username_exists` | error | Le nom d'utilisateur « {} » est déjà pris. | section admin |
| `builtin.user_email_exists` | `admin.builtin.error.user_email_exists` | error | L'adresse e-mail « {} » est déjà utilisée. | section admin |
| `bulk_create.all_exist` | `admin.bulk_create.error.all_exist` | error | Toutes les valeurs existent déjà : rien n'a été… | section admin |
| `history.subtitle` | `admin.history.label.subtitle` | label | Les créations, modifications et suppressions fa… | 1 tpl |
| `history.whole_row` | `admin.history.label.whole_row` | label | Toute l'histoire de cette ligne | 1 tpl |
| `history.th_column` | `admin.history.label.th_column` | label | Colonne | 1 tpl |
| `history.th_old` | `admin.history.label.th_old` | label | Avant | 1 tpl |
| `history.th_new` | `admin.history.label.th_new` | label | Après | 1 tpl |
| `history.no_changes` | `admin.history.label.no_changes` | label | Aucun changement lisible | 1 tpl |
| `history.all_resources` | `admin.history.label.all_resources` | label | Toutes les ressources | 1 tpl |
| `history.all_actions` | `admin.history.label.all_actions` | label | Toutes les actions | 1 tpl |
| `history.action_create` | `admin.history.label.action_create` | label | Créé | 1 tpl |
| `history.action_update` | `admin.history.label.action_update` | label | Modifié | 1 tpl |
| `history.action_delete` | `admin.history.label.action_delete` | label | Supprimé | 1 tpl |
| `history.btn_filter` | `admin.history.label.btn_filter` | label | Filtrer | 1 tpl |
| `history.btn_reset` | `admin.history.label.btn_reset` | label | Réinitialiser | 1 tpl |
| `history.th_date` | `admin.history.label.th_date` | label | Date | 1 tpl |
| `history.th_account` | `admin.history.label.th_account` | label | Compte | 1 tpl |
| `history.th_resource` | `admin.history.label.th_resource` | label | Ressource | 1 tpl |
| `history.th_object` | `admin.history.label.th_object` | label | Objet | 1 tpl |
| `history.th_action` | `admin.history.label.th_action` | label | Action | 1 tpl |
| `history.btn_entry` | `admin.history.label.btn_entry` | label | Détail | 1 tpl |
| `history.empty` | `admin.history.label.empty` | label | Aucune action enregistrée | 1 tpl |
| `history.badge_create` | `admin.history.label.badge_create` | label | créé | 1 tpl |
| `history.badge_update` | `admin.history.label.badge_update` | label | modifié | 1 tpl |
| `history.badge_delete` | `admin.history.label.badge_delete` | label | supprimé | 1 tpl |
| `history.badge_rights` | `admin.history.label.badge_rights` | label | droits modifiés | 1 tpl |
| `history.badge_rank` | `admin.history.label.badge_rank` | label | rang modifié | 1 tpl |
| `history.badge_reset_password` | `admin.history.label.badge_reset_password` | label | lien de mot de passe envoyé | 1 tpl |
| `history.badge_groups` | `admin.history.label.badge_groups` | label | Groupes | 1 tpl |
| `history.badge_transfer` | `admin.history.label.badge_transfer` | label | Transfert | 1 tpl |
| `rights.title` | `admin.rights.label.title` | label | Droits | 1 tpl |
| `rights.subtitle` | `admin.rights.label.subtitle` | label | Ce que cet admin peut faire, table par table et… | 1 tpl |
| `rights.go_to_table` | `admin.rights.label.go_to_table` | label | Aller à une table | 1 tpl |
| `rights.toc` | `admin.rights.label.toc` | label | Tables | 1 tpl |
| `rights.btn_cancel` | `admin.rights.label.btn_cancel` | label | Annuler | 1 tpl |
| `rights.btn_save` | `admin.rights.label.btn_save` | label | Enregistrer | 1 tpl |
| `rights.th_column` | `admin.rights.label.th_column` | label | Colonne | 1 tpl |
| `rights.col_read` | `admin.rights.label.col_read` | label | Lire | 1 tpl, exéc. |
| `rights.col_create` | `admin.rights.label.col_create` | label | Créer | 1 tpl, exéc. |
| `rights.col_update` | `admin.rights.label.col_update` | label | Modifier | 1 tpl, exéc. |
| `rights.right_create` | `admin.rights.label.right_create` | label | Créer | 1 code, exéc. |
| `rights.right_read` | `admin.rights.label.right_read` | label | Lister / lire | 1 code, exéc. |
| `rights.right_read_own` | `admin.rights.label.right_read_own` | label | Lire les siennes | 1 code, exéc. |
| `rights.right_update` | `admin.rights.label.right_update` | label | Modifier | 1 code, exéc. |
| `rights.right_update_own` | `admin.rights.label.right_update_own` | label | Modifier les siennes | 1 code, exéc. |
| `rights.right_delete` | `admin.rights.label.right_delete` | label | Supprimer | 1 code, exéc. |
| `rights.right_delete_own` | `admin.rights.label.right_delete_own` | label | Supprimer les siennes | 1 code, exéc. |
| `rights.group_subtitle` | `admin.rights.label.group_subtitle` | label | Ce que les membres de ce groupe peuvent faire, … | 1 tpl |
| `rights.group_label` | `admin.rights.label.group_label` | label | Groupe | 1 tpl |
| `rights.btn_show` | `admin.rights.label.btn_show` | label | Afficher | 1 tpl |
| `rights.group_sheet` | `admin.rights.label.group_sheet` | label | Fiche du groupe | 1 tpl |
| `rank.superuser` | `admin.rank.label.superuser` | label | Superutilisateur | exéc. |
| `rank.lambda` | `admin.rank.label.lambda` | label | Compte du site | exéc. |
| `rank.admin` | `admin.rank.label.admin` | label | Admin | exéc. |
| `rank.dirigeant` | `admin.rank.label.dirigeant` | label | Dirigeant | exéc. |
| `rank.success` | `admin.rank.info.success` | info | Rang modifié ({}) | 1 code, exéc. |
| `transfer.title` | `admin.transfer.label.title` | label | Transférer le groupe | 1 tpl |
| `transfer.from` | `admin.transfer.label.from` | label | Responsable actuel | 1 tpl |
| `transfer.to` | `admin.transfer.label.to` | label | Nouveau responsable | 1 tpl |
| `transfer.losses` | `admin.transfer.label.losses` | label | Droits en jeu | 1 tpl |
| `transfer.no_loss` | `admin.transfer.label.no_loss` | label | Le groupe garde tous ses droits. | 1 tpl |
| `transfer.btn_confirm` | `admin.transfer.label.btn_confirm` | label | Confirmer le transfert | 1 tpl |
| `transfer.btn_cancel` | `admin.transfer.label.btn_cancel` | label | Annuler | 1 tpl |
| `transfer.unsuitable` | `admin.transfer.error.unsuitable` | error | Cet utilisateur ne convient pas. | 3 code |
| `transfer.ranked_members` | `admin.transfer.error.ranked_members` | error | Ce groupe compte des membres avec un rang : un … | 1 code |
| `transfer.changed` | `admin.transfer.error.changed` | error | Les droits ont changé entre-temps : vérifiez à … | 1 code |
| `transfer.success` | `admin.transfer.info.success` | info | Groupe transféré. | 1 code |
| `transfer.gained_right` | `admin.transfer.label.gained_right` | label | reçu par | 1 tpl |
| `transfer.lost_right` | `admin.transfer.label.lost_right` | label | perdu par les membres | 1 tpl |

### `atomic` (16 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `reserved_name` | `atomic.error.reserved_name` | error | {} : ce nom est réservé à Runique | 1 code |
| `not_a_file` | `atomic.error.not_a_file` | error | {} : ce n'est pas le chemin d'un fichier | 1 code |
| `duplicate` | `atomic.error.duplicate` | error | {} : ce fichier est déjà dans le lot | 1 code |
| `in_the_way` | `atomic.error.in_the_way` | error | {} occupe la place : ce fichier ne vient pas du… | 1 code |
| `outside_journal` | `atomic.error.outside_journal` | error | {} : hors du dossier du journal | 1 code |
| `leaves_journal` | `atomic.error.leaves_journal` | error | {} : ce chemin sort du dossier du journal | 1 code |
| `journal_forged` | `atomic.error.journal_forged` | error | {} : le journal nomme un chemin que Runique n'a… | 2 code |
| `no_journal` | `atomic.error.no_journal` | error | {} : aucun journal à marquer | 1 code |
| `stray_backup` | `atomic.error.stray_backup` | error | {} : une sauvegarde existe, alors qu'il n'y ava… | 1 code |
| `gone` | `atomic.error.gone` | error | {} : le fichier a disparu, et aucune sauvegarde… | 1 code |
| `not_created` | `atomic.error.not_created` | error | {} : ce n'est pas le fichier que Runique était … | 1 code |
| `changed` | `atomic.error.changed` | error | {} : modifié depuis l'interruption, et sa sauve… | 1 code |
| `stranger` | `atomic.error.stranger` | error | {} : ce fichier ne vient pas de Runique, dans u… | 1 code |
| `moved_out` | `atomic.error.moved_out` | error | {} : ce fichier n'est plus dans le dossier du j… | 1 code |
| `unfinished` | `atomic.error.unfinished` | error | le lot n'a pas abouti après ses passes | 1 code |
| `wrong_step` | `atomic.error.wrong_step` | error | cette action est impossible à cette étape du fi… | 3 code |

### `auth` (2 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `login_cannot_sign_in` | `auth.error.login_cannot_sign_in` | error | Ce compte ne peut pas se connecter : il est ina… | 1 code |
| `login_session_failed` | `auth.error.login_session_failed` | error | La session n'a pas pu être enregistrée : {} | 1 code |

### `build` (28 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `subscriber_already_installed` | `build.error.subscriber_already_installed` | error | Un subscriber tracing est déjà installé ; la co… | 1 code |
| `external_outputs_ignored` | `build.error.external_outputs_ignored` | error | `with_log(\|l\| l.external())` est défini ; les… | 1 code |
| `validation_failed` | `build.error.validation_failed` | error | Échec de la validation du build : {} | 1 code |
| `check_failed` | `build.error.check_failed` | error | Échec du build avec {} erreur(s) de vérification | 1 code |
| `template_failed` | `build.error.template_failed` | error | Échec du chargement du template : {} | 1 code |
| `driver_not_enabled` | `build.error.driver_not_enabled` | error | Pilote {} non activé : Runique a été compilé av… | 1 code |
| `db_missing` | `build.error.db_missing` | error | Connexion à la base de données requise lorsque … | 1 code |
| `component_not_ready` | `build.error.component_not_ready` | error | Le composant '{}' n'est pas prêt | 1 code |
| `check.db_required` | `build.error.db_required` | error | Connexion ou configuration de base de données r… | 1 code |
| `check.db_required_hint` | `build.hint.db_required` | hint | Ajoutez .with_database(db) ou .with_database_co… | 1 code |
| `check.admin_prefix_empty` | `build.error.admin_prefix_empty` | error | Le préfixe de route de l'admin ne peut pas être… | 1 code |
| `check.admin_prefix_empty_hint` | `build.hint.admin_prefix_empty` | hint | Utilisez .prefix("/admin") ou gardez la valeur … | 1 code |
| `check.cache_control_invalid` | `build.error.cache_control_invalid` | error | Valeur d'en-tête Cache-Control invalide : '{}' | 1 code |
| `check.cache_control_invalid_hint` | `build.hint.cache_control_invalid` | hint | N'utilisez que des caractères ASCII visibles (p… | 1 code |
| `check.media_root_unusable` | `build.error.media_root_unusable` | error | Impossible de créer ou d'accéder au dossier MED… | 1 code |
| `check.media_root_unusable_hint` | `build.hint.media_root_unusable` | hint | Définissez MEDIA_ROOT sur un chemin absolu acce… | 1 code |
| `check.secret_key_weak` | `build.error.secret_key_weak` | error | SECRET_KEY est absente, vide, à sa valeur par d… | 1 code |
| `check.secret_key_weak_hint` | `build.hint.secret_key_weak` | hint | Définissez SECRET_KEY à une chaîne aléatoire d'… | 1 code |
| `check.public_url_missing` | `build.error.public_url_missing` | error | Aucune URL publique : les liens de reset seraie… | 1 code |
| `check.public_url_missing_hint` | `build.hint.public_url_missing` | hint | Définissez-la dans le builder : .with_public_ur… | 1 code |
| `check.acme_domain_missing` | `build.error.acme_domain_missing` | error | ACME_ENABLED=true mais ACME_DOMAIN n'est pas dé… | 1 code |
| `check.acme_domain_missing_hint` | `build.hint.acme_domain_missing` | hint | Définissez ACME_DOMAIN à votre domaine de produ… | 1 code |
| `check.acme_email_missing` | `build.error.acme_email_missing` | error | ACME_ENABLED=true mais ACME_EMAIL n'est pas déf… | 1 code |
| `check.acme_email_missing_hint` | `build.hint.acme_email_missing` | hint | Définissez ACME_EMAIL à votre email de contact … | 1 code |
| `check.suggestion` | `build.label.suggestion` | label | Suggestion | 1 code |
| `check.context` | `build.label.context` | label | Contexte | 1 code |
| `check.mailer_missing` | `build.error.mailer_missing` | error | Réinitialisation du mot de passe sans mailer : … | 1 code |
| `check.mailer_missing_hint` | `build.hint.mailer_missing` | hint | Renseignez-le dans le .env : EMAIL_BACKEND, ou … | 1 code |

### `cli` (14 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `creating_project` | `cli.info.creating_project` | info | Création du projet '{}'... | 1 code |
| `name_empty` | `cli.error.name_empty` | error | Le nom du projet ne peut pas être vide | 1 code |
| `name_invalid` | `cli.error.name_invalid` | error | Le nom du projet ne doit contenir que des lettr… | 1 code |
| `name_dash` | `cli.error.name_dash` | error | Le nom du projet ne peut pas commencer par - | 1 code |
| `folder_exists` | `cli.error.folder_exists` | error | Le dossier '{}' existe déjà | 1 code |
| `interrupted_run` | `cli.error.interrupted_run` | error | Un lancement a été interrompu dans '{}' et des … | 1 code |
| `run_repaired` | `cli.info.run_repaired` | info | Un lancement interrompu a été réparé dans '{}'. | 1 code |
| `project_created` | `cli.info.project_created` | info | Projet '{}' créé avec succès ! | 1 code |
| `getting_started` | `cli.info.getting_started` | info | Pour démarrer : | 1 code |
| `cd_hint` | `cli.hint.cd` | hint | cd {} | 1 code |
| `cargo_run_hint` | `cli.hint.cargo_run` | hint | cargo run | 1 code |
| `name_digit` | `cli.error.name_digit` | error | Le nom du projet ne peut pas commencer par un c… | 1 code |
| `name_reserved` | `cli.error.name_reserved` | error | '{}' est réservé (mot-clé ou crate de Rust, nom… | 1 code |
| `migration_up_hint` | `cli.hint.migration_up` | hint | runique migration up      (crée les tables du f… | 1 code |

### `csrf` (5 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `missing` | `csrf.error.missing` | error | Token CSRF manquant | 2 code |
| `invalid` | `csrf.error.invalid` | error | Token CSRF invalide | 1 code |
| `invalid_or_missing` | `csrf.error.invalid_or_missing` | error | Token CSRF invalide ou manquant | 8 code |
| `forbidden` | `csrf.error.forbidden` | error | Token CSRF invalide | — **à supprimer ?** |
| `session_write_error` | `csrf.error.session_write` | error | Erreur d'écriture de session | — **à supprimer ?** |

### `database` (12 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `reason_unknown_start` | `database.error.reason_unknown_start` | error | son début ne désigne aucun moteur pris en charge | 4 code |
| `reason_unreadable` | `database.error.reason_unreadable` | error | elle est illisible | 3 code |
| `reason_no_database` | `database.error.reason_no_database` | error | elle ne nomme aucune base | 5 code |
| `reason_sqlite_no_file` | `database.error.reason_sqlite_no_file` | error | elle demande SQLite sans nommer de fichier | 4 code |
| `url_incomplete_using_keys` | `database.error.url_incomplete_using_keys` | error | DATABASE_URL est incomplète ({}) : connexion pa… | 1 code |
| `url_incomplete_reminder` | `database.info.url_incomplete_reminder` | info | Connexion établie par les clés DB_*. DATABASE_U… | 1 code |
| `url_incomplete_no_keys` | `database.error.url_incomplete_no_keys` | error | DATABASE_URL est incomplète ({}) et aucune clé … | 2 code |
| `url_incomplete_keys_unusable` | `database.error.url_incomplete_keys_unusable` | error | DATABASE_URL est incomplète ({}), et les clés D… | 2 code |
| `url_sqlite_default_file` | `database.error.url_sqlite_default_file` | error | DATABASE_URL demande SQLite sans nommer de fich… | 1 code |
| `keys_without_engine` | `database.error.keys_without_engine` | error | DB_ENGINE manque : {} est renseigné sans le mot… | 3 code |
| `url_failed_trying_keys` | `database.error.url_failed_trying_keys` | error | La connexion par DATABASE_URL a échoué ({}) : n… | 1 code |
| `url_failed_keys_unusable` | `database.error.url_failed_keys_unusable` | error | La connexion par DATABASE_URL a échoué ({}), et… | 1 code |

### `debug` (2 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `stack_trace_title` | `debug.label.stack_trace_title` | label | Trace d'erreur détaillée | 2 tpl |
| `stack_trace_tip` | `debug.label.stack_trace_tip` | label | L'erreur la plus haute (niveau 0) est généralem… | 1 tpl |

### `error` (22 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `source` | `error.error.source` | error | Source: {} | 1 code |
| `AppError` | `error.error.AppError` | error | Erreur d'application | 1 code |
| `build` | `error.error.build` | error | Erreur de build: {} | 2 code |
| `internal` | `error.error.internal` | error | Erreur interne | 2 code |
| `forbidden` | `error.error.forbidden` | error | Accès interdit | 3 code |
| `not_found` | `error.error.not_found` | error | Ressource introuvable | 3 code |
| `validation` | `error.error.validation` | error | Erreur de validation: {} | 2 code |
| `database` | `error.error.database` | error | Erreur base de données: {} | 3 code |
| `io` | `error.error.io` | error | Erreur IO: {} | 2 code |
| `template` | `error.error.template` | error | Erreur template: {} | 3 code |
| `custom` | `error.error.custom` | error | Erreur custom: {} | 1 code |
| `title.bad_request` | `error.label.bad_request` | label | Requête invalide | 1 code |
| `title.not_found` | `error.label.not_found` | label | Page non trouvée | 3 code |
| `title.forbidden` | `error.label.forbidden` | label | Accès interdit | — **à supprimer ?** |
| `title.validation` | `error.label.validation` | label | Erreur de validation | — **à supprimer ?** |
| `title.database` | `error.label.database` | label | Erreur base de données | — **à supprimer ?** |
| `title.template` | `error.label.template` | label | Erreur de rendu de template | — **à supprimer ?** |
| `title.internal` | `error.label.internal` | label | Erreur serveur interne | 1 code |
| `invalid_query` | `error.error.invalid_query` | error | Paramètres d'URL invalides : {} | 1 code |
| `path_not_found` | `error.error.path_not_found` | error | Le chemin demandé '{}' n'a pas été trouvé | 1 code |
| `internal_occurred` | `error.error.internal_occurred` | error | Une erreur interne est survenue | 2 code |
| `context_occurred` | `error.error.context_occurred` | error | Erreur AppError survenue | — **à supprimer ?** |

### `flash` (6 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `welcome` | `flash.info.welcome` | info | Bienvenue {}, votre compte est créé ! | — **à supprimer ?** |
| `fix_errors` | `flash.error.fix_errors` | error | Veuillez corriger les erreurs | — **à supprimer ?** |
| `success_demo` | `flash.info.success_demo` | info | Ceci est un message de succès. | — **à supprimer ?** |
| `info_demo` | `flash.info.info_demo` | info | Ceci est un message d'information. | — **à supprimer ?** |
| `warning_demo` | `flash.info.warning_demo` | info | Ceci est un message d'avertissement. | — **à supprimer ?** |
| `error_demo` | `flash.error.demo` | error | Ceci est un message d'erreur. | — **à supprimer ?** |

### `forms` (95 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `upload_dir_error` | `forms.error.upload_dir` | error | Erreur lors de la création du dossier d'upload | 3 code |
| `file_create_error` | `forms.error.file_create` | error | Erreur lors de la création du fichier | 1 code |
| `multipart_stream_error` | `forms.error.multipart_stream` | error | Erreur dans le flux multipart | 1 code |
| `file_write_error` | `forms.error.file_write` | error | Erreur lors de l'écriture du fichier | 2 code |
| `required` | `forms.error.required` | error | Ce champ est obligatoire | 14 code |
| `too_short` | `forms.error.too_short` | error | Trop court (min {}) | 1 code |
| `too_long` | `forms.error.too_long` | error | Trop long (max {}) | 1 code |
| `email_invalid` | `forms.error.email_invalid` | error | Format d'adresse email invalide | 1 code |
| `url_invalid` | `forms.error.url_invalid` | error | Veuillez entrer une URL valide | 1 code |
| `phone_invalid` | `forms.error.phone_invalid` | error | Numéro de téléphone invalide (chiffres, +, espa… | 1 code |
| `hash_error` | `forms.error.hash` | error | Erreur de hachage : {} | 4 code |
| `password_empty` | `forms.error.password_empty` | error | Le mot de passe ne peut pas être vide | 3 code |
| `filter_invalid_value` | `forms.error.filter_invalid_value` | error | Le filtre static/media a reçu une valeur invali… | — **à supprimer ?** |
| `number_invalid` | `forms.error.number_invalid` | error | Nombre invalide | 4 code |
| `number_required` | `forms.error.number_required` | error | Requis | 1 code |
| `integer_required` | `forms.error.integer_required` | error | Doit être un entier | 1 code |
| `decimal_required` | `forms.error.decimal_required` | error | Doit être au format décimal | — **à supprimer ?** |
| `min_value` | `forms.error.min_value` | error | Min {} requis | 3 code |
| `max_value` | `forms.error.max_value` | error | Max {} dépassé | 3 code |
| `precision_max` | `forms.error.precision_max` | error | Précision max {} chiffres après virgule | 1 code |
| `integer_digits_max` | `forms.error.integer_digits_max` | error | {} chiffres maximum avant la virgule | 1 code |
| `value_unconvertible` | `forms.error.value_unconvertible` | error | Cette valeur ne correspond pas à ce champ | 6 code |
| `must_be_checked` | `forms.error.must_be_checked` | error | Cette case doit être cochée | 1 code |
| `precision_min` | `forms.error.precision_min` | error | Précision min {} chiffres après virgule | 1 code |
| `color_invalid` | `forms.error.color_invalid` | error | Couleur invalide (format: #RRGGBB ou #RGB) | 1 code |
| `color_no_hash` | `forms.error.color_no_hash` | error | La couleur doit commencer par # | 1 code |
| `color_bad_hex` | `forms.error.color_bad_hex` | error | Contient des caractères non hexadécimaux | 1 code |
| `slug_invalid` | `forms.error.slug_invalid` | error | Slug invalide | 1 code |
| `slug_no_dash` | `forms.error.slug_no_dash` | error | Le slug ne doit pas commencer ou finir par un t… | 1 code |
| `uuid_invalid` | `forms.error.uuid_invalid` | error | Format UUID invalide (attendu: xxxxxxxx-xxxx-xx… | 1 code |
| `json_invalid` | `forms.error.json_invalid` | error | JSON invalide : {} | 1 code |
| `ipv4_only` | `forms.error.ipv4_only` | error | Seules les adresses IPv4 sont acceptées | 1 code |
| `ipv6_only` | `forms.error.ipv6_only` | error | Seules les adresses IPv6 sont acceptées | 1 code |
| `ip_invalid` | `forms.error.ip_invalid` | error | Adresse IP invalide | 1 code |
| `date_invalid` | `forms.error.date_invalid` | error | Format de date invalide | 2 code |
| `date_too_old` | `forms.error.date_too_old` | error | Trop ancien (min : {}) | 1 code |
| `date_too_far` | `forms.error.date_too_far` | error | Trop loin (max : {}) | 1 code |
| `time_invalid` | `forms.error.time_invalid` | error | Format de temps invalide | 1 code |
| `file_required` | `forms.error.file_required` | error | Veuillez sélectionner au moins un fichier | 2 code |
| `data_invalid` | `forms.error.data_invalid` | error | Valeur invalide pour « {} » | 1 code |
| `data_required` | `forms.error.data_required` | error | « {} » est obligatoire | 1 code |
| `data_password` | `forms.error.data_password` | error | Le mot de passe de « {} » n'a pas pu être haché… | 1 code |
| `file_extension_blocked` | `forms.error.file_extension_blocked` | error | Type de fichier non autorisé. Extensions accept… | 1 code |
| `finalize_error` | `forms.error.finalize` | error | Erreur lors de la finalisation du champ '{}': {} | 3 code |
| `template_missing` | `forms.error.template_missing` | error | Template manquant: {} | 2 code |
| `render_js_error` | `forms.error.render_js` | error | Erreur rendu JS: {} | 1 code |
| `render_errors_error` | `forms.error.render_errors` | error | Erreur rendu des erreurs du formulaire: {} | 1 code |
| `tera_not_configured` | `forms.error.tera_not_configured` | error | Tera non configuré | 1 code |
| `config_not_found` | `forms.error.config_not_found` | error | Configuration introuvable dans les extensions | 1 code |
| `hint_slug` | `forms.hint.slug` | hint | Format : lettres-chiffres-tirets | 1 code |
| `hint_uuid` | `forms.hint.uuid` | hint | Format : xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx | 1 code |
| `hint_json` | `forms.hint.json` | hint | Format JSON valide requis | 1 code |
| `hint_ip_v4` | `forms.hint.ip_v4` | hint | Format IPv4 : 192.168.1.1 | 1 code |
| `hint_ip_v6` | `forms.hint.ip_v6` | hint | Format IPv6 : 2001:0db8:85a3::8a2e:0370:7334 | 1 code |
| `hint_ip` | `forms.hint.ip` | hint | Format IPv4 ou IPv6 | 1 code |
| `validation_overflow` | `forms.error.validation_overflow` | error | Stack overflow détecté : récursion infinie dans… | 2 code |
| `choice_invalid` | `forms.error.choice_invalid` | error | Choix invalide | 2 code |
| `choice_invalid_value` | `forms.error.choice_invalid_value` | error | Choix invalide : {} | 2 code |
| `select_required` | `forms.error.select_required` | error | Veuillez sélectionner une option | 2 code |
| `checkbox_required` | `forms.error.checkbox_required` | error | Veuillez sélectionner au moins une option | 1 code |
| `multipart_error` | `forms.error.multipart` | error | Erreur multipart | 1 code |
| `body_error` | `forms.error.body` | error | Erreur lecture du body | 1 code |
| `aegis_query_error` | `forms.error.aegis_query` | error | Échec parsing query string : {} | 1 code |
| `aegis_urlencoded_error` | `forms.error.aegis_urlencoded` | error | Échec parsing urlencoded : {} | — **à supprimer ?** |
| `aegis_json_error` | `forms.error.aegis_json` | error | Échec parsing JSON : {} | 1 code |
| `time_too_old` | `forms.error.time_too_old` | error | Trop tôt (min : {}) | 1 code |
| `time_too_far` | `forms.error.time_too_far` | error | Trop tard (max : {}) | 1 code |
| `datetime_too_old` | `forms.error.datetime_too_old` | error | Trop tôt (min : {}) | 1 code |
| `datetime_too_far` | `forms.error.datetime_too_far` | error | Trop tard (max : {}) | 1 code |
| `duration_invalid` | `forms.error.duration_invalid` | error | Durée invalide (nombre de secondes attendu) | 1 code |
| `duration_too_short` | `forms.error.duration_too_short` | error | Durée minimale : {} secondes | 1 code |
| `duration_too_long` | `forms.error.duration_too_long` | error | Durée maximale : {} secondes | 1 code |
| `file_max_count` | `forms.error.file_max_count` | error | Maximum {} fichiers autorisés | 1 code |
| `file_too_large` | `forms.error.file_too_large` | error | Fichier trop volumineux, taille maximale : {} Mo | 2 code |
| `file_unreadable` | `forms.error.file_unreadable` | error | Le fichier envoyé n'a pas pu être lu, merci de … | 3 code |
| `file_invalid_image` | `forms.error.file_invalid_image` | error | Ce fichier n'est pas une image valide | 2 code |
| `file_invalid_audio` | `forms.error.file_invalid_audio` | error | Ce fichier n'est pas un fichier audio valide | 2 code |
| `file_invalid_video` | `forms.error.file_invalid_video` | error | Ce fichier n'est pas une vidéo valide | 2 code |
| `image_too_wide` | `forms.error.image_too_wide` | error | Image trop large, largeur maximale : {} px | 1 code |
| `image_too_tall` | `forms.error.image_too_tall` | error | Image trop haute, hauteur maximale : {} px | 1 code |
| `boolean_required` | `forms.error.boolean_required` | error | Vous devez accepter ce champ | — **à supprimer ?** |
| `slug_unicode_invalid` | `forms.error.slug_unicode_invalid` | error | Seules les lettres, chiffres, tirets et undersc… | 1 code |
| `unique_field_taken` | `forms.error.unique_field_taken` | error | Ce {} est déjà utilisé. | 2 code |
| `unique_value_taken` | `forms.error.unique_value_taken` | error | La valeur du champ '{}' est déjà utilisée. | 1 code |
| `unique_constraint_violated` | `forms.error.unique_constraint_violated` | error | Une contrainte d'unicité a été violée. | 2 code |
| `db_error` | `forms.error.db` | error | L'enregistrement a été refusé par la base. | 4 code |
| `db_foreign_key` | `forms.error.db_foreign_key` | error | Cette valeur ne correspond à aucun enregistreme… | 2 code |
| `db_not_null` | `forms.error.db_not_null` | error | Une valeur obligatoire est manquante. | 2 code |
| `db_check` | `forms.error.db_check` | error | Cette valeur n'est pas acceptée. | 3 code |
| `delete_linked` | `forms.error.delete_linked` | error | Impossible de supprimer : d'autres enregistreme… | 3 code |
| `text_field_too_large` | `forms.error.text_field_too_large` | error | Champ texte trop volumineux (max 1 Mo) | 1 code |
| `upload_too_large` | `forms.error.upload_too_large` | error | Fichier trop volumineux (max {}Mo) | 1 code |
| `body_too_large` | `forms.error.body_too_large` | error | Requête trop volumineuse | 3 code |
| `upload_failed` | `forms.error.upload_failed` | error | Le fichier n'a pas pu être enregistré. Réessaye… | 2 code |
| `max_size_above_model` | `forms.error.max_size_above_model` | error | La taille maximale demandée ({} Mo) dépasse la … | 2 code |

### `html` (16 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `400_title` | `http.label.400` | label | 400 | exéc. |
| `400_text` | `http.error.400` | error | Requête invalide : les paramètres de l'adresse … | exéc. |
| `413_title` | `http.label.413` | label | 413 | exéc. |
| `413_text` | `http.error.413` | error | Envoi trop volumineux : réduisez sa taille et r… | exéc. |
| `404_title` | `http.label.404` | label | 404 | exéc. |
| `404_text` | `http.error.404` | error | Page non trouvée | exéc. |
| `500_title` | `http.label.500` | label | 500 | exéc. |
| `500_text` | `http.error.500` | error | Une erreur est survenue sur le serveur | exéc. |
| `429_title` | `http.label.429` | label | 429 | exéc. |
| `429_text` | `http.error.429` | error | Trop de requêtes. Veuillez patienter un moment … | 1 code, exéc. |
| `503_title` | `http.label.503` | label | 503 | exéc. |
| `503_text` | `http.error.503` | error | Service temporairement indisponible en raison d… | exéc. |
| `back_home` | `http.label.back_home` | label | Retour à l'accueil | 2 code, exéc. |
| `critical_error_title` | `http.label.critical` | label | Erreur critique | 2 code, exéc. |
| `critical_error_text` | `http.error.critical_text` | error | Le système de gestion d'erreurs a lui-même renc… | 1 code, exéc. |
| `critical_error_contact` | `http.error.critical_contact` | error | Cette situation ne devrait jamais se produire. … | 1 code, exéc. |

### `log` (53 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `csrf.refused` | `csrf.log.refused` | log | CSRF refusé | 1 code |
| `csrf.reason_header_invalid` | `csrf.log.reason_header_invalid` | log | jeton invalide dans l'en-tête X-CSRF-Token | 1 code |
| `csrf.reason_not_a_form` | `csrf.log.reason_not_a_form` | log | aucun jeton, et le corps n'est pas un formulaire | 1 code |
| `csrf.reason_cross_site` | `csrf.log.reason_cross_site` | log | formulaire envoyé depuis un autre site | 1 code |
| `csrf.reason_form_mismatch` | `csrf.log.reason_form_mismatch` | log | formulaire : le jeton envoyé n'est pas celui de… | 1 code |
| `csrf.reason_form_missing` | `csrf.log.reason_form_missing` | log | formulaire : aucun jeton dans le corps ni dans … | 1 code |
| `csrf.reason_admin_login_missing` | `csrf.log.reason_admin_login_missing` | log | connexion à l'admin : aucun jeton dans le formu… | 1 code |
| `csrf.reason_admin_login_mismatch` | `csrf.log.reason_admin_login_mismatch` | log | connexion à l'admin : le jeton envoyé n'est pas… | 1 code |
| `csrf.reason_admin_logout` | `csrf.log.reason_admin_logout` | log | déconnexion de l'admin : aucun jeton valide | 1 code |
| `csrf.reason_upload` | `csrf.log.reason_upload` | log | envoi de fichier : un fichier est arrivé avant … | 1 code |
| `csrf.token_dropped` | `csrf.log.token_dropped` | log | jeton CSRF abandonné (connexion ou déconnexion)… | 1 code |
| `csrf.token_new` | `csrf.log.token_new` | log | nouveau jeton CSRF pour cette session | 1 code |
| `csrf.read_token` | `csrf.log.read_token` | log | lecture du jeton CSRF dans la session | 1 code |
| `csrf.keep_token` | `csrf.log.keep_token` | log | conservation du jeton CSRF dans la session | 1 code |
| `csrf.read_account` | `csrf.log.read_account` | log | lecture du compte connecté pour le jeton CSRF | 1 code |
| `csrf.write_token` | `csrf.log.write_token` | log | écriture d'un nouveau jeton CSRF dans la session | 1 code |
| `csrf.set_value` | `csrf.log.set_value` | log | set_value [jeton CSRF reçu : son contrôle est s… | 1 code |
| `session.read_account` | `session.log.read_account` | log | lecture du compte connecté dans la session | 2 code |
| `session.read_version` | `session.log.read_version` | log | lecture de la version de la session | 1 code |
| `session.read_flash` | `session.log.read_flash` | log | lecture des messages flash dans la session | 2 code |
| `session.read_dashboard_template` | `session.log.read_dashboard_template` | log | lecture du gabarit de tableau de bord choisi da… | 2 code |
| `session.decode_user_memory` | `session.log.decode_user_memory` | log | décodage du user_id de la session (chargement d… | 1 code |
| `session.decode_user_db` | `session.log.decode_user_db` | log | décodage du user_id de la session (chargement d… | 1 code |
| `anti_bot.read_name` | `forms.log.anti_bot_read_name` | log | lecture du nom du champ piège dans la session | 1 code |
| `anti_bot.filled` | `forms.log.anti_bot_filled` | log | champ piège rempli : formulaire refusé | 1 code |
| `https.redirected` | `server.log.https_redirected` | log | requête reçue en HTTP : redirigée vers HTTPS | 1 code |
| `https.port80_stopped` | `server.log.https_port80_stopped` | log | le serveur HTTP du port 80 (défi ACME et redire… | 1 code |
| `auth.hash_failed` | `auth.log.hash_failed` | log | échec du hachage du mot de passe à la réinitial… | 1 code |
| `auth.account_not_read` | `auth.log.account_not_read` | log | compte non lu : session gardée, requête répondu… | 1 code |
| `auth.admin_account_not_read` | `auth.log.admin_account_not_read` | log | compte de l'admin non lu : session gardée, requ… | 1 code |
| `forms.validation_failed` | `forms.log.validation_failed` | log | échec de la validation du formulaire | 1 code |
| `forms.remove_oversized` | `forms.log.remove_oversized` | log | suppression d'un fichier en transit trop volumi… | 1 code |
| `forms.remove_replaced` | `forms.log.remove_replaced` | log | suppression du fichier qu'un nouvel envoi rempl… | 1 code |
| `mailer.init_twice` | `mailer.log.init_twice` | log | mailer_init() appelé plusieurs fois : la config… | 1 code |
| `db.read_reset_token` | `database.log.read_reset_token` | log | lecture d'un jeton de réinitialisation de mot d… | 1 code |
| `admin.write` | `admin.log.write` | log | écriture dans l'admin | 1 code |
| `admin.outcome_saved` | `admin.log.outcome_saved` | log | enregistré | 1 code |
| `admin.outcome_save_refused` | `admin.log.outcome_save_refused` | log | enregistrement refusé | 1 code |
| `admin.outcome_deleted` | `admin.log.outcome_deleted` | log | supprimé | 1 code |
| `admin.outcome_delete_refused` | `admin.log.outcome_delete_refused` | log | suppression refusée | 1 code |
| `admin.bulk_done` | `admin.log.bulk_done` | log | action groupée effectuée | 1 code |
| `admin.bulk_refused` | `admin.log.bulk_refused` | log | action groupée refusée | 1 code |
| `admin.list` | `admin.log.list` | log | liste de l'admin | 1 code |
| `boot.admin` | `admin.log.boot_admin` | log | admin | 1 code |
| `boot.model` | `admin.log.boot_model` | log | modèle de l'admin | 1 code |
| `boot.route` | `admin.log.boot_route` | log | route déclarée de l'admin | 1 code |
| `boot.no_guard` | `admin.log.boot_no_guard` | log | la connexion de l'admin n'a ni LoginGuard ni li… | 1 code |
| `boot.options_without_effect` | `admin.log.boot_options_without_effect` | log | options de liste de l'admin acceptées, mais aff… | 1 code |
| `boot.stylesheets_missing` | `admin.log.boot_stylesheets_missing` | log | feuilles de style de Runique introuvables pour … | 1 code |
| `boot.debug_in_prod` | `build.log.debug_in_prod` | log | DEBUG=true en production : tous les visiteurs v… | 1 code |
| `boot.prune_failed` | `build.log.prune_failed` | log | échec de l'élagage des droits orphelins : des d… | 1 code |
| `server.ctrl_c_not_installed` | `server.log.ctrl_c_not_installed` | log | gestionnaire de Ctrl+C non installé | 1 code |
| `server.sigterm_not_installed` | `server.log.sigterm_not_installed` | log | gestionnaire de SIGTERM non installé | 1 code |

### `makemigrations` (23 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `project_pk_cargo` | `makemigrations.error.project_pk_cargo` | error | Impossible de lire les features du projet avec … | 3 code |
| `project_pk_no_runique` | `makemigrations.error.project_pk_no_runique` | error | `runique` n'est pas une dépendance du projet la… | 2 code |
| `destructive_detected` | `makemigrations.error.destructive_detected` | error | Changements destructifs détectés : | 1 code |
| `destructive_require_force` | `makemigrations.error.destructive_require_force` | error | Utilisez --force pour appliquer ces changements… | 1 code |
| `long_identifier_detected` | `makemigrations.error.long_identifier_detected` | error | Identifiants SQL générés trop longs (limite MyS… | 1 code |
| `long_identifier_rename_required` | `makemigrations.error.long_identifier_rename_required` | error | Raccourcissez les noms de table/colonne concern… | 1 code |
| `files_ready` | `makemigrations.info.files_ready` | info | {} fichier(s) prêt(s) à migrer. | 1 code |
| `snapshots_upgraded` | `makemigrations.info.snapshots_upgraded` | info | {} snapshot(s) mis à jour pour enregistrer les … | 1 code |
| `type_change_not_migrated` | `makemigrations.error.type_change_not_migrated` | error | ATTENTION : --force a laissé passer ces changem… | 1 code |
| `type_change_write_by_hand` | `makemigrations.error.type_change_write_by_hand` | error | Écrivez à la main le up() et le down() de cette… | 1 code |
| `no_changes` | `makemigrations.info.no_changes` | info | Aucun changement détecté : vos modèles correspo… | 1 code |
| `apply_hint` | `makemigrations.hint.apply` | hint | Appliquez les migrations en attente avec : runi… | 3 code |
| `database_unreachable` | `makemigrations.error.database_unreachable` | error | Connexion à la base impossible : {} | 2 code |
| `applied_unknown` | `makemigrations.error.applied_unknown` | error | Ces migrations sont inscrites dans la base mais… | 1 code |
| `snapshot_unreadable` | `makemigrations.error.snapshot_unreadable` | error | Le snapshot {} est illisible ou n'est pas un sn… | 2 code |
| `snapshot_misnamed` | `makemigrations.error.snapshot_misnamed` | error | Le snapshot {} décrit la table `{}` : son nom d… | 2 code |
| `duplicate_table` | `makemigrations.error.duplicate_table` | error | Deux tables portent le nom `{}`. La table d'un … | 1 code |
| `waiting_removed` | `makemigrations.info.waiting_removed` | info | {} migration(s) en attente retirée(s) : vos mod… | 1 code |
| `waiting_unchanged` | `makemigrations.info.waiting_unchanged` | info | {} migration(s) en attente, inchangée(s) : rien… | 1 code |
| `parallel_changes` | `makemigrations.error.parallel_changes` | error | Deux migrations partent du même point pour la m… | 2 code |
| `paused_tables` | `makemigrations.info.paused_tables` | info | Tables en pause (`managed: false`), ignorées pa… | 1 code |
| `rename_with_new_key` | `makemigrations.error.rename_with_new_key` | error | La table `{}` est renommée dans ce lancement et… | 1 code |
| `renamed_twice` | `makemigrations.error.renamed_twice` | error | Deux tables se disent renommées depuis `{}` : `… | 1 code |

### `middleware` (2 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `bad_request` | `middleware.error.bad_request` | error | Mauvaise requête | 1 code |
| `slot_collision` | `middleware.error.slot_collision` | error | Collision de slot middleware Runique : « {} » e… | 1 code |

### `migrate` (12 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `complete_up` | `migrate.info.complete_up` | info | Migration terminé. | 1 code |
| `crate_missing` | `migrate.error.crate_missing` | error | Crate de migrations introuvable : '{}' ne conti… | 1 code |
| `sqlite_key_change` | `migrate.error.sqlite_key_change` | error | {} : cette migration ajoute ou retire une clé é… | 2 code |
| `engine_unsupported` | `migrate.error.engine_unsupported` | error | Moteur de base de données non pris en charge po… | 1 code |
| `running` | `migrate.info.running` | info | Exécution de 'migrate {}' avec le crate de migr… | 1 code |
| `complete_down` | `migrate.info.complete_down` | info | Retour arrière terminé. | 1 code |
| `cargo_missing` | `migrate.error.cargo_missing` | error | Impossible de lancer cargo. Rust est-il install… | 1 code |
| `failed` | `migrate.error.failed` | error | Le crate de migrations a échoué sur 'migrate {}… | 1 code |
| `crate_created` | `migrate.info.crate_created` | info | Crate de migrations créé dans '{}'. | 1 code |
| `interrupted` | `migrate.error.interrupted` | error | Un makemigrations a été interrompu dans '{}'. R… | 3 code |
| `renamed_from_kept` | `migrate.error.renamed_from_kept` | error | {} : les `renamed_from` appliqués n'ont pas pu … | 1 code |
| `engine_defaulted` | `migrate.error.engine_defaulted` | error | Ni DATABASE_URL ni DB_ENGINE ne désignent un mo… | 1 code |

### `reset` (18 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `forgot_title` | `reset.label.forgot_title` | label | Mot de passe oublié | 1 code |
| `reset_title` | `reset.label.reset_title` | label | Nouveau mot de passe | 6 code |
| `success_title` | `reset.label.success_title` | label | Mot de passe réinitialisé | 1 code |
| `email_label` | `reset.label.email_label` | label | Adresse email | 2 code |
| `new_password_label` | `reset.label.new_password_label` | label | Nouveau mot de passe | 1 code |
| `confirm_label` | `reset.label.confirm_label` | label | Confirmer le mot de passe | 1 code |
| `email_mismatch` | `reset.error.email_mismatch` | error | L'adresse email ne correspond pas au lien. | 1 code |
| `invalid_link` | `reset.error.invalid_link` | error | Lien invalide. | — **à supprimer ?** |
| `invalid_or_expired` | `reset.error.invalid_or_expired` | error | Lien de réinitialisation invalide ou expiré. | 4 code |
| `password_min_length` | `reset.error.password_min_length` | error | Le mot de passe doit contenir au moins {} carac… | 1 code |
| `password_weak` | `reset.error.password_weak` | error | Le mot de passe doit contenir au moins une maju… | 1 code |
| `password_mismatch` | `reset.error.password_mismatch` | error | Les mots de passe ne correspondent pas. | 1 code |
| `check_inbox` | `reset.info.check_inbox` | info | Si cette adresse est associée à un compte, vous… | 1 code |
| `blocked_subject` | `reset.label.blocked_subject` | label | Votre compte est bloqué | 1 code |
| `blocked_body` | `reset.label.blocked_body` | label | <p>Bonjour {},</p><p>Une demande de réinitialis… | 1 code |
| `email_subject` | `reset.label.email_subject` | label | Réinitialisation de votre mot de passe | 1 code |
| `email_body` | `reset.label.email_body` | label | <p>Bonjour {},</p><p>Cliquez sur le lien suivan… | 1 code |
| `unavailable` | `reset.error.unavailable` | error | La réinitialisation par email n'est pas disponi… | 1 code |

### `runique_test` (22 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `connect_failed` | `runique_test.error.connect_failed` | error | connexion impossible : {} | 1 code |
| `begin_failed` | `runique_test.error.begin_failed` | error | impossible d'ouvrir la transaction de test : {} | 1 code |
| `env_read_failed` | `runique_test.error.env_read_failed` | error | impossible de lire {} : {} | 1 code |
| `env_parse_failed` | `runique_test.error.env_parse_failed` | error | impossible d'analyser {} : {} | 1 code |
| `env_line_malformed` | `runique_test.error.env_line_malformed` | error | impossible d'analyser {} : une ligne est mal fo… | 1 code |
| `no_panic_message` | `runique_test.error.no_panic_message` | error | aucun message | 2 code |
| `query_failed` | `runique_test.error.query_failed` | error | échec | 2 code |
| `panicked` | `runique_test.error.panicked` | error | panique : {} | 1 code |
| `rollback_failed` | `runique_test.error.rollback_failed` | error | rollback : {} | 1 code |
| `test_failed` | `runique_test.error.test_failed` | error | {} a échoué, voir la sortie ci-dessus | 1 code |
| `clone_outlived` | `runique_test.error.clone_outlived` | error | un clone de la connexion de test a survécu au h… | 1 code |
| `not_a_transaction` | `runique_test.error.not_a_transaction` | error | le test ne s'est pas exécuté dans une transacti… | 1 code |
| `nested` | `runique_test.error.nested` | error | runique_test ne peut pas tourner à l'intérieur … | 1 code |
| `timed_out` | `runique_test.error.timed_out` | error | le handler n'a pas fini en {} s : un interbloca… | 1 code |
| `bad_timeout` | `runique_test.error.bad_timeout` | error | {} doit être un nombre entier de secondes supér… | 1 code |
| `transaction_ended` | `runique_test.error.transaction_ended` | error | la transaction du test a été terminée depuis le… | 2 code |
| `swallowed_failures` | `runique_test.error.swallowed_failures` | error | requêtes en échec : {}, et pourtant le handler … | 2 code |
| `rollback_timed_out` | `runique_test.error.rollback_timed_out` | error | le rollback n'a pas fini en {} s | 1 code |
| `expected` | `runique_test.label.expected` | label | attendu | 1 code |
| `not_refused` | `runique_test.error.not_refused` | error | la base a accepté ce que le test s'attendait à … | 1 code |
| `no_db_config` | `runique_test.error.no_db_config` | error | le fichier env ne définit ni DATABASE_URL ni DB… | 1 code |
| `in_memory` | `runique_test.label.in_memory` | label | en mémoire : chaque test part d'une base vide | 3 code |

### `server` (9 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `operational` | `server.info.operational` | info | Runique est opérationnel | 2 code |
| `launched` | `server.info.launched` | info | Serveur lancé sur {} | 2 code |
| `connected_db` | `server.info.connected_db` | info | Connecté à la base de données {} -> {} | 1 code |
| `connected_engine` | `server.info.connected_engine` | info | Connecté à la base de données {} | 1 code |
| `stop_hint` | `server.hint.stop` | hint | ctrl + c pour arrêter | 1 code |
| `shutting_down` | `server.info.shutting_down` | info | Arrêt du serveur Runique... | 1 code |
| `acme_not_compiled` | `server.error.acme_not_compiled` | error | ACME_ENABLED=true mais la feature `acme` n'est … | 1 code |
| `acme_add_feature` | `server.info.acme_add_feature` | info | Ajoute la feature `acme` à ton Cargo.toml. | 1 code |
| `test_utils_feature_enabled` | `server.info.test_utils_feature_enabled` | info | Feature `test-utils` compilée dans ce build — a… | 2 code |

### `test_runner` (16 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `no_cargo_toml` | `test_runner.error.no_cargo_toml` | error | Pas de Cargo.toml ici : lancez `runique test` d… | 1 code |
| `test_utils_missing` | `test_runner.error.test_utils_missing` | error | La feature `test-utils` n'est pas activée pour … | 1 code |
| `test_utils_shipped` | `test_runner.error.test_utils_shipped` | error | Cargo.toml active `test-utils` en dehors de [de… | 1 code |
| `no_mod_file` | `test_runner.error.no_mod_file` | error | Pas de src/runique_test/mod.rs : créez-le, puis… | 1 code |
| `module_not_declared` | `test_runner.error.module_not_declared` | error | src/main.rs ne déclare pas le module de test. A… | 1 code |
| `module_not_gated` | `test_runner.error.module_not_gated` | error | Le module runique_test n'est pas derrière `cfg(… | 1 code |
| `file_not_declared_warning` | `test_runner.error.file_not_declared_warning` | error | attention : src/runique_test/{}.rs n'est pas dé… | 1 code |
| `no_test_file` | `test_runner.error.no_test_file` | error | Pas de src/runique_test/{}.rs. Fichiers de test… | 1 code |
| `file_not_declared` | `test_runner.error.file_not_declared` | error | src/runique_test/{}.rs n'est pas déclaré dans m… | 1 code |
| `cargo_spawn_failed` | `test_runner.error.cargo_spawn_failed` | error | Impossible de lancer cargo. | 2 code |
| `output_failed` | `test_runner.error.output_failed` | error | Impossible d'afficher la sortie de cargo. | 1 code |
| `cargo_wait_failed` | `test_runner.error.cargo_wait_failed` | error | cargo test ne s'est pas terminé correctement. | 1 code |
| `tests_failed` | `test_runner.error.tests_failed` | error | cargo test a échoué, voir la sortie ci-dessus. | 1 code |
| `build_failed` | `test_runner.error.build_failed` | error | cargo n'a pas pu compiler les tests, voir les e… | 1 code |
| `no_such_test` | `test_runner.error.no_such_test` | error | Aucun test "{}" dans {}.rs. Tests disponibles :… | 1 code |
| `dir_read_failed` | `test_runner.error.dir_read_failed` | error | Impossible de lire {} | 1 code |

### `title` (1 clés)

| Avant | Après | Nature | Texte (fr) | Usages |
|---|---|---|---|---|
| `template` | `error.label.template_in` | label | Erreur de template dans '{}' | 1 code |

