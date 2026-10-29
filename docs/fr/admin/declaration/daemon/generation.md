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

### Par modèle (`ModelAdmin::validate`)

- cohérence liste / formulaire (action sur colonne non writable, etc.)
- secrets jamais exposés (display, search, filter, sort, form, action)
- sur la table comptes : pas d'action groupée sur les rangs / `is_active`
- rangs absents de `fields` (réservés au gate)
- owner jamais cible d'une action de transfert déguisée

### Au niveau site (`AdminSite::check`)

- chaque modèle enregistré passe ses contrôles
- l'admin comptes framework n'est pas « doublé » par un `register` pirate
- groupes et comptes restent cohérents

### Au niveau staging

- path admin valide
- pas de collision de routes déclarées
- rate limiter / login guard cohérents si fournis

## Défauts appliqués avant service

- sans `display` : colonnes non secrètes (comptes → username, email, is_active)
- sans `links` : première colonne de display
- sans `form.fields` : tous les champs du formulaire modèle, filtrés par le gate

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
