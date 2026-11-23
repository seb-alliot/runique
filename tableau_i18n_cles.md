# Tableau des clés de traduction

Produit le 2026-11-23 par lecture des fichiers `runique/runique_i18n/src/locales/` et du code. Document de travail pour décider des renommages : **rien n'est modifié**.

Méthode : une clé est comptée « utilisée » quand elle apparaît en toutes lettres entre guillemets dans un fichier `.rs`, ou quand sa variable de template (`admin.login.title` → `admin_login_title`) apparaît dans un template. Une clé fabriquée à l'exécution (`format!("log.{}…")`) n'est pas vue par cette lecture : elle est signalée à part.

- 22 thèmes, 657 clés par langue.
- 570 clés trouvées dans le code ou les templates ; 69 probablement fabriquées à l'exécution ; **18 sans aucune référence trouvée**.
- 30 groupes de clés au texte strictement identique (français et anglais) ; 20 groupes au texte presque identique.
- **2 clés écrites dans le code et absentes des fichiers** : le message affiché est la clé elle-même.

## 1. Vue par thème

| Thème | Clés | Sous-sections | Utilisées (code) | Utilisées (templates) | Sans référence | Où (crates) |
|---|---:|---:|---:|---:|---:|---|
| `TemplateMessage` | 30 | 0 | 0 | 0 | 0 |  |
| `admin` | 253 | 21 | 224 | 128 | 0 | runique (193), runique-cli (31) |
| `atomic` | 16 | 0 | 16 | 0 | 0 | runique_atomic (16) |
| `auth` | 2 | 0 | 2 | 0 | 0 | runique (2) |
| `build` | 28 | 1 | 28 | 0 | 0 | runique (28) |
| `cli` | 14 | 0 | 14 | 0 | 0 | runique-cli (14) |
| `csrf` | 5 | 0 | 3 | 0 | 2 | runique (3) |
| `database` | 12 | 0 | 12 | 0 | 0 | runique (12) |
| `debug` | 2 | 0 | 0 | 2 | 0 |  |
| `error` | 22 | 1 | 17 | 0 | 5 | runique (17) |
| `flash` | 6 | 0 | 0 | 0 | 6 |  |
| `forms` | 95 | 0 | 91 | 0 | 4 | runique (91), runique_i18n (1) |
| `html` | 16 | 0 | 5 | 0 | 0 | runique (5) |
| `log` | 53 | 11 | 53 | 0 | 0 | runique (53) |
| `makemigrations` | 23 | 0 | 23 | 0 | 0 | runique-cli (23) |
| `middleware` | 2 | 0 | 2 | 0 | 0 | runique (2) |
| `migrate` | 12 | 0 | 12 | 0 | 0 | runique-cli (10), runique (2) |
| `reset` | 18 | 0 | 17 | 0 | 1 | runique (17) |
| `runique_test` | 22 | 0 | 22 | 0 | 0 | runique (22) |
| `server` | 9 | 0 | 9 | 0 | 0 | runique (9) |
| `test_runner` | 16 | 0 | 16 | 0 | 0 | runique-cli (16) |
| `title` | 1 | 0 | 1 | 0 | 0 | runique (1) |

## 2. Textes strictement identiques sous plusieurs clés

Même texte en français **et** en anglais. Candidats à une seule clé partagée, ou doublons voulus (même mot dans deux écrans).

| Texte (fr) | Clés |
|---|---|
| Créer | `admin.create.breadcrumb`<br>`admin.create.btn_submit`<br>`admin.create.title`<br>`admin.dashboard.btn_create`<br>`admin.list.btn_create`<br>`admin.rights.col_create`<br>`admin.rights.right_create` |
| Supprimer | `admin.delete.breadcrumb`<br>`admin.delete.title`<br>`admin.detail.btn_delete`<br>`admin.list.btn_delete`<br>`admin.list.bulk_delete`<br>`admin.rights.right_delete` |
| Annuler | `admin.create.btn_cancel`<br>`admin.edit.btn_cancel`<br>`admin.rights.btn_cancel`<br>`admin.transfer.btn_cancel` |
| Modifier | `admin.detail.btn_edit`<br>`admin.edit.breadcrumb`<br>`admin.edit.title`<br>`admin.list.btn_edit` |
| Groupes | `admin.groups`<br>`admin.history.badge_groups`<br>`admin.list.groups_label` |
| Détail | `admin.detail.breadcrumb`<br>`admin.detail.title`<br>`admin.list.btn_detail` |
| Droits | `admin.detail.btn_rights`<br>`admin.list.btn_rights`<br>`admin.rights.title` |
| Ressource | `admin.dashboard.th_resource`<br>`admin.history.th_resource`<br>`admin.resource` |
| Ressources | `admin.dashboard.card_resources`<br>`admin.dashboard.kpi_resources`<br>`admin.resources` |
| Nom | `TemplateMessage.debug_header_name`<br>`admin.name` |
| Action non autorisée | `admin.access.action_not_allowed`<br>`admin.access.superuser_only` |
| Admin | `admin.base.breadcrumb`<br>`admin.rank.admin` |
| Informations | `admin.create.card_info`<br>`admin.edit.card_info` |
| Aucun champ disponible pour le moment. | `admin.create.no_fields`<br>`admin.edit.no_fields` |
| Actions | `admin.dashboard.th_actions`<br>`admin.list.th_actions` |
| Réinitialiser le mot de passe | `admin.detail.btn_reset_password`<br>`admin.reset_password.btn` |
| Réinitialiser | `admin.detail.btn_reset_password_short`<br>`admin.history.btn_reset` |
| Enregistrer | `admin.edit.btn_submit`<br>`admin.rights.btn_save` |
| Groupe | `admin.group`<br>`admin.rights.group_label` |
| supprimé | `admin.history.badge_delete`<br>`log.admin.outcome_deleted` |
| Colonne | `admin.history.th_column`<br>`admin.rights.th_column` |
| Modifier | `admin.rights.col_update`<br>`admin.rights.right_update` |
| Superutilisateur | `admin.rank.superuser`<br>`admin.superuser` |
| Action | `admin.history.th_action`<br>`admin.superuser_wizard.review_action` |
| Format d'adresse email invalide | `admin.user.invalid_email`<br>`forms.email_invalid` |
| Nom d'utilisateur | `admin.login.label_username`<br>`admin.username` |
| Token CSRF invalide | `csrf.forbidden`<br>`csrf.invalid` |
| Trop tard (max : {}) | `forms.datetime_too_far`<br>`forms.time_too_far` |
| Trop tôt (min : {}) | `forms.datetime_too_old`<br>`forms.time_too_old` |
| Nouveau mot de passe | `reset.new_password_label`<br>`reset.reset_title` |

## 3. Textes presque identiques

Même texte une fois retirés la casse, les accents et la ponctuation.

| Clé | Texte (fr) |
|---|---|
| `TemplateMessage.debug_stack_trace_tip` | L'erreur la plus haute (niveau 0) est généralement la cause racine. Les erreurs… |
| `debug.stack_trace_tip` | L'erreur la plus haute (niveau 0) est généralement la cause racine. Les erreurs… |
| | |
| `TemplateMessage.debug_stack_trace_title` | Trace d'erreur détaillée |
| `debug.stack_trace_title` | Trace d'erreur détaillée |
| | |
| `admin.base.breadcrumb` | Admin |
| `admin.rank.admin` | Admin |
| `log.boot.admin` | admin |
| | |
| `admin.create.btn_cancel` | Annuler |
| `admin.delete.btn_cancel` | ← Annuler |
| `admin.edit.btn_cancel` | Annuler |
| `admin.rights.btn_cancel` | Annuler |
| `admin.transfer.btn_cancel` | Annuler |
| | |
| `admin.dashboard.btn_list` | Liste |
| `admin.detail.btn_list` | ← Liste |
| | |
| `admin.dashboard.th_count` | Entrées |
| `admin.list.entries_count_many` | entrées |
| | |
| `admin.delete.not_found` | Ressource introuvable. |
| `error.not_found` | Ressource introuvable |
| | |
| `admin.history.action_create` | Créé |
| `admin.history.badge_create` | créé |
| | |
| `admin.history.action_delete` | Supprimé |
| `admin.history.badge_delete` | supprimé |
| `log.admin.outcome_deleted` | supprimé |
| | |
| `admin.history.action_update` | Modifié |
| `admin.history.badge_update` | modifié |
| | |
| `admin.detail.breadcrumb` | Détail |
| `admin.detail.title` | Détail |
| `admin.history.btn_entry` | Détail |
| `admin.list.btn_detail` | Détail |
| | |
| `admin.detail.btn_edit` | Modifier |
| `admin.edit.breadcrumb` | Modifier |
| `admin.edit.title` | Modifier |
| `admin.list.btn_edit` | Modifier |
| `admin.rights.col_update` | Modifier |
| `admin.rights.right_update` | Modifier |
| | |
| `admin.detail.entry_label` | Entrée |
| `admin.list.entries_count_one` | entrée |
| | |
| `admin.login.label_password` | Mot de passe |
| `admin.superuser_wizard.review_password` |   Mot de passe : •••••••• |
| | |
| `admin.reset_password.email_subject` | Réinitialisation de votre mot de passe |
| `reset.email_subject` | Réinitialisation de votre mot de passe |
| | |
| `admin.dashboard.my_groups_staff` | staff |
| `admin.staff` | Staff |
| | |
| `admin.superuser_wizard.email_line` |   Email       : {} |
| `admin.superuser_wizard.review_email` |   Email       : {} |
| | |
| `admin.superuser_wizard.review_username` |   Utilisateur : {} |
| `admin.superuser_wizard.username_line` |   Utilisateur : {} |
| | |
| `error.forbidden` | Accès interdit |
| `error.title.forbidden` | Accès interdit |
| | |
| `error.title.not_found` | Page non trouvée |
| `html.404_text` | Page non trouvée |
| | |

## 4. Même nom de clé dans plusieurs thèmes

| Clé (sans le thème) | Thèmes |
|---|---|
| `forbidden` | `csrf`, `error` |
| `template` | `error`, `title` |

## 5. Clés qui répètent le nom de leur thème

