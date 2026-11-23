# Validation au boot

L'admin ne génère plus aucun fichier. Ce que tu déclares dans `src/admin.rs` est contrôlé par `AdminStaging::validate()` avant le montage des routes. S'il y a une erreur, l'application ne démarre pas et le message dit ce qui ne va pas.

## Quand ça tourne

`.with_admin(admin::config)` construit le staging, puis `build()` appelle `staging.validate()`. Si le résultat est `Ok`, les routes CRUD sont montées sous `prefix + path`.

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

### Sur chaque modèle

Pour la liste :

- `links` et `sortable` ne citent que des colonnes de `display`
- `search_help` sans recherche (`search([])`) est refusé
- une action groupée ne vise pas la clé primaire, sa valeur tient dans la colonne, et une même colonne n'a pas deux fois la même valeur

Pour le formulaire :

- chaque champ de `fields` existe dans le formulaire du modèle (`ModelMeta::Form`) et peut porter le type de la colonne
- les champs cités ailleurs (libellés, lecture seule…) sont dans `fields`
- une colonne secrète obligatoire à la création (NOT NULL, sans défaut) est refusée : l'admin ne pourrait jamais la remplir, donne-lui un défaut

Et pour les deux à la fois :

- une colonne secrète (`password`…) n'apparaît nulle part : ni affichage, ni recherche, ni filtre, ni tri, ni formulaire, ni action
- une action groupée vise une colonne que le formulaire écrit en édition, donc présente dans `fields` et ni `readonly` ni `create_only`
- une action groupée ne vise jamais la colonne propriétaire, un transfert est une opération à part
- sur les comptes, pas d'action groupée sur les rangs ni sur `is_active`, et pas de drapeau de rang dans `fields`

### Sur le site

Un modèle enregistré deux fois est refusé.

L'admin des comptes ne se remplace que par `accounts_users`, avec la table `eihwaz_users`, sa clé et ses protections.

Les groupes sont servis par le framework. Les enregistrer toi-même est refusé.

### Sur le staging

Le chemin de l'admin ne peut pas être vide, et tu ne peux pas déclarer de routes admin dans `urlpatterns!` si l'admin est désactivé.

Chaque route admin déclarée doit :

- viser un modèle enregistré
- avoir un chemin absolu, avec un seul segment `{id}` et rien d'autre si elle remplace le détail
- ne chevaucher ni une route publique ni l'espace de l'admin
- n'être déclarée qu'une fois, chemin comme nom

Une vue ne se remplace qu'une fois.

### Ailleurs au démarrage

Les surcharges de templates sont vérifiées aussi, voir [Surcharge](/docs/fr/admin/template-surcharge).

En production, `with_public_url` est obligatoire quand l'admin est actif.

Attention aux droits : ceux qui visent un modèle non enregistré sont supprimés définitivement de la base, que ce soit les droits des groupes ou les grilles des admins. Ça ne se produit que si le projet a enregistré au moins un de ses modèles.

## Défauts appliqués

Sans `display`, la liste montre toutes les colonnes sauf les secrètes. Sur les comptes, elle se limite à `username`, `email` et `is_active`.

Sans `.form()`, le formulaire prend tous les champs du formulaire du modèle (`ModelMeta::Form`).

À l'exécution, chaque liste de colonnes est ensuite réduite à ce que les droits de l'utilisateur laissent voir ou écrire.

## Fichiers concernés

Ta config est dans `src/admin.rs`, c'est le seul fichier à maintenir. Côté framework, le builder et sa validation sont dans `runique/src/admin/builder/`, le staging et `validate()` dans `runique/src/app/staging/admin_staging.rs`.

Rien n'est écrit sous `src/admins/` : ce dossier n'existe plus.

## Pages liées

L'API qui alimente ces contrôles est décrite dans [Builder ModelAdmin](/docs/fr/admin/declaration-macro), le câblage dans [Mise en place](/docs/fr/admin/setup) et les droits à l'exécution dans [Permissions](/docs/fr/admin/permission).

Retour au [sommaire Admin](/docs/fr/admin).
