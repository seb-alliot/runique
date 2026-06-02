# Runique — Roadmap

## Légende

- ✅ Implémenté
- 🔧 En cours / infrastructure posée
- [ ] À faire

---

## Stabilisation des features existantes

- ✅ **Recherche admin** — debounce + WHERE sur colonnes `list_display` ou toutes colonnes
- ✅ **Responsive admin** — layout mobile/tablette
- ✅ **Tracing structuré** — `RuniqueLog` par domaine (forms, admin, auth, mailer, builder)
- ✅ **`extend!{}` génère le code Rust** — entité complète + ActiveModel + AdminForm
- [ ] **Historique admin** — filtres par resource/action/user + diff avant/après + timeline
- [ ] **Persistance des filtres admin** — conserver `search`, `filter_*`, `page`, `sort_by` dans l'URL de retour après edit/delete
- [ ] **Boot validation** — refuser le démarrage en production si la config est incohérente
- [ ] **Reset token persisté en DB** — actuellement en mémoire, perdu au redémarrage
- [ ] **Pagination changelog** — demo-app : liste des entrées sans pagination

---

## Panel Admin

### Affichage liste

- [ ] **Filtres cumulables** — cliquer un filtre réinitialise les autres colonnes ; le backend supporte déjà plusieurs filtres simultanés (`Vec`), c'est le template qui ne préserve pas les autres
- [ ] **Filtres par date/temps** — `list_filter` sur colonnes `timestamp` avec plages prédéfinies (aujourd'hui, cette semaine, ce mois, cette année)
- [ ] **Filtres FK traversal** — `list_filter` sur colonnes de tables liées (`article__auteur__nom`)
- [ ] **`date_hierarchy`** — navigation drill-down par date (année > mois > jour) dans la liste
- [ ] **`list_display_links`** — choisir quelle(s) colonne(s) sont des liens vers le détail (actuellement toujours la première)
- [ ] **`list_editable`** — éditer des champs directement dans la liste sans ouvrir le formulaire
- [ ] **`ordering` par ressource** — tri par défaut configurable dans le DSL `admin!{}`, indépendant du `meta.ordering` du modèle
- [ ] **`empty_value_display`** — valeur affichée pour les champs null/vides dans la liste

### Formulaires create/edit

- [ ] **`fieldsets`** — grouper les champs en sections avec titre dans le formulaire
- [ ] **`readonly_fields`** — champs affichables mais non éditables
- [ ] **`prepopulated_fields`** — auto-remplissage JS d'un champ depuis un autre (ex : slug depuis titre)
- [ ] **`autocomplete_fields`** — widget FK/M2M avec recherche textuelle (requiert `search_fields` sur la ressource liée)
- [ ] **`save_as`** — bouton "Enregistrer comme nouveau" pour dupliquer un objet
- [ ] **Vue publique depuis l'admin** — lien depuis le formulaire admin vers la vue publique de l'objet (`view_on_site` Django)
- [ ] **Redirection post-save configurable** — actuellement fixe vers la liste ; permettre de rester sur le formulaire ou d'en ouvrir un nouveau

### Actions de groupe

- [ ] **Actions custom avec logique Rust** — `GroupAction::custom(label, handler_fn)` où `handler_fn` reçoit les IDs + l'engine ; actuellement limité à `UPDATE SET field = value`
- [ ] **`action_form`** — formulaire supplémentaire affiché quand une action bulk est sélectionnée (ex : choisir un statut avant d'appliquer)

### Filtres et recherche

- [ ] **`SimpleListFilter`** — filtre custom avec logique arbitraire (queryset, labels, choix dynamiques) ; actuellement uniquement `filter_fn` bas niveau
- [ ] **Recherche multi-table** — `search_fields` avec traversée FK (`client.nom`) via JOIN

### Hooks et customisation

- [ ] **`get_queryset`** — personnaliser le queryset de base de la liste par ressource
- [ ] **Hooks dynamiques** — `list_display`, `list_filter`, `fields`, `readonly_fields` configurables par requête
- [ ] **Assets CSS/JS par ressource** — injecter des assets spécifiques à une ressource (équivalent `class Media` Django)

### Inlines

- [ ] **`TabularInline` / `StackedInline`** — éditer les objets liés (FK vers parent) directement dans le formulaire du parent
- [ ] **Inlines M2M** — gérer une relation M2M via inline sans passer par la table de jonction

### Divers admin

- [ ] **Export** — CSV/JSON natif depuis la liste
- [ ] **Boutons save avancés** — "Enregistrer et continuer" / "Enregistrer et ajouter un autre"
- [ ] **Dialog de confirmation** — remplacer submit direct des boutons delete/bulk-delete par un `<dialog>` natif
- [ ] **Multiple sites admin** — plusieurs instances admin indépendantes sur des préfixes différents

---

## ORM & Modèles

- 🔧 **Hooks / Signals** — `before_save`, `after_save`, `before_delete`, `after_delete` via `SignalBuilder` ; infrastructure `HooksDef` posée dans `migration/hooks/`, générateur à brancher dans `derive_form`
- [ ] **`makemigrations` — détecter les suppressions** — générer `DROP COLUMN` quand une colonne disparaît du DSL (actuellement silencieux)
- [ ] **`search!` — agrégats** — `.avg()`, `.sum()`, `.count_by()` sur `RuniqueQueryBuilder` (actuellement SQL brut requis)
- [ ] **`search!` — `.first()` simplifié** — retourner `Option<T>` au lieu de `Result<Option<T>>`, cohérent avec `.all()` et `.count()`
- [ ] **`search!` — filtres conditionnels** — bras `?Col in (opt_vec)` qui saute si `None`/vide

---

## Framework Core

- [ ] **Surcharge champs `#[form]`** — `overrides = [title(label = "...", placeholder = "...")]` par formulaire sans modifier le schéma
- [ ] **TypeMap — `with_extension`** — `N` connexions de types différents simultanément (Redis, MongoDB, reqwest) ; actuellement une seule via `with_custom_db`
- [ ] **`crud!{}` — vues CRUD publiques génériques** — équivalent des `ListView`, `DetailView`, `CreateView` Django pour les vues publiques ; l'admin!{} prouve que le pattern est viable
- [ ] **Modificateurs inline dans `urlpatterns!{}`** — `protect = "login_required"`, `rate_limit = (5, 60)` par route directement dans `url.rs`
- [ ] **Middleware i18n auto** — détection langue (user DB → cookie → `Accept-Language`) au slot 57, `{{ lang }}` injecté dans chaque contexte Tera, route `/_runique/set-lang`
- [ ] **`request.path_params` / `query_params` encapsulés** — rendre privés, exposer uniquement des getters (v2.2+)
- [ ] **Validation séquentielle S1/S2/S3** — typestate form : CSRF → validation → cleaned_data ; CSRF mandatoire par construction (v3.0, breaking)
- [ ] **Guard mot de passe en clair** — `tracing::warn!` si `cleaned_string("password")` retourne du plaintext non hashé après `finalize()`

---

## Authentification & Sécurité

- [ ] **OAuth / OIDC** — flow complet (Google, Microsoft, Apple, LDAP, SAML) ; stub `PasswordConfig::Delegated` existe mais non implémenté
- [ ] **JWT / API key auth** — pour les clients API/mobile
- [ ] **CSP `report-uri` / `report-to`** — collecter les violations CSP en production
- [ ] **Audit log authentification** — table `eihwaz_auth_log` : connexions réussies/échouées/lockouts tracées en DB

---

## Déploiement & Infrastructure

- [ ] **Sitemap auto-généré** — middleware lit `url_registry` + `allowed_hosts`, génère `/sitemap.xml` ; routes dynamiques incluses via callback DB optionnel
- [ ] **Redimensionnement d'images** — resize/crop côté serveur (actuellement non natif)

---

## DX (Developer Experience)

- [ ] **Test client intégré** — client HTTP natif pour les tests de handlers (actuellement : `reqwest` ou `axum::test`)
- [ ] **Fixtures** — `loaddata`/`dumpdata` ; actuellement les seeds sont des fonctions Rust
- [ ] **Management commands** — équivalent `manage.py custom_command` ; actuellement `src/bin/` uniquement
- [ ] **Documentation API publique** — couverture docs.rs avec `///` + exemples `# Examples` sur toutes les fonctions publiques
- [ ] **i18n complète** — pluralisation + traduction des templates Tera (actuellement `t()`/`tf()` uniquement dans le code Rust/contexte)
- [ ] **Sitemap / RSS** — non natif