| Clé | Texte (fr) |
|---|---|
| `TemplateMessage.debug_app_version` | Version App |
| `TemplateMessage.debug_available_templates` | Templates disponibles |
| `TemplateMessage.debug_debug_mode` | Mode debug |
| `TemplateMessage.debug_details` | Détails supplémentaires |
| `TemplateMessage.debug_disabled` | Désactivé |
| `TemplateMessage.debug_enabled` | Activé |
| `TemplateMessage.debug_environment` | Environnement |
| `TemplateMessage.debug_error_count` | Erreurs |
| `TemplateMessage.debug_error_line` | Ligne de l'erreur |
| `TemplateMessage.debug_field_count` | Nombre de champs |
| `TemplateMessage.debug_field_title` | Titre |
| `TemplateMessage.debug_header_name` | Nom |
| `TemplateMessage.debug_header_value` | Valeur |
| `TemplateMessage.debug_http_headers` | En-têtes HTTP |
| `TemplateMessage.debug_http_method` | Méthode HTTP |
| `TemplateMessage.debug_info_title` | Informations de debug |
| `TemplateMessage.debug_main_message` | Message principal |
| `TemplateMessage.debug_occurred_at` | Erreur survenue le : |
| `TemplateMessage.debug_path` | Chemin |
| `TemplateMessage.debug_query_params` | Paramètres de requête |
| `TemplateMessage.debug_request_info` | Informations de la requête |
| `TemplateMessage.debug_root_cause` | Cause racine |
| `TemplateMessage.debug_runique_version` | Version Runique |
| `TemplateMessage.debug_rust_version` | Version Rust |
| `TemplateMessage.debug_stack_trace_tip` | L'erreur la plus haute (niveau 0) est généralement la cause racine. L… |
| `TemplateMessage.debug_stack_trace_title` | Trace d'erreur détaillée |
| `TemplateMessage.debug_status_code` | Code de statut |
| `TemplateMessage.debug_template_info` | Informations du template |
| `TemplateMessage.debug_template_name` | Nom du template |
| `TemplateMessage.debug_template_source` | Code source du template |
| `reset.reset_title` | Nouveau mot de passe |

## 6. Clés sans aucune référence trouvée

Ni en toutes lettres dans le code, ni comme variable dans un template, ni sous un préfixe fabriqué à l'exécution. **À vérifier une par une avant de supprimer** : une clé peut être atteinte par un chemin que cette lecture ne voit pas.

| Clé | Texte (fr) |
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
| `forms.filter_invalid_value` | Le filtre static/media a reçu une valeur invalide ({}) au lieu d'un chemin de f… |
| `reset.invalid_link` | Lien invalide. |

## 7. Clés fabriquées à l'exécution

| Modèle dans le code | Fichiers | Clés couvertes |
|---|---|---:|
| `TemplateMessage.{key}` | runique/src/middleware/errors/error.rs | 30 |
| `admin.rank.{rank}` | runique/src/admin/router/account/summary.rs | 5 |
| `admin.rank.{value}` | runique/src/admin/router/permission/rank.rs | 5 |
| `admin.rights.col_{}` | runique/src/admin/router/permission/transfer.rs | 3 |
| `admin.rights.right_{key}` | runique/src/admin/router/admin_router.rs | 7 |
| `admin.rights.right_{}` | runique/src/admin/router/permission/transfer.rs | 7 |
| `admin.{section}.` | runique/src/admin/template/assemble.rs<br>runique/src/admin/trad/mod.rs | 253 |
| `html.{code}_text` | runique/src/middleware/errors/error.rs | 16 |
| `html.{code}_title` | runique/src/middleware/errors/error.rs | 16 |

## 8. Clés écrites dans le code et absentes des fichiers

