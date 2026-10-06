# Modèle utilisateur

## Modèle built-in

Runique inclut un modèle utilisateur prêt à l'emploi, sans aucune configuration.

**Table générée :** `eihwaz_users`

| Champ | Type | Description |
|---------------|----------|----------------------------------------|
| `id` | `Pk` | Clé primaire (`i32` par défaut, `i64` avec `big-pk`, `Uuid` avec `pk-uuid`) |
| `username` | `String` | Nom d'utilisateur unique |
| `email` | `String` | Adresse email |
| `password` | `String` | Hash Argon2 — jamais en clair |
| `is_active` | `bool` | Compte actif |
| `is_staff` | `bool` | Accès au panneau admin (limité) |
| `is_superuser` | `bool` | Accès complet, bypass toutes les règles |
| `created_at` | datetime | Date de création |
| `updated_at` | datetime | Date de mise à jour |
| `activated_at` | datetime | Activation par le propriétaire (vide tant que le compte est en attente) |

Pour créer le premier superutilisateur :

```bash
runique create-superuser
```

### Clé primaire i64 ou UUID

Par défaut, la clé primaire est un `i32`. Pour passer à `i64` ou `Uuid` — **une seule feature à
la fois**, activer les deux ensemble est une erreur de compilation volontaire :

```toml
# Cargo.toml du projet
runique = { version = "3.0.0", features = ["postgres", "big-pk"] }    # Pk = i64
runique = { version = "3.0.0", features = ["postgres", "pk-uuid"] }   # Pk = Uuid (Uuid::now_v7())
```

Le choix doit être fait avant la première migration — voir
[DSL `model!` & `extend!`](/docs/fr/model/dsl) pour le détail complet (types compatibles,
contraintes FK, changement de mode après coup).

---

## États d'un compte

`eihwaz_users` est le seul modèle utilisateur : pour y ajouter des champs, utiliser
`extend!{ table: "eihwaz_users", ... }`.

Deux colonnes décrivent l'état d'un compte : `is_active` (est-il actif ?) et `activated_at`
(quand son propriétaire l'a-t-il activé, via le lien reçu par email ?).

| `is_active` | `activated_at` | État | Connexion | « Mot de passe oublié » |
| --- | --- | --- | --- | --- |
| `false` | vide | **En attente** — créé, jamais activé | Refusée | Lien qui **active** le compte |
| `true` | rempli | **Actif** | Autorisée | Lien de réinitialisation |
| `false` | rempli | **Désactivé** par le staff | Refusée | Email « compte bloqué », sans lien |
| `true` | vide | Impossible — refusé par la base | — | — |

- La **première activation** appartient au propriétaire de l'email : un admin ne peut pas cocher
  `is_active` sur un compte en attente.
- **Désactiver et réactiver** un compte déjà activé est le rôle du staff (`can_update` sur `users`).
- Changer de mot de passe **ne réactive jamais** un compte désactivé.
- La page « mot de passe oublié » répond la même chose quel que soit l'état ou l'existence du compte.
- Pour annuler une invitation (compte en attente), supprimer le compte.

La garantie « pas de `is_active` sans `activated_at` » est posée **en base** : une contrainte
`CHECK` créée avec la table `eihwaz_users`, sur les trois moteurs. Elle s'applique aussi au SQL brut.

Côté code, le modèle implémente le trait `RuniqueUser` :

| Méthode | Description |
| --- | --- |
| `can_sign_in()` | `is_active` **et** `activated_at` rempli |
| `can_access_admin()` | `can_sign_in()` et (`is_staff` ou `is_superuser`) |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Helpers de session](/docs/fr/auth/session) | `login`, `auth_login`, `logout` |
| [Middlewares & CurrentUser](/docs/fr/auth/middleware) | Protection des routes |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
