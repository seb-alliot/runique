# Validation au boot

Plus de génération de fichiers admin. À la place, `AdminStaging::validate()` contrôle toute la config **avant** de monter les routes. Une erreur empêche le démarrage avec un message explicite.

## Quand ça tourne

1. `.with_admin(admin::config)` construit le staging
2. `build()` appelle `staging.validate()`
3. Si `Ok`, les routes CRUD sont montées sous `prefix + path`

Tu peux rejouer les mêmes contrôles en test :

```rust
#[test]
fn every_registered_model_passes_the_boot_checks() {
    let staging = super::config(AdminStaging::new().enable());
    if let Err(e) = staging.validate() {
        panic!("{e}");
    }
}
```

## Ce qui est contrôlé

### Par modèle

**Liste**

- `links` et `sortable` ne citent que des colonnes de `display` ;
- `search_help` sans recherche (`search([])`) est refusé ;
- une action groupée ne vise pas la clé primaire, sa valeur tient dans la colonne, et une même colonne n'a pas deux fois la même valeur.

**Formulaire**

- chaque champ de `fields` existe dans le formulaire du modèle (`ModelMeta::Form`) et peut porter le type de la colonne ;
- les champs cités ailleurs (libellés, lecture seule…) sont dans `fields` ;
- une colonne secrète obligatoire à la création (NOT NULL, sans défaut) est refusée : l'admin ne pourrait jamais la remplir, donnez-lui un défaut.

**Les deux**

- une colonne secrète (`password`…) n'apparaît dans aucune liste : affichage, recherche, filtre, tri, formulaire, action ;
- une action groupée vise une colonne que le formulaire écrit en édition (dans `fields`, ni `readonly` ni `create_only`) ;
- une action groupée ne vise jamais la colonne propriétaire : un transfert est une opération à part ;
- sur les comptes : pas d'action groupée sur les rangs ni sur `is_active`, et pas de drapeau de rang dans `fields`.

### Au niveau du site

- un modèle enregistré deux fois est refusé ;
- l'admin des comptes ne se remplace que par `accounts_users`, avec la table `eihwaz_users` (sa clé et ses protections) ;
- les groupes sont servis par le framework : les enregistrer est refusé.

### Au niveau du staging

- le chemin de l'admin n'est pas vide ;
- des routes admin déclarées dans `urlpatterns!` alors que l'admin est désactivé : refusé ;
- chaque route admin déclarée vise un modèle enregistré, a un chemin absolu (un seul segment `{id}`, et rien d'autre, pour remplacer le détail), ne chevauche ni une route publique ni l'espace de l'admin, n'est déclarée qu'une fois (chemin et nom), et une vue n'est remplacée qu'une fois.

### Ailleurs au démarrage

- les surcharges de templates (voir [Surcharge](/docs/fr/admin/template-surcharge)) ;
- en production, `with_public_url` est obligatoire quand l'admin est actif ;
- les droits qui visent un modèle non enregistré (ceux des groupes comme les grilles des admins) sont **supprimés définitivement** de la base (seulement si le projet a enregistré au moins un de ses modèles).

## Défauts appliqués

- sans `display` : toutes les colonnes sauf les secrètes ; sur les comptes, seulement `username`, `email`, `is_active` ;
- sans `.form()` : tous les champs du formulaire du modèle (`ModelMeta::Form`).

À l'exécution, chaque liste de colonnes est ensuite réduite à ce que les droits de l'utilisateur laissent voir ou écrire.

## Fichiers concernés

| Fichier | Rôle |
| --- | --- |
| `src/admin.rs` | Ta config (à maintenir) |
| `runique/src/admin/builder/` | Builder + validation |
| `runique/src/app/staging/admin_staging.rs` | Staging + `validate()` |

Rien n'est écrit sous `src/admins/` : ce dossier n'existe plus.

## Voir aussi

| Section | Description |
| --- | --- |
| [Builder ModelAdmin](/docs/fr/admin/declaration-macro) | API qui alimente la validation |
| [Mise en place](/docs/fr/admin/setup) | Câblage |
| [Permissions](/docs/fr/admin/permission) | Gate runtime |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