Vérifiées à la main dans le code : les deux sont de vrais appels à `t()`. Faute de clé dans aucune langue, c'est le texte de la clé qui s'affiche (`admin.logout_success` dans le message flash après déconnexion de l'admin, `admin.password` comme libellé du champ mot de passe du formulaire de connexion).

| Clé | Fichiers |
|---|---|
| `admin.logout_success` | runique/src/admin/router/admin_router.rs |
| `admin.password` | runique/src/auth/form.rs |

## 9. Toutes les clés, thème par thème

Colonne « Comment » : `t` = message simple, `tf` = message avec paramètres `{}`, `liste` = dans `ADMIN_MESSAGE_KEYS` (envoyée aux templates de l'admin), `template` = variable lue dans un template, `autre` = passée par une constante ou une autre fonction.

### `TemplateMessage` (30 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `debug_info_title` | Informations de debug |  |  |  | fabriquée à l'exécution |
| `debug_field_title` | Titre |  |  |  | fabriquée à l'exécution |
| `debug_field_count` | Nombre de champs |  |  |  | fabriquée à l'exécution |
| `debug_error_count` | Erreurs |  |  |  | fabriquée à l'exécution |
| `debug_main_message` | Message principal |  |  |  | fabriquée à l'exécution |
| `debug_root_cause` | Cause racine |  |  |  | fabriquée à l'exécution |
| `debug_details` | Détails supplémentaires |  |  |  | fabriquée à l'exécution |
| `debug_request_info` | Informations de la requête |  |  |  | fabriquée à l'exécution |
| `debug_http_method` | Méthode HTTP |  |  |  | fabriquée à l'exécution |
| `debug_path` | Chemin |  |  |  | fabriquée à l'exécution |
| `debug_query_params` | Paramètres de requête |  |  |  | fabriquée à l'exécution |
| `debug_http_headers` | En-têtes HTTP |  |  |  | fabriquée à l'exécution |
| `debug_template_info` | Informations du template |  |  |  | fabriquée à l'exécution |
| `debug_template_name` | Nom du template |  |  |  | fabriquée à l'exécution |
| `debug_error_line` | Ligne de l'erreur |  |  |  | fabriquée à l'exécution |
| `debug_template_source` | Code source du template |  |  |  | fabriquée à l'exécution |
| `debug_available_templates` | Templates disponibles |  |  |  | fabriquée à l'exécution |
| `debug_stack_trace_title` | Trace d'erreur détaillée |  |  |  | fabriquée à l'exécution |
| `debug_stack_trace_tip` | L'erreur la plus haute (niveau 0) est généralement la … |  |  |  | fabriquée à l'exécution |
| `debug_environment` | Environnement |  |  |  | fabriquée à l'exécution |
| `debug_debug_mode` | Mode debug |  |  |  | fabriquée à l'exécution |
| `debug_enabled` | Activé |  |  |  | fabriquée à l'exécution |
| `debug_disabled` | Désactivé |  |  |  | fabriquée à l'exécution |
| `debug_rust_version` | Version Rust |  |  |  | fabriquée à l'exécution |
| `debug_app_version` | Version App |  |  |  | fabriquée à l'exécution |
| `debug_runique_version` | Version Runique |  |  |  | fabriquée à l'exécution |
| `debug_occurred_at` | Erreur survenue le : |  |  |  | fabriquée à l'exécution |
| `debug_status_code` | Code de statut |  |  |  | fabriquée à l'exécution |
| `debug_header_name` | Nom |  |  |  | fabriquée à l'exécution |
| `debug_header_value` | Valeur |  |  |  | fabriquée à l'exécution |

### `admin` (253 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `group` | Groupe |  |  |  | fabriquée à l'exécution |
| `groups` | Groupes |  |  | template |  |
| `resource` | Ressource |  |  |  | fabriquée à l'exécution |
| `resources` | Ressources |  |  |  | fabriquée à l'exécution |
| `name` | Nom |  |  |  | fabriquée à l'exécution |
| `username` | Nom d'utilisateur |  | 1 | t | auth/form.rs |
| `email` | Email |  |  |  | fabriquée à l'exécution |
| `staff` | Staff |  |  |  | fabriquée à l'exécution |
| `superuser` | Superutilisateur |  |  |  | fabriquée à l'exécution |
| `active` | Actif |  |  |  | fabriquée à l'exécution |
| `user.username_too_short` | Le nom d'utilisateur est trop court (min 3 car.) |  |  |  | fabriquée à l'exécution |
| `user.invalid_email` | Format d'adresse email invalide |  |  |  | fabriquée à l'exécution |
| `user.not_activated` | Ce compte n'a pas encore été activé par son utilisateu… |  |  |  | fabriquée à l'exécution |
| `login.title` | Connexion |  |  | liste, template |  |
| `login.subtitle` | Connectez-vous pour accéder au panneau d'administration |  |  | liste, template |  |
| `login.label_username` | Nom d'utilisateur |  |  | liste, template |  |
| `login.label_password` | Mot de passe |  |  | liste, template |  |
| `login.btn_submit` | Se connecter |  |  | liste, template |  |
| `login.back_to_site` | ← Retour au site |  |  | liste, template |  |
| `login.error_session` | Erreur lors de l'ouverture de session. |  | 1 | t, liste | router/admin_router.rs |
| `login.error_credentials` | Identifiants incorrects ou droits insuffisants. |  | 1 | t, liste | router/admin_router.rs |
| `login.error_locked` | Trop de tentatives. Compte temporairement verrouillé. … | 1 | 1 | tf, liste | router/admin_router.rs |
| `logout.success` | Déconnexion réussie. |  |  | liste |  |
| `access.insufficient_rights` | Droits insuffisants pour accéder à l'administration |  |  | liste |  |
| `access.superuser_only` | Action non autorisée |  |  |  | fabriquée à l'exécution |
| `access.action_not_allowed` | Action non autorisée |  | 6 | autre, t | gate/refusal.rs, middleware/admin_middleware.rs |
| `access.still_responsable` | Ce compte répond encore de groupes : transférez-les d'… |  | 2 | t | gate/save.rs |
| `dashboard.subtitle` | Vue d'ensemble de l'application |  |  | liste, template |  |
| `dashboard.card_resources` | Ressources |  |  | liste, template |  |
| `dashboard.th_resource` | Ressource |  |  | liste |  |
| `dashboard.th_key` | Clé |  |  | liste |  |
| `dashboard.th_permissions` | Permissions |  |  | liste |  |
| `dashboard.th_actions` | Actions |  |  | liste |  |
| `dashboard.btn_list` | Liste |  |  | liste, template |  |
| `dashboard.btn_create` | Créer |  |  | liste |  |
| `dashboard.see_list` | Voir la liste → |  |  | liste |  |
| `dashboard.empty_title` | Aucune ressource enregistrée |  |  | liste, template |  |
| `dashboard.empty_desc` | Enregistrez vos modèles dans src/admin.rs avec le buil… |  |  | liste, template |  |
| `dashboard.kpi_resources` | Ressources |  |  | liste |  |
| `dashboard.kpi_entries` | Entrées totales |  |  | liste |  |
| `dashboard.kpi_largest` | Plus grande |  |  | liste |  |
| `dashboard.th_count` | Entrées |  |  | liste |  |
| `dashboard.toggle_template` | ← Template configuré |  |  | liste, template |  |
| `dashboard.my_groups_title` | Mes groupes |  |  | liste, template |  |
| `dashboard.my_groups_staff` | staff |  |  | liste, template |  |
| `dashboard.my_groups_none` | Ce groupe ne donne aucun droit sur les tables. |  |  | liste, template |  |
| `list.rank_label` | Rang |  |  | liste, template |  |
| `list.groups_label` | Groupes |  |  | liste, template |  |
| `list.entries_count_one` | entrée |  |  | liste, template |  |
| `list.entries_count_many` | entrées |  |  | liste, template |  |
| `list.btn_create` | Créer |  |  | liste, template |  |
| `list.th_id` | id |  |  | liste |  |
| `list.th_actions` | Actions |  |  | liste |  |
| `list.bool_true` | Oui |  |  | liste, template |  |
| `list.bool_false` | Non |  |  | liste, template |  |
| `list.btn_detail` | Détail |  |  | liste, template |  |
| `list.btn_edit` | Modifier |  |  | liste, template |  |
| `list.btn_delete` | Supprimer |  |  | liste, template |  |
| `list.confirm_delete` | Vraiment supprimer ? |  |  | liste |  |
| `list.empty_title` | Aucune entrée |  |  | liste, template |  |
| `list.empty_desc` | Commencez par créer un élément. |  |  | liste, template |  |
| `list.btn_create_first` | Créer le premier |  |  | liste, template |  |
| `list.search_placeholder` | Rechercher... |  |  | liste, template |  |
| `list.per_page` | Par page |  |  | liste, template |  |
| `list.filter_per_page` | Valeurs par page |  |  | liste, template |  |
| `list.bulk_selected` | sélectionné(s) |  |  | liste, template |  |
| `list.bulk_delete` | Supprimer |  |  | liste, template |  |
| `list.filters` | Filtres |  |  | liste, template |  |
| `list.filters_hide` | Masquer les filtres |  |  | liste, template |  |
| `list.filters_clear` | ✕ Tout |  |  | liste, template |  |
| `list.select_all` | Tout sélectionner |  |  | liste, template |  |
| `list.row_details` | Détails |  |  | liste |  |
| `list.btn_rights` | Droits |  |  | liste, template |  |
| `list.action_activate` | Activer |  | 1 | t | builder/site.rs |
| `list.action_deactivate` | Désactiver |  | 1 | t | builder/site.rs |
| `list.responsable_pending` | Responsable à nommer |  |  | liste, template |  |
| `list.taken_by` | repris par |  |  | liste, template |  |
| `create.title` | Créer |  |  | liste, template |  |
| `create.breadcrumb` | Créer |  |  | liste, template |  |
| `create.card_info` | Informations |  |  | liste, template |  |
| `create.no_fields` | Aucun champ disponible pour le moment. |  | 1 | t, liste | crud/create.rs |
| `create.btn_cancel` | Annuler |  | 1 | t, liste | crud/create.rs |
| `create.btn_submit` | Créer |  | 1 | t, liste | crud/create.rs |
| `create.success` | Entrée créée avec succès ! |  | 1 | t, liste | crud/create.rs |
| `edit.title` | Modifier |  |  | liste, template |  |
| `edit.breadcrumb` | Modifier |  |  | liste, template |  |
| `edit.card_info` | Informations |  |  | liste, template |  |
| `edit.no_fields` | Aucun champ disponible pour le moment. |  | 1 | t, liste | crud/edit.rs |
| `edit.btn_cancel` | Annuler |  | 1 | t, liste | crud/edit.rs |
| `edit.btn_submit` | Enregistrer |  | 1 | t, liste | crud/edit.rs |
| `edit.success` | Entrée mise à jour avec succès ! |  | 4 | t, liste | account/memberships.rs, crud/edit.rs, permission/rights.rs |
| `detail.title` | Détail |  |  | liste, template |  |
| `detail.breadcrumb` | Détail |  |  | liste, template |  |
| `detail.entry_label` | Entrée |  |  | liste, template |  |
| `detail.btn_list` | ← Liste |  |  | liste, template |  |
| `detail.btn_edit` | Modifier |  |  | liste, template |  |
| `detail.btn_delete` | Supprimer |  |  | liste, template |  |
| `detail.confirm_delete` | Supprimer cette entrée ? |  |  | liste |  |
| `detail.btn_reset_password` | Réinitialiser le mot de passe |  |  | liste, template |  |
| `detail.btn_reset_password_short` | Réinitialiser |  |  | liste |  |
| `detail.btn_rights` | Droits |  |  | liste, template |  |
| `detail.btn_rank` | Changer le rang |  |  | liste, template |  |
| `detail.groups_title` | Groupes de ce compte |  |  | liste, template |  |
| `detail.btn_save_groups` | Attribuer |  |  | liste, template |  |
| `detail.transfer_title` | Responsable du groupe |  |  | liste, template |  |
| `detail.btn_transfer` | Transférer |  |  | liste, template |  |
| `reset_password.btn` | Réinitialiser le mot de passe |  |  |  | fabriquée à l'exécution |
| `reset_password.success_email` | Lien de réinitialisation envoyé. |  | 1 | t | account/reset_password.rs |
| `reset_password.success_link` | Lien de réinitialisation (mailer non configuré) : {} | 1 |  |  | fabriquée à l'exécution |
| `reset_password.error_no_email` | Aucun champ email trouvé pour cet utilisateur. |  |  |  | fabriquée à l'exécution |
| `reset_password.error_no_mailer` | L'envoi de courriel n'est pas configuré sur ce site : … |  | 2 | t | account/reset_password.rs, crud/create.rs |
| `reset_password.error_unavailable` | Réinitialisation du mot de passe impossible pour cet é… |  | 2 | t | account/reset_password.rs, crud/create.rs |
| `reset_password.error_send` | Erreur lors de l'envoi de l'email : {} | 1 |  |  | fabriquée à l'exécution |
| `reset_password.email_subject` | Réinitialisation de votre mot de passe |  | 1 | autre | account/reset_password.rs |
| `reset_password.email_title` | Réinitialisation du mot de passe |  | 1 | autre | account/reset_password.rs |
| `reset_password.email_greeting` | Bonjour, |  | 1 | autre | account/reset_password.rs |
| `reset_password.email_body` | Cliquez sur le lien ci-dessous pour réinitialiser votr… | 1 | 1 | tf | account/reset_password.rs |
| `reset_password.email_btn` | Réinitialiser mon mot de passe |  | 1 | autre | account/reset_password.rs |
| `reset_password.email_ignore` | Si vous n'avez pas demandé cette réinitialisation, ign… |  | 1 | autre | account/reset_password.rs |
| `user_created.email_subject` | Bienvenue — définissez votre mot de passe |  | 1 | autre | account/reset_password.rs |
| `user_created.email_sent` | Email de bienvenue envoyé. |  | 1 | t | crud/create.rs |
| `user_created.email_greeting` | Bienvenue, |  | 1 | autre | account/reset_password.rs |
| `user_created.email_body` | Un compte a été créé pour vous. Cliquez sur le lien ci… |  | 1 | autre | account/reset_password.rs |
| `user_created.email_btn` | Définir mon mot de passe |  | 1 | autre | account/reset_password.rs |
| `user_created.email_validity` | Ce lien est valide {} min. Si vous n'attendiez pas ce … | 1 | 1 | tf | account/reset_password.rs |
| `delete.title` | Supprimer |  |  | liste, template |  |
| `delete.breadcrumb` | Supprimer |  |  | liste, template |  |
| `delete.heading` | Confirmer la suppression |  |  | liste, template |  |
| `delete.btn_cancel` | ← Annuler |  |  | liste, template |  |
| `delete.btn_confirm` | Oui, supprimer définitivement |  |  | liste, template |  |
| `delete.success` | Entrée supprimée avec succès ! |  | 1 | t, liste | crud/delete.rs |
| `delete.not_found` | Ressource introuvable. |  |  | liste |  |
| `delete.warning.title` | Attention ! |  |  | liste, template |  |
| `delete.warning.desc` | Vous êtes sur le point de supprimer l'entrée |  |  | liste, template |  |
| `delete.warning.of` | de la ressource |  |  | liste, template |  |
| `delete.warning.irreversible` | Cette action est irréversible. |  |  | liste, template |  |
| `bulk.no_selection` | Aucun élément sélectionné. |  | 1 | t | builder/registered.rs |
| `bulk.no_field_selected` | Aucun champ modifié. |  |  |  | fabriquée à l'exécution |
| `bulk.delete_success` | élément(s) supprimé(s). |  | 1 | t | crud/bulk.rs |
| `bulk.update_success` | élément(s) mis à jour. |  | 1 | t | crud/bulk.rs |
| `base.title` | Administration |  |  | liste, template |  |
| `base.breadcrumb` | Admin |  |  | liste, template |  |
| `base.toggle` | Réduire le menu |  |  | liste, template |  |
| `base.logout_title` | Déconnexion |  |  | liste, template |  |
| `base.theme_toggle` | Basculer le thème |  |  | liste, template |  |
| `base.back_to_site` | ← Site |  |  | liste, template |  |
| `base.dashboard` | Tableau de bord |  |  | liste, template |  |
| `base.history` | Historique |  |  | liste, template |  |
| `superuser_wizard.title` | === Créer un superutilisateur === |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.algo_prompt` | [1/5] Algorithme de hachage |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.algo_argon2` | Argon2 (recommandé) |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.algo_bcrypt` | Bcrypt |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.algo_scrypt` | Scrypt |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.algo_custom` | Fournisseur personnalisé |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.provider_path` | Chemin du fournisseur |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.invalid_path` | Chemin invalide. |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.username_prompt` | [2/5] Nom d'utilisateur |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.username_empty` | Le nom d'utilisateur ne peut pas être vide. |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.username_taken` | Un utilisateur avec ce nom existe déjà. |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.email_prompt` | [3/5] Email |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.email_invalid` | Email invalide. |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.email_taken` | Un utilisateur avec cet email existe déjà. |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.password_prompt` | [4/5] Mot de passe |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.password_too_short` | Le mot de passe doit contenir au moins 12 caractères. |  | 3 | autre | src/create_superuser.rs |
| `superuser_wizard.password_weak` | Le mot de passe doit contenir au moins une minuscule, … |  | 2 | autre | src/create_superuser.rs |
| `superuser_wizard.confirm_prompt` | [5/5] Confirmer le mot de passe |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.password_mismatch` | Les mots de passe ne correspondent pas. Veuillez réess… |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.review_algo` |   Algorithme  : {} | 1 | 1 | tf | src/create_superuser.rs |
| `superuser_wizard.review_username` |   Utilisateur : {} | 1 | 1 | tf | src/create_superuser.rs |
| `superuser_wizard.review_email` |   Email       : {} | 1 | 1 | tf | src/create_superuser.rs |
| `superuser_wizard.review_password` |   Mot de passe : •••••••• |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.review_action` | Action |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.confirm_create` | Confirmer et créer |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.change_algo` | Changer d'algorithme |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.modify_password` | Modifier le mot de passe |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.success` | ✓ Superutilisateur créé avec succès ! |  | 1 | t | src/create_superuser.rs |
| `superuser_wizard.id_line` |   ID          : {} | 1 | 1 | tf | src/create_superuser.rs |
| `superuser_wizard.username_line` |   Utilisateur : {} | 1 | 1 | tf | src/create_superuser.rs |
| `superuser_wizard.email_line` |   Email       : {} | 1 | 1 | tf | src/create_superuser.rs |
| `builtin.invalid_id` | Identifiant invalide. |  |  |  | fabriquée à l'exécution |
| `builtin.droit_resource_required` | Veuillez sélectionner au moins une ressource. |  |  |  | fabriquée à l'exécution |
| `builtin.droit_exists` | Le droit "{}" existe déjà. | 1 |  |  | fabriquée à l'exécution |
| `builtin.groupe_exists` | Un groupe nommé « {} » existe déjà. | 1 |  |  | fabriquée à l'exécution |
| `builtin.user_username_exists` | Le nom d'utilisateur « {} » est déjà pris. | 1 |  |  | fabriquée à l'exécution |
| `builtin.user_email_exists` | L'adresse e-mail « {} » est déjà utilisée. | 1 |  |  | fabriquée à l'exécution |
| `bulk_create.all_exist` | Toutes les valeurs existent déjà : rien n'a été créé |  |  |  | fabriquée à l'exécution |
| `history.subtitle` | Les créations, modifications et suppressions faites da… |  |  | liste, template |  |
| `history.whole_row` | Toute l'histoire de cette ligne |  |  | liste, template |  |
| `history.th_column` | Colonne |  |  | liste, template |  |
| `history.th_old` | Avant |  |  | liste, template |  |
| `history.th_new` | Après |  |  | liste, template |  |
| `history.no_changes` | Aucun changement lisible |  |  | liste, template |  |
| `history.all_resources` | Toutes les ressources |  |  | liste, template |  |
| `history.all_actions` | Toutes les actions |  |  | liste, template |  |
| `history.action_create` | Créé |  |  | liste, template |  |
| `history.action_update` | Modifié |  |  | liste, template |  |
| `history.action_delete` | Supprimé |  |  | liste, template |  |
| `history.btn_filter` | Filtrer |  |  | liste, template |  |
| `history.btn_reset` | Réinitialiser |  |  | liste, template |  |
| `history.th_date` | Date |  |  | liste, template |  |
| `history.th_account` | Compte |  |  | liste, template |  |
| `history.th_resource` | Ressource |  |  | liste, template |  |
| `history.th_object` | Objet |  |  | liste, template |  |
| `history.th_action` | Action |  |  | liste, template |  |
| `history.btn_entry` | Détail |  |  | liste, template |  |
| `history.empty` | Aucune action enregistrée |  |  | liste, template |  |
| `history.badge_create` | créé |  |  | liste, template |  |
| `history.badge_update` | modifié |  |  | liste, template |  |
| `history.badge_delete` | supprimé |  |  | liste, template |  |
| `history.badge_rights` | droits modifiés |  |  | liste, template |  |
| `history.badge_rank` | rang modifié |  |  | liste, template |  |
| `history.badge_reset_password` | lien de mot de passe envoyé |  |  | liste, template |  |
| `history.badge_groups` | Groupes |  |  | liste, template |  |
| `history.badge_transfer` | Transfert |  |  | liste, template |  |
| `rights.title` | Droits |  |  | liste, template |  |
| `rights.subtitle` | Ce que cet admin peut faire, table par table et colonn… |  |  | liste, template |  |
| `rights.go_to_table` | Aller à une table |  |  | liste, template |  |
| `rights.toc` | Tables |  |  | liste, template |  |
| `rights.btn_cancel` | Annuler |  |  | liste, template |  |
| `rights.btn_save` | Enregistrer |  |  | liste, template |  |
| `rights.th_column` | Colonne |  |  | liste, template |  |
| `rights.col_read` | Lire |  |  | liste, template |  |
| `rights.col_create` | Créer |  |  | liste, template |  |
| `rights.col_update` | Modifier |  |  | liste, template |  |
| `rights.right_create` | Créer |  | 1 | autre | permission/rights.rs |
| `rights.right_read` | Lister / lire |  | 1 | autre | permission/rights.rs |
| `rights.right_read_own` | Lire les siennes |  | 1 | autre | permission/rights.rs |
| `rights.right_update` | Modifier |  | 1 | autre | permission/rights.rs |
| `rights.right_update_own` | Modifier les siennes |  | 1 | autre | permission/rights.rs |
| `rights.right_delete` | Supprimer |  | 1 | autre | permission/rights.rs |
| `rights.right_delete_own` | Supprimer les siennes |  | 1 | autre | permission/rights.rs |
| `rights.group_subtitle` | Ce que les membres de ce groupe peuvent faire, table p… |  |  | liste, template |  |
| `rights.group_label` | Groupe |  |  | liste, template |  |
| `rights.btn_show` | Afficher |  |  | liste, template |  |
| `rights.group_sheet` | Fiche du groupe |  |  | liste, template |  |
| `rank.superuser` | Superutilisateur |  |  |  | fabriquée à l'exécution |
| `rank.lambda` | Compte du site |  |  |  | fabriquée à l'exécution |
| `rank.admin` | Admin |  |  |  | fabriquée à l'exécution |
| `rank.dirigeant` | Dirigeant |  |  |  | fabriquée à l'exécution |
| `rank.success` | Rang modifié ({}) | 1 | 1 | tf | permission/rank.rs |
| `transfer.title` | Transférer le groupe |  |  | liste, template |  |
| `transfer.from` | Responsable actuel |  |  | liste, template |  |
| `transfer.to` | Nouveau responsable |  |  | liste, template |  |
| `transfer.losses` | Droits en jeu |  |  | liste, template |  |
| `transfer.no_loss` | Le groupe garde tous ses droits. |  |  | liste, template |  |
| `transfer.btn_confirm` | Confirmer le transfert |  |  | liste, template |  |
| `transfer.btn_cancel` | Annuler |  |  | liste, template |  |
| `transfer.unsuitable` | Cet utilisateur ne convient pas. |  | 3 | t | gate/transfer.rs, permission/transfer.rs |
| `transfer.ranked_members` | Ce groupe compte des membres avec un rang : un admin n… |  | 1 | t | gate/transfer.rs |
| `transfer.changed` | Les droits ont changé entre-temps : vérifiez à nouveau. |  | 1 | t | permission/transfer.rs |
| `transfer.success` | Groupe transféré. |  | 1 | t | permission/transfer.rs |
| `transfer.gained_right` | reçu par |  |  | liste, template |  |
| `transfer.lost_right` | perdu par les membres |  |  | liste, template |  |

### `atomic` (16 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `reserved_name` | {} : ce nom est réservé à Runique | 1 | 1 | tf | src/builder.rs |
| `not_a_file` | {} : ce n'est pas le chemin d'un fichier | 1 | 1 | tf | src/builder.rs |
| `duplicate` | {} : ce fichier est déjà dans le lot | 1 | 1 | tf | src/builder.rs |
| `in_the_way` | {} occupe la place : ce fichier ne vient pas du lancem… | 1 | 1 | tf | step/waiting.rs |
| `outside_journal` | {} : hors du dossier du journal | 1 | 1 | autre | src/journal.rs |
| `leaves_journal` | {} : ce chemin sort du dossier du journal | 1 | 1 | autre | src/journal.rs |
| `journal_forged` | {} : le journal nomme un chemin que Runique n'a pas pu… | 1 | 2 | autre | src/journal.rs |
| `no_journal` | {} : aucun journal à marquer | 1 | 1 | autre | src/journal.rs |
| `stray_backup` | {} : une sauvegarde existe, alors qu'il n'y avait rien… | 1 | 1 | autre | src/journal.rs |
| `gone` | {} : le fichier a disparu, et aucune sauvegarde ne le … | 1 | 1 | autre | src/journal.rs |
| `not_created` | {} : ce n'est pas le fichier que Runique était en trai… | 1 | 1 | autre | src/journal.rs |
| `changed` | {} : modifié depuis l'interruption, et sa sauvegarde c… | 1 | 1 | autre | src/journal.rs |
| `stranger` | {} : ce fichier ne vient pas de Runique, dans un dossi… | 1 | 1 | tf | src/journal.rs |
| `moved_out` | {} : ce fichier n'est plus dans le dossier du journal | 1 | 1 | autre | src/journal.rs |
| `unfinished` | le lot n'a pas abouti après ses passes |  | 1 | t | src/manager.rs |
| `wrong_step` | cette action est impossible à cette étape du fichier |  | 3 | t | src/builder.rs, src/tests.rs, step/mod.rs |

### `auth` (2 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `login_cannot_sign_in` | Ce compte ne peut pas se connecter : il est inactif ou… |  | 1 | t | auth/session.rs |
| `login_session_failed` | La session n'a pas pu être enregistrée : {} | 1 | 1 | tf | auth/session.rs |

### `build` (28 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `subscriber_already_installed` | Un subscriber tracing est déjà installé ; la configura… |  | 1 | t | runique_log/mod.rs |
| `external_outputs_ignored` | `with_log(\|l\| l.external())` est défini ; les sortie… |  | 1 | t | runique_log/mod.rs |
| `validation_failed` | Échec de la validation du build : {} | 1 | 1 | tf | app/error_build.rs |
| `check_failed` | Échec du build avec {} erreur(s) de vérification | 1 | 1 | tf | app/error_build.rs |
| `template_failed` | Échec du chargement du template : {} | 1 | 1 | tf | app/error_build.rs |
| `driver_not_enabled` | Pilote {} non activé : Runique a été compilé avec '{}'… | 8 | 1 | tf | db/engine.rs |
| `db_missing` | Connexion à la base de données requise lorsque la feat… |  | 1 | t | app/error_build.rs |
| `component_not_ready` | Le composant '{}' n'est pas prêt | 1 | 1 | tf | app/error_build.rs |
| `check.db_required` | Connexion ou configuration de base de données requise … |  | 1 | t | staging/core_staging.rs |
| `check.db_required_hint` | Ajoutez .with_database(db) ou .with_database_config(co… |  | 1 | t | staging/core_staging.rs |
| `check.admin_prefix_empty` | Le préfixe de route de l'admin ne peut pas être vide |  | 1 | t | staging/admin_staging.rs |
| `check.admin_prefix_empty_hint` | Utilisez .prefix("/admin") ou gardez la valeur par déf… |  | 1 | t | staging/admin_staging.rs |
| `check.cache_control_invalid` | Valeur d'en-tête Cache-Control invalide : '{}' | 1 | 1 | tf | staging/static_staging.rs |
| `check.cache_control_invalid_hint` | N'utilisez que des caractères ASCII visibles (pas de c… |  | 1 | autre | staging/static_staging.rs |
| `check.media_root_unusable` | Impossible de créer ou d'accéder au dossier MEDIA_ROOT… | 2 | 1 | autre | staging/static_staging.rs |
| `check.media_root_unusable_hint` | Définissez MEDIA_ROOT sur un chemin absolu accessible … |  | 1 | autre | staging/static_staging.rs |
| `check.secret_key_weak` | SECRET_KEY est absente, vide, à sa valeur par défaut, … |  | 1 | t | builder/build.rs |
| `check.secret_key_weak_hint` | Définissez SECRET_KEY à une chaîne aléatoire d'au moin… |  | 1 | t | builder/build.rs |
| `check.public_url_missing` | Aucune URL publique : les liens de reset seraient cons… |  | 1 | t | builder/build.rs |
| `check.public_url_missing_hint` | Définissez-la dans le builder : .with_public_url("http… |  | 1 | t | builder/build.rs |
| `check.acme_domain_missing` | ACME_ENABLED=true mais ACME_DOMAIN n'est pas défini |  | 1 | t | builder/build.rs |
| `check.acme_domain_missing_hint` | Définissez ACME_DOMAIN à votre domaine de production d… |  | 1 | autre | builder/build.rs |
| `check.acme_email_missing` | ACME_ENABLED=true mais ACME_EMAIL n'est pas défini |  | 1 | t | builder/build.rs |
| `check.acme_email_missing_hint` | Définissez ACME_EMAIL à votre email de contact Let's E… |  | 1 | t | builder/build.rs |
| `check.suggestion` | Suggestion |  | 1 | t | app/error_build.rs |
| `check.context` | Contexte |  | 1 | t | app/error_build.rs |
| `check.mailer_missing` | Réinitialisation du mot de passe sans mailer : le lien… |  | 1 | t | builder/build.rs |
| `check.mailer_missing_hint` | Renseignez-le dans le .env : EMAIL_BACKEND, ou SMTP_HO… |  | 1 | t | builder/build.rs |

### `cli` (14 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `creating_project` | Création du projet '{}'... | 1 | 1 | tf | new_project/mod.rs |
| `name_empty` | Le nom du projet ne peut pas être vide |  | 1 | t | new_project/mod.rs |
| `name_invalid` | Le nom du projet ne doit contenir que des lettres, chi… |  | 1 | t | new_project/mod.rs |
| `name_dash` | Le nom du projet ne peut pas commencer par - |  | 1 | t | new_project/mod.rs |
| `folder_exists` | Le dossier '{}' existe déjà | 1 | 1 | tf | new_project/mod.rs |
| `interrupted_run` | Un lancement a été interrompu dans '{}' et des fichier… | 1 | 1 | tf | cli_migration/init_migrate.rs |
| `run_repaired` | Un lancement interrompu a été réparé dans '{}'. | 1 | 1 | tf | cli_migration/init_migrate.rs |
| `project_created` | Projet '{}' créé avec succès ! | 1 | 1 | tf | new_project/mod.rs |
| `getting_started` | Pour démarrer : |  | 1 | t | new_project/mod.rs |
| `cd_hint` | cd {} | 1 | 1 | tf | new_project/mod.rs |
| `cargo_run_hint` | cargo run |  | 1 | t | new_project/mod.rs |
| `name_digit` | Le nom du projet ne peut pas commencer par un chiffre |  | 1 | t | new_project/mod.rs |
| `name_reserved` | '{}' est réservé (mot-clé ou crate de Rust, nom de car… | 1 | 1 | tf | new_project/mod.rs |
| `migration_up_hint` | runique migration up      (crée les tables du framewor… |  | 1 | t | new_project/mod.rs |

### `csrf` (5 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `missing` | Token CSRF manquant |  | 2 | t | fields/hidden.rs, forms/extractor.rs |
| `invalid` | Token CSRF invalide |  | 1 | t | fields/hidden.rs |
| `invalid_or_missing` | Token CSRF invalide ou manquant |  | 8 | t | account/memberships.rs, account/reset_password.rs, context/template.rs +4 |
| `forbidden` | Token CSRF invalide |  |  |  | **aucune référence** |
| `session_write_error` | Erreur d'écriture de session |  |  |  | **aucune référence** |

### `database` (12 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `reason_unknown_start` | son début ne désigne aucun moteur pris en charge |  | 4 | autre | db/config.rs |
| `reason_unreadable` | elle est illisible |  | 3 | autre | db/config.rs |
| `reason_no_database` | elle ne nomme aucune base |  | 5 | autre | db/config.rs |
| `reason_sqlite_no_file` | elle demande SQLite sans nommer de fichier |  | 4 | autre, t | db/config.rs |
| `url_incomplete_using_keys` | DATABASE_URL est incomplète ({}) : connexion par les c… | 1 | 1 | tf | db/config.rs |
| `url_incomplete_reminder` | Connexion établie par les clés DB_*. DATABASE_URL rest… | 1 | 1 | tf | db/config.rs |
| `url_incomplete_no_keys` | DATABASE_URL est incomplète ({}) et aucune clé DB_* ne… | 1 | 2 | tf | db/config.rs |
| `url_incomplete_keys_unusable` | DATABASE_URL est incomplète ({}), et les clés DB_* ne … | 2 | 2 | autre | db/config.rs |
| `url_sqlite_default_file` | DATABASE_URL demande SQLite sans nommer de fichier : l… | 1 | 1 | tf | db/config.rs |
| `keys_without_engine` | DB_ENGINE manque : {} est renseigné sans le moteur. Va… | 1 | 3 | tf | db/config.rs |
| `url_failed_trying_keys` | La connexion par DATABASE_URL a échoué ({}) : nouvel e… | 1 | 1 | tf | db/config.rs |
| `url_failed_keys_unusable` | La connexion par DATABASE_URL a échoué ({}), et les cl… | 2 | 1 | autre | db/config.rs |

### `debug` (2 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `stack_trace_title` | Trace d'erreur détaillée |  |  | template |  |
| `stack_trace_tip` | L'erreur la plus haute (niveau 0) est généralement la … |  |  | template |  |

### `error` (22 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `source` | Source: {} | 1 | 1 | tf | errors/error.rs |
| `AppError` | Erreur d'application |  | 1 | t | errors/error.rs |
| `build` | Erreur de build: {} | 1 | 2 | tf | errors/error.rs |
| `internal` | Erreur interne |  | 2 | t | errors/error.rs |
| `forbidden` | Accès interdit |  | 3 | t | errors/error.rs |
| `not_found` | Ressource introuvable |  | 3 | t | errors/error.rs, forms/save.rs |
| `validation` | Erreur de validation: {} | 1 | 2 | t, tf | errors/error.rs |
| `database` | Erreur base de données: {} | 1 | 3 | t, tf | errors/error.rs |
| `io` | Erreur IO: {} | 1 | 2 | tf | errors/error.rs |
| `template` | Erreur template: {} | 1 | 3 | t, tf | errors/error.rs |
| `custom` | Erreur custom: {} | 1 | 1 | tf | errors/error.rs |
| `title.bad_request` | Requête invalide |  | 1 | t | errors/error.rs |
| `title.not_found` | Page non trouvée |  | 3 | t | bdd/objects.rs, bdd/query.rs, errors/error.rs |
| `title.forbidden` | Accès interdit |  |  |  | **aucune référence** |
| `title.validation` | Erreur de validation |  |  |  | **aucune référence** |
| `title.database` | Erreur base de données |  |  |  | **aucune référence** |
| `title.template` | Erreur de rendu de template |  |  |  | **aucune référence** |
| `title.internal` | Erreur serveur interne |  | 1 | t | errors/error.rs |
| `invalid_query` | Paramètres d'URL invalides : {} | 1 | 1 | autre | context/template.rs |
| `path_not_found` | Le chemin demandé '{}' n'a pas été trouvé | 1 | 1 | tf | errors/error.rs |
| `internal_occurred` | Une erreur interne est survenue |  | 2 | t | errors/error.rs |
| `context_occurred` | Erreur AppError survenue |  |  |  | **aucune référence** |

### `flash` (6 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `welcome` | Bienvenue {}, votre compte est créé ! | 1 |  |  | **aucune référence** |
| `fix_errors` | Veuillez corriger les erreurs |  |  |  | **aucune référence** |
| `success_demo` | Ceci est un message de succès. |  |  |  | **aucune référence** |
| `info_demo` | Ceci est un message d'information. |  |  |  | **aucune référence** |
| `warning_demo` | Ceci est un message d'avertissement. |  |  |  | **aucune référence** |
| `error_demo` | Ceci est un message d'erreur. |  |  |  | **aucune référence** |

### `forms` (95 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `upload_dir_error` | Erreur lors de la création du dossier d'upload |  | 3 | t | forms/parse_html.rs |
| `file_create_error` | Erreur lors de la création du fichier |  | 1 | t | forms/parse_html.rs |
| `multipart_stream_error` | Erreur dans le flux multipart |  | 1 | t | forms/parse_html.rs |
| `file_write_error` | Erreur lors de l'écriture du fichier |  | 2 | t | forms/parse_html.rs |
| `required` | Ce champ est obligatoire |  | 14 | autre, t | fields/datetime.rs, fields/hidden.rs, fields/special.rs +3 |
| `too_short` | Trop court (min {}) | 1 | 1 | tf | fields/text.rs |
| `too_long` | Trop long (max {}) | 1 | 1 | tf | fields/text.rs |
| `email_invalid` | Format d'adresse email invalide |  | 1 | t | fields/text.rs |
| `url_invalid` | Veuillez entrer une URL valide |  | 1 | t | fields/text.rs |
| `phone_invalid` | Numéro de téléphone invalide (chiffres, +, espaces, ti… |  | 1 | t | fields/text.rs |
| `hash_error` | Erreur de hachage : {} | 1 | 4 | tf | fields/text.rs, password/mod.rs |
| `password_empty` | Le mot de passe ne peut pas être vide |  | 3 | t | password/mod.rs |
| `filter_invalid_value` | Le filtre static/media a reçu une valeur invalide ({})… | 1 |  |  | **aucune référence** |
| `number_invalid` | Nombre invalide |  | 4 | t | fields/number.rs |
| `number_required` | Requis |  | 1 | autre | fields/number.rs |
| `integer_required` | Doit être un entier |  | 1 | t | fields/number.rs |
| `decimal_required` | Doit être au format décimal |  |  |  | **aucune référence** |
| `min_value` | Min {} requis | 1 | 3 | tf | fields/number.rs |
| `max_value` | Max {} dépassé | 1 | 3 | tf | fields/number.rs |
| `precision_max` | Précision max {} chiffres après virgule | 1 | 1 | autre | fields/number.rs |
| `integer_digits_max` | {} chiffres maximum avant la virgule | 1 | 1 | tf | fields/number.rs |
| `value_unconvertible` | Cette valeur ne correspond pas à ce champ |  | 6 | t | fields/boolean.rs, forms/column_value.rs, gate/change.rs |
| `must_be_checked` | Cette case doit être cochée |  | 1 | t | fields/boolean.rs |
| `precision_min` | Précision min {} chiffres après virgule | 1 | 1 | tf | fields/number.rs |
| `color_invalid` | Couleur invalide (format: #RRGGBB ou #RGB) |  | 1 | t | fields/special.rs |
| `color_no_hash` | La couleur doit commencer par # |  | 1 | t | fields/special.rs |
| `color_bad_hex` | Contient des caractères non hexadécimaux |  | 1 | t | fields/special.rs |
| `slug_invalid` | Slug invalide |  | 1 | t | fields/special.rs |
| `slug_no_dash` | Le slug ne doit pas commencer ou finir par un tiret |  | 1 | t | fields/special.rs |
| `uuid_invalid` | Format UUID invalide (attendu: xxxxxxxx-xxxx-xxxx-xxxx… |  | 1 | t | fields/special.rs |
| `json_invalid` | JSON invalide : {} | 1 | 1 | tf | fields/special.rs |
| `ipv4_only` | Seules les adresses IPv4 sont acceptées |  | 1 | t | fields/special.rs |
| `ipv6_only` | Seules les adresses IPv6 sont acceptées |  | 1 | t | fields/special.rs |
| `ip_invalid` | Adresse IP invalide |  | 1 | t | fields/special.rs |
| `date_invalid` | Format de date invalide |  | 2 | t | fields/datetime.rs |
| `date_too_old` | Trop ancien (min : {}) | 1 | 1 | tf | fields/datetime.rs |
| `date_too_far` | Trop loin (max : {}) | 1 | 1 | tf | fields/datetime.rs |
| `time_invalid` | Format de temps invalide |  | 1 | t | fields/datetime.rs |
| `file_required` | Veuillez sélectionner au moins un fichier |  | 2 | autre, t | fields/binary.rs, fields/media.rs |
| `data_invalid` | Valeur invalide pour « {} » | 1 | 1 | tf | forms/form_data.rs |
| `data_required` | « {} » est obligatoire | 1 | 1 | tf | forms/form_data.rs |
| `data_password` | Le mot de passe de « {} » n'a pas pu être haché : rien… | 1 | 1 | tf | forms/form_data.rs |
| `file_extension_blocked` | Type de fichier non autorisé. Extensions acceptées : {} | 1 | 1 | tf | fields/media.rs |
| `finalize_error` | Erreur lors de la finalisation du champ '{}': {} | 2 | 3 | tf | forms/base.rs, forms/form.rs, forms/renderer.rs |
| `template_missing` | Template manquant: {} | 1 | 2 | tf | forms/renderer.rs |
| `render_js_error` | Erreur rendu JS: {} | 1 | 1 | tf | forms/renderer.rs |
| `render_errors_error` | Erreur rendu des erreurs du formulaire: {} | 1 | 1 | tf | forms/renderer.rs |
| `tera_not_configured` | Tera non configuré |  | 1 | t | forms/form.rs |
| `config_not_found` | Configuration introuvable dans les extensions |  | 1 | t | forms/extractor.rs |
| `hint_slug` | Format : lettres-chiffres-tirets |  | 1 | t | fields/special.rs |
| `hint_uuid` | Format : xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx |  | 1 | t | fields/special.rs |
| `hint_json` | Format JSON valide requis |  | 1 | t | fields/special.rs |
| `hint_ip_v4` | Format IPv4 : 192.168.1.1 |  | 1 | t | fields/special.rs |
| `hint_ip_v6` | Format IPv6 : 2001:0db8:85a3::8a2e:0370:7334 |  | 1 | t | fields/special.rs |
| `hint_ip` | Format IPv4 ou IPv6 |  | 1 | t | fields/special.rs |
| `validation_overflow` | Stack overflow détecté : récursion infinie dans la val… |  | 2 | t | forms/field.rs, forms/validator.rs |
| `choice_invalid` | Choix invalide |  | 2 | t | fields/choice.rs |
| `choice_invalid_value` | Choix invalide : {} | 1 | 2 | tf | fields/choice.rs |
| `select_required` | Veuillez sélectionner une option |  | 2 | autre | fields/choice.rs |
| `checkbox_required` | Veuillez sélectionner au moins une option |  | 1 | autre | fields/choice.rs |
| `multipart_error` | Erreur multipart |  | 1 | t | prisme/aegis.rs |
| `body_error` | Erreur lecture du body |  | 1 | t | prisme/aegis.rs |
| `aegis_query_error` | Échec parsing query string : {} | 1 | 1 | tf | prisme/aegis.rs |
| `aegis_urlencoded_error` | Échec parsing urlencoded : {} | 1 |  |  | **aucune référence** |
| `aegis_json_error` | Échec parsing JSON : {} | 1 | 1 | tf | prisme/aegis.rs |
| `time_too_old` | Trop tôt (min : {}) | 1 | 1 | tf | fields/datetime.rs |
| `time_too_far` | Trop tard (max : {}) | 1 | 1 | tf | fields/datetime.rs |
| `datetime_too_old` | Trop tôt (min : {}) | 1 | 1 | tf | fields/datetime.rs |
| `datetime_too_far` | Trop tard (max : {}) | 1 | 1 | tf | fields/datetime.rs |
| `duration_invalid` | Durée invalide (nombre de secondes attendu) |  | 1 | t | fields/datetime.rs |
| `duration_too_short` | Durée minimale : {} secondes | 1 | 1 | tf | fields/datetime.rs |
| `duration_too_long` | Durée maximale : {} secondes | 1 | 1 | tf | fields/datetime.rs |
| `file_max_count` | Maximum {} fichiers autorisés | 1 | 1 | tf | fields/media.rs |
| `file_too_large` | Fichier trop volumineux, taille maximale : {} Mo | 1 | 2 | autre | fields/binary.rs, fields/media.rs |
| `file_unreadable` | Le fichier envoyé n'a pas pu être lu, merci de l'envoy… |  | 3 | t | fields/binary.rs, fields/media.rs |
| `file_invalid_image` | Ce fichier n'est pas une image valide |  | 2 | t | fields/media.rs |
| `file_invalid_audio` | Ce fichier n'est pas un fichier audio valide |  | 2 | t | fields/media.rs |
| `file_invalid_video` | Ce fichier n'est pas une vidéo valide |  | 2 | t | fields/media.rs |
| `image_too_wide` | Image trop large, largeur maximale : {} px | 1 | 1 | tf | fields/media.rs |
| `image_too_tall` | Image trop haute, hauteur maximale : {} px | 1 | 1 | tf | fields/media.rs |
| `boolean_required` | Vous devez accepter ce champ |  |  |  | **aucune référence** |
| `slug_unicode_invalid` | Seules les lettres, chiffres, tirets et underscores so… |  | 1 | t | fields/special.rs |
| `unique_field_taken` | Ce {} est déjà utilisé. | 1 | 2 | tf | forms/form.rs |
| `unique_value_taken` | La valeur du champ '{}' est déjà utilisée. | 1 | 1 | tf | forms/form.rs |
| `unique_constraint_violated` | Une contrainte d'unicité a été violée. |  | 2 | t | forms/form.rs |
| `db_error` | L'enregistrement a été refusé par la base. |  | 4 | autre, t | forms/form.rs |
| `db_foreign_key` | Cette valeur ne correspond à aucun enregistrement lié. |  | 2 | autre, t | forms/form.rs |
| `db_not_null` | Une valeur obligatoire est manquante. |  | 2 | autre, t | forms/form.rs |
| `db_check` | Cette valeur n'est pas acceptée. |  | 3 | autre, t | forms/form.rs |
| `delete_linked` | Impossible de supprimer : d'autres enregistrements l'u… |  | 3 | t | gate/refusal.rs, gate/write.rs |
| `text_field_too_large` | Champ texte trop volumineux (max 1 Mo) |  | 1 | t | forms/parse_html.rs |
| `upload_too_large` | Fichier trop volumineux (max {}Mo) | 1 | 1 | autre | forms/parse_html.rs |
| `body_too_large` | Requête trop volumineuse |  | 3 | t | forms/parse_html.rs, prisme/aegis.rs |
| `upload_failed` | Le fichier n'a pas pu être enregistré. Réessayez. |  | 2 | t | fields/file.rs, fields/media.rs |
| `max_size_above_model` | La taille maximale demandée ({} Mo) dépasse la limite … | 2 | 2 | autre, tf | fields/file.rs, fields/media.rs |

### `html` (16 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `400_title` | 400 |  |  |  | fabriquée à l'exécution |
| `400_text` | Requête invalide : les paramètres de l'adresse ne sont… |  |  |  | fabriquée à l'exécution |
| `413_title` | 413 |  |  |  | fabriquée à l'exécution |
| `413_text` | Envoi trop volumineux : réduisez sa taille et réessaye… |  |  |  | fabriquée à l'exécution |
| `404_title` | 404 |  |  |  | fabriquée à l'exécution |
| `404_text` | Page non trouvée |  |  |  | fabriquée à l'exécution |
| `500_title` | 500 |  |  |  | fabriquée à l'exécution |
| `500_text` | Une erreur est survenue sur le serveur |  |  |  | fabriquée à l'exécution |
| `429_title` | 429 |  |  |  | fabriquée à l'exécution |
| `429_text` | Trop de requêtes. Veuillez patienter un moment avant d… |  | 1 | t | security/rate_limit.rs |
| `503_title` | 503 |  |  |  | fabriquée à l'exécution |
| `503_text` | Service temporairement indisponible en raison d'une fo… |  |  |  | fabriquée à l'exécution |
| `back_home` | Retour à l'accueil |  | 2 | t | errors/error.rs |
| `critical_error_title` | Erreur critique |  | 2 | t | errors/error.rs |
| `critical_error_text` | Le système de gestion d'erreurs a lui-même rencontré u… |  | 1 | t | errors/error.rs |
| `critical_error_contact` | Cette situation ne devrait jamais se produire. Veuille… |  | 1 | t | errors/error.rs |

### `log` (53 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `csrf.refused` | CSRF refusé |  | 1 | t | security/csrf.rs |
| `csrf.reason_header_invalid` | jeton invalide dans l'en-tête X-CSRF-Token |  | 1 | autre | security/csrf.rs |
| `csrf.reason_not_a_form` | aucun jeton, et le corps n'est pas un formulaire |  | 1 | autre | security/csrf.rs |
| `csrf.reason_cross_site` | formulaire envoyé depuis un autre site |  | 1 | autre | security/csrf.rs |
| `csrf.reason_form_mismatch` | formulaire : le jeton envoyé n'est pas celui de la ses… |  | 1 | autre | context/template.rs |
| `csrf.reason_form_missing` | formulaire : aucun jeton dans le corps ni dans X-CSRF-… |  | 1 | autre | context/template.rs |
| `csrf.reason_admin_login_missing` | connexion à l'admin : aucun jeton dans le formulaire |  | 1 | autre | router/admin_router.rs |
| `csrf.reason_admin_login_mismatch` | connexion à l'admin : le jeton envoyé n'est pas celui … |  | 1 | autre | router/admin_router.rs |
| `csrf.reason_admin_logout` | déconnexion de l'admin : aucun jeton valide |  | 1 | autre | router/admin_router.rs |
| `csrf.reason_upload` | envoi de fichier : un fichier est arrivé avant un cham… |  | 1 | autre | forms/parse_html.rs |
| `csrf.token_dropped` | jeton CSRF abandonné (connexion ou déconnexion) : les … |  | 1 | t | security/csrf.rs |
| `csrf.token_new` | nouveau jeton CSRF pour cette session |  | 1 | t | security/csrf.rs |
| `csrf.read_token` | lecture du jeton CSRF dans la session |  | 1 | autre | security/csrf.rs |
| `csrf.keep_token` | conservation du jeton CSRF dans la session |  | 1 | autre | security/csrf.rs |
| `csrf.read_account` | lecture du compte connecté pour le jeton CSRF |  | 1 | autre | security/csrf.rs |
| `csrf.write_token` | écriture d'un nouveau jeton CSRF dans la session |  | 1 | autre | security/csrf.rs |
| `csrf.set_value` | set_value [jeton CSRF reçu : son contrôle est sur le c… |  | 1 | t | forms/form.rs |
| `session.read_account` | lecture du compte connecté dans la session |  | 2 | autre | auth/session.rs |
| `session.read_version` | lecture de la version de la session |  | 1 | autre | auth/session.rs |
| `session.read_flash` | lecture des messages flash dans la session |  | 2 | autre | flash/flash_manager.rs |
| `session.read_dashboard_template` | lecture du gabarit de tableau de bord choisi dans la s… |  | 2 | autre | router/admin_router.rs |
| `session.decode_user_memory` | décodage du user_id de la session (chargement depuis l… |  | 1 | autre | session/cleaning_store.rs |
| `session.decode_user_db` | décodage du user_id de la session (chargement depuis l… |  | 1 | autre | session/cleaning_store.rs |
| `anti_bot.read_name` | lecture du nom du champ piège dans la session |  | 1 | autre | security/anti_bot.rs |
| `anti_bot.filled` | champ piège rempli : formulaire refusé |  | 1 | t | context/template.rs |
| `https.redirected` | requête reçue en HTTP : redirigée vers HTTPS |  | 1 | t | security/csp.rs |
| `https.port80_stopped` | le serveur HTTP du port 80 (défi ACME et redirection H… |  | 1 | autre | app/runique_app.rs |
| `auth.hash_failed` | échec du hachage du mot de passe à la réinitialisation |  | 1 | t | auth/password.rs |
| `auth.account_not_read` | compte non lu : session gardée, requête répondue par u… |  | 1 | t | middleware_staging/applicator.rs |
| `auth.admin_account_not_read` | compte de l'admin non lu : session gardée, requête rép… |  | 1 | t | middleware/admin_middleware.rs |
| `forms.validation_failed` | échec de la validation du formulaire |  | 1 | t | forms/field.rs |
| `forms.remove_oversized` | suppression d'un fichier en transit trop volumineux |  | 1 | autre | fields/binary.rs |
| `forms.remove_replaced` | suppression du fichier qu'un nouvel envoi remplace |  | 1 | autre | fields/media.rs |
| `mailer.init_twice` | mailer_init() appelé plusieurs fois : la configuration… |  | 1 | t | mailer/mod.rs |
| `db.read_reset_token` | lecture d'un jeton de réinitialisation de mot de passe |  | 1 | autre | reset_token/mod.rs |
| `admin.write` | écriture dans l'admin |  | 1 | t | builder/registered.rs |
| `admin.outcome_saved` | enregistré |  | 1 | autre | builder/registered.rs |
| `admin.outcome_save_refused` | enregistrement refusé |  | 1 | autre | builder/registered.rs |
| `admin.outcome_deleted` | supprimé |  | 1 | autre | builder/registered.rs |
| `admin.outcome_delete_refused` | suppression refusée |  | 1 | autre | builder/registered.rs |
| `admin.bulk_done` | action groupée effectuée |  | 1 | t | builder/registered.rs |
| `admin.bulk_refused` | action groupée refusée |  | 1 | t | builder/registered.rs |
| `admin.list` | liste de l'admin |  | 1 | t | crud/list.rs |
| `boot.admin` | admin |  | 1 | t | staging/admin_staging.rs |
| `boot.model` | modèle de l'admin |  | 1 | t | staging/admin_staging.rs |
| `boot.route` | route déclarée de l'admin |  | 1 | t | staging/admin_staging.rs |
| `boot.no_guard` | la connexion de l'admin n'a ni LoginGuard ni limiteur … |  | 1 | t | staging/admin_staging.rs |
| `boot.options_without_effect` | options de liste de l'admin acceptées, mais affichées … |  | 1 | t | staging/admin_staging.rs |
| `boot.stylesheets_missing` | feuilles de style de Runique introuvables pour l'admin… |  | 1 | t | template/stylesheet.rs |
| `boot.debug_in_prod` | DEBUG=true en production : tous les visiteurs voient l… |  | 1 | t | builder/build.rs |
| `boot.prune_failed` | échec de l'élagage des droits orphelins : des droits p… |  | 1 | t | builder/build.rs |
| `server.ctrl_c_not_installed` | gestionnaire de Ctrl+C non installé |  | 1 | t | app/runique_app.rs |
| `server.sigterm_not_installed` | gestionnaire de SIGTERM non installé |  | 1 | t | app/runique_app.rs |

### `makemigrations` (23 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `project_pk_cargo` | Impossible de lire les features du projet avec cargo :… | 1 | 3 | tf | cli_migration/project_pk.rs |
| `project_pk_no_runique` | `runique` n'est pas une dépendance du projet lancé ici… |  | 2 | t | cli_migration/project_pk.rs |
| `destructive_detected` | Changements destructifs détectés : |  | 1 | autre | cli_migration/report.rs |
| `destructive_require_force` | Utilisez --force pour appliquer ces changements destru… |  | 1 | autre | cli_migration/report.rs |
| `long_identifier_detected` | Identifiants SQL générés trop longs (limite MySQL : 64… |  | 1 | autre | cli_migration/report.rs |
| `long_identifier_rename_required` | Raccourcissez les noms de table/colonne concernés — au… |  | 1 | autre | cli_migration/report.rs |
| `files_ready` | {} fichier(s) prêt(s) à migrer. | 1 | 1 | tf | cli_migration/mod.rs |
| `snapshots_upgraded` | {} snapshot(s) mis à jour pour enregistrer les longueu… | 1 | 1 | tf | cli_migration/mod.rs |
| `type_change_not_migrated` | ATTENTION : --force a laissé passer ces changements de… |  | 1 | t | cli_migration/report.rs |
| `type_change_write_by_hand` | Écrivez à la main le up() et le down() de cette migrat… |  | 1 | t | cli_migration/report.rs |
| `no_changes` | Aucun changement détecté : vos modèles correspondent à… |  | 1 | t | cli_migration/mod.rs |
| `apply_hint` | Appliquez les migrations en attente avec : runique mig… |  | 3 | t | cli_migration/init_migrate.rs, cli_migration/mod.rs |
| `database_unreachable` | Connexion à la base impossible : {} | 1 | 2 | tf | cli_migration/migrate.rs, cli_migration/mod.rs |
| `applied_unknown` | Ces migrations sont inscrites dans la base mais absent… | 1 | 1 | tf | cli_migration/applied.rs |
| `snapshot_unreadable` | Le snapshot {} est illisible ou n'est pas un snapshot.… | 1 | 2 | tf | cli_migration/snapshot.rs, cli_migration/tests.rs |
| `snapshot_misnamed` | Le snapshot {} décrit la table `{}` : son nom de fichi… | 2 | 2 | autre | cli_migration/snapshot.rs, cli_migration/tests.rs |
| `duplicate_table` | Deux tables portent le nom `{}`. La table d'un champ l… | 1 | 1 | tf | cli_migration/scan.rs |
| `waiting_removed` | {} migration(s) en attente retirée(s) : vos modèles co… | 1 | 1 | tf | cli_migration/mod.rs |
| `waiting_unchanged` | {} migration(s) en attente, inchangée(s) : rien n'a ét… | 1 | 1 | tf | cli_migration/mod.rs |
| `parallel_changes` | Deux migrations partent du même point pour la même tab… | 1 | 2 | autre, tf | cli_migration/plan.rs, cli_migration/tests.rs |
| `paused_tables` | Tables en pause (`managed: false`), ignorées par makem… | 1 | 1 | tf | cli_migration/mod.rs |
| `rename_with_new_key` | La table `{}` est renommée dans ce lancement et reçoit… | 2 | 1 | autre | changes/renames.rs |
| `renamed_twice` | Deux tables se disent renommées depuis `{}` : `{}` et … | 3 | 1 | tf | cli_migration/plan.rs |

### `middleware` (2 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `bad_request` | Mauvaise requête |  | 1 | t | security/allowed_hosts.rs |
| `slot_collision` | Collision de slot middleware Runique : « {} » et « {} … | 3 | 1 | autre | middleware_staging/applicator.rs |

### `migrate` (12 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `complete_up` | Migration terminé. |  | 1 | t | cli_migration/migrate.rs |
| `crate_missing` | Crate de migrations introuvable : '{}' ne contient pas… | 1 | 1 | tf | cli_migration/migrate.rs |
| `sqlite_key_change` | {} : cette migration ajoute ou retire une clé étrangèr… | 1 | 2 | tf | runtime/mod.rs, runtime/sqlite.rs |
| `engine_unsupported` | Moteur de base de données non pris en charge pour cett… | 1 | 1 | tf | runtime/mod.rs |
| `running` | Exécution de 'migrate {}' avec le crate de migrations … | 2 | 1 | tf | cli_migration/migrate.rs |
| `complete_down` | Retour arrière terminé. |  | 1 | t | cli_migration/migrate.rs |
| `cargo_missing` | Impossible de lancer cargo. Rust est-il installé et ca… |  | 1 | t | cli_migration/migrate.rs |
| `failed` | Le crate de migrations a échoué sur 'migrate {}' (code… | 2 | 1 | tf | cli_migration/migrate.rs |
| `crate_created` | Crate de migrations créé dans '{}'. | 1 | 1 | tf | cli_migration/init_migrate.rs |
| `interrupted` | Un makemigrations a été interrompu dans '{}'. Relancez… | 1 | 3 | tf | cli_migration/migrate.rs |
| `renamed_from_kept` | {} : les `renamed_from` appliqués n'ont pas pu être re… | 1 | 1 | tf | cli_migration/applied_renames.rs |
| `engine_defaulted` | Ni DATABASE_URL ni DB_ENGINE ne désignent un moteur co… | 1 | 1 | autre | cli_migration/init_migrate.rs |

### `reset` (18 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `forgot_title` | Mot de passe oublié |  | 1 | t | auth/password.rs |
| `reset_title` | Nouveau mot de passe |  | 6 | t | auth/password.rs |
| `success_title` | Mot de passe réinitialisé |  | 1 | t | auth/password.rs |
| `email_label` | Adresse email |  | 2 | t | auth/password.rs |
| `new_password_label` | Nouveau mot de passe |  | 1 | t | auth/password.rs |
| `confirm_label` | Confirmer le mot de passe |  | 1 | t | auth/password.rs |
| `email_mismatch` | L'adresse email ne correspond pas au lien. |  | 1 | t | auth/password.rs |
| `invalid_link` | Lien invalide. |  |  |  | **aucune référence** |
| `invalid_or_expired` | Lien de réinitialisation invalide ou expiré. |  | 4 | t | auth/password.rs |
| `password_min_length` | Le mot de passe doit contenir au moins {} caractères. | 1 | 1 | tf | auth/password.rs |
| `password_weak` | Le mot de passe doit contenir au moins une majuscule, … |  | 1 | t | auth/password.rs |
| `password_mismatch` | Les mots de passe ne correspondent pas. |  | 1 | t | auth/password.rs |
| `check_inbox` | Si cette adresse est associée à un compte, vous recevr… |  | 1 | t | auth/password.rs |
| `blocked_subject` | Votre compte est bloqué |  | 1 | t | auth/password.rs |
| `blocked_body` | <p>Bonjour {},</p><p>Une demande de réinitialisation d… | 1 | 1 | tf | auth/password.rs |
| `email_subject` | Réinitialisation de votre mot de passe |  | 1 | t | auth/password.rs |
| `email_body` | <p>Bonjour {},</p><p>Cliquez sur le lien suivant pour … | 4 | 1 | tf | auth/password.rs |
| `unavailable` | La réinitialisation par email n'est pas disponible pou… |  | 1 | t | auth/password.rs |

### `runique_test` (22 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `connect_failed` | connexion impossible : {} | 1 | 1 | autre | logic/builder_test.rs |
| `begin_failed` | impossible d'ouvrir la transaction de test : {} | 1 | 1 | autre | logic/builder_test.rs |
| `env_read_failed` | impossible de lire {} : {} | 2 | 1 | autre | logic/builder_test.rs |
| `env_parse_failed` | impossible d'analyser {} : {} | 2 | 1 | autre | logic/builder_test.rs |
| `env_line_malformed` | impossible d'analyser {} : une ligne est mal formée au… | 2 | 1 | autre | logic/builder_test.rs |
| `no_panic_message` | aucun message |  | 2 | autre | logic/builder_test.rs |
| `query_failed` | échec |  | 2 | autre | logic/struct_test.rs |
| `panicked` | panique : {} | 1 | 1 | autre | logic/struct_test.rs |
| `rollback_failed` | rollback : {} | 1 | 1 | autre | logic/struct_test.rs |
| `test_failed` | {} a échoué, voir la sortie ci-dessus | 1 | 1 | autre | logic/struct_test.rs |
| `clone_outlived` | un clone de la connexion de test a survécu au handler … |  | 1 | autre | logic/transaction_test.rs |
| `not_a_transaction` | le test ne s'est pas exécuté dans une transaction : ri… |  | 1 | autre | logic/transaction_test.rs |
| `nested` | runique_test ne peut pas tourner à l'intérieur d'un au… |  | 1 | autre | logic/builder_test.rs |
| `timed_out` | le handler n'a pas fini en {} s : un interblocage, ou … | 1 | 1 | autre | logic/struct_test.rs |
| `bad_timeout` | {} doit être un nombre entier de secondes supérieur à … | 2 | 1 | autre | logic/builder_test.rs |
| `transaction_ended` | la transaction du test a été terminée depuis le test l… |  | 2 | autre | logic/struct_test.rs |
| `swallowed_failures` | requêtes en échec : {}, et pourtant le handler a renvo… | 1 | 2 | autre | logic/struct_test.rs |
| `rollback_timed_out` | le rollback n'a pas fini en {} s | 1 | 1 | autre | logic/builder_test.rs |
| `expected` | attendu |  | 1 | autre | logic/struct_test.rs |
| `not_refused` | la base a accepté ce que le test s'attendait à la voir… |  | 1 | autre | logic/expect_test.rs |
| `no_db_config` | le fichier env ne définit ni DATABASE_URL ni DB_ENGINE… |  | 1 | autre | logic/transaction_test.rs |
| `in_memory` | en mémoire : chaque test part d'une base vide |  | 3 | autre | logic/transaction_test.rs |

### `server` (9 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `operational` | Runique est opérationnel |  | 2 | t | app/runique_app.rs |
| `launched` | Serveur lancé sur {} | 1 | 2 | tf | app/runique_app.rs |
| `connected_db` | Connecté à la base de données {} -> {} | 2 | 1 | tf | app/runique_app.rs |
| `connected_engine` | Connecté à la base de données {} | 1 | 1 | tf | app/runique_app.rs |
| `stop_hint` | ctrl + c pour arrêter |  | 1 | t | app/runique_app.rs |
| `shutting_down` | Arrêt du serveur Runique... |  | 1 | t | app/runique_app.rs |
| `acme_not_compiled` | ACME_ENABLED=true mais la feature `acme` n'est pas com… |  | 1 | t | app/runique_app.rs |
| `acme_add_feature` | Ajoute la feature `acme` à ton Cargo.toml. |  | 1 | t | app/runique_app.rs |
| `test_utils_feature_enabled` | Feature `test-utils` compilée dans ce build — assure-t… |  | 2 | t | app/runique_app.rs |

### `test_runner` (16 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `no_cargo_toml` | Pas de Cargo.toml ici : lancez `runique test` depuis l… |  | 1 | t | src/test_runner.rs |
| `test_utils_missing` | La feature `test-utils` n'est pas activée pour runique… |  | 1 | t | src/test_runner.rs |
| `test_utils_shipped` | Cargo.toml active `test-utils` en dehors de [dev-depen… |  | 1 | t | src/test_runner.rs |
| `no_mod_file` | Pas de src/runique_test/mod.rs : créez-le, puis déclar… |  | 1 | t | src/test_runner.rs |
| `module_not_declared` | src/main.rs ne déclare pas le module de test. Ajoutez … |  | 1 | t | src/test_runner.rs |
| `module_not_gated` | Le module runique_test n'est pas derrière `cfg(test)` … |  | 1 | t | src/test_runner.rs |
| `file_not_declared_warning` | attention : src/runique_test/{}.rs n'est pas déclaré d… | 2 | 1 | tf | src/test_runner.rs |
| `no_test_file` | Pas de src/runique_test/{}.rs. Fichiers de test : {} | 2 | 1 | autre | src/test_runner.rs |
| `file_not_declared` | src/runique_test/{}.rs n'est pas déclaré dans mod.rs :… | 2 | 1 | tf | src/test_runner.rs |
| `cargo_spawn_failed` | Impossible de lancer cargo. |  | 2 | t | src/test_runner.rs |
| `output_failed` | Impossible d'afficher la sortie de cargo. |  | 1 | t | src/test_runner.rs |
| `cargo_wait_failed` | cargo test ne s'est pas terminé correctement. |  | 1 | t | src/test_runner.rs |
| `tests_failed` | cargo test a échoué, voir la sortie ci-dessus. |  | 1 | t | src/test_runner.rs |
| `build_failed` | cargo n'a pas pu compiler les tests, voir les erreurs … |  | 1 | t | src/test_runner.rs |
| `no_such_test` | Aucun test "{}" dans {}.rs. Tests disponibles : {} | 3 | 1 | autre | src/test_runner.rs |
| `dir_read_failed` | Impossible de lire {} | 1 | 1 | tf | src/test_runner.rs |

### `title` (1 clés)

| Clé | Texte (fr) | `{}` | Appels | Comment | Où |
|---|---|---:|---:|---|---|
| `template` | Erreur de template dans '{}' | 1 | 1 | tf | errors/error.rs |

