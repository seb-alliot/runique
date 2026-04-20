# Runique — Roadmap

## Stabilisation des features existantes

### Admin — UX & Fonctionnel

- [ ] **Recherche admin**
  - Debounce 300-500ms (ne pas chercher à chaque lettre)
  - WHERE sur toutes les colonnes du modèle si pas de `list_display`, sinon WHERE sur les colonnes de `list_display`
  - Bug actuel : fallback sur `id` uniquement quand `list_display` vide

- [ ] **Responsive admin**
  - Plusieurs éléments visuels à corriger sur mobile/tablette

- [ ] **Historique admin**
  - Log des actions CRUD par utilisateur : qui a modifié quoi et quand

- [ ] **Persistance des filtres admin**
  - Conserver `search`, `filter_*`, `page`, `sort_by` dans l'URL de retour après edit/delete
  - Comportement actuel : tout se réinitialise après une redirection

### Framework

- [ ] **Boot validation**
  - Refuser le démarrage en production si la configuration est incohérente ou incomplète

- [ ] **Coverage tests**
  - Réévaluer le taux actuel (beaucoup de features ajoutées depuis le dernier audit)
  - Objectif : 85% minimum sur les lignes

- [ ] **Pagination changelog** — la liste des entrées changelog dans demo-app n'a pas de pagination

## Features futures

- [ ] **Surcharge des champs `#[form]`** — `field_override("name", |f| f.label("...").placeholder("..."))` pour configurer label/placeholder sur les champs générés par la proc-macro

- [ ] **Hooks / Signals** — fichier `hooks.rs` déclaratif branché sur `ActiveModelBehavior` SeaORM
