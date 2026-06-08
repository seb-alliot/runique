# Rôles et permissions

## Vue d'ensemble

Le système de permissions admin repose sur trois niveaux :

| Niveau | Contrôle | Effet |
| --- | --- | --- |
| **`is_staff`** | Champ utilisateur | Donne accès à la page de login admin uniquement |
| **Groupes** | Tables `eihwaz_groupes` + `eihwaz_groupes_droits` | Permissions CRUD granulaires par ressource |
| **`is_superuser`** | Champ utilisateur | Bypass tous les contrôles |

---

## Champs de contrôle d'accès

| Champ | Type | Rôle |
| --- | --- | --- |
| `is_staff` | `bool` | Autorise la connexion à la page `/admin/login` |
| `is_superuser` | `bool` | Accès total, bypass toutes les vérifications |
| `is_active` | `bool` | Bloque les comptes inactifs |

---

## Permissions granulaires par groupe

Les permissions sont portées par les **groupes**, pas directement par les utilisateurs. Un utilisateur hérite des permissions de tous ses groupes (agrégation par OR logique).

Chaque groupe dispose d'une entrée par ressource dans `eihwaz_groupes_droits` :

| Champ | Effet |
| --- | --- |
| `can_read` | Voir la ressource dans la nav + accéder à la liste |
| `can_create` | Créer un enregistrement |
| `can_update` | Modifier n'importe quel enregistrement |
| `can_delete` | Supprimer n'importe quel enregistrement |
| `can_update_own` | Modifier uniquement ses propres enregistrements |
| `can_delete_own` | Supprimer uniquement ses propres enregistrements |

---

## Configuration via le panel

1. Aller dans **Admin → Groupes** → créer un groupe
2. Configurer les permissions de ce groupe par ressource
3. Aller dans **Admin → Utilisateurs** → assigner le groupe à l'utilisateur

Un utilisateur sans groupe ne voit aucune ressource dans la nav (sauf superuser).

### Révocation immédiate

Retirer un groupe d'un utilisateur prend effet à sa prochaine requête. Supprimer un groupe vide le cache de permissions de tous ses membres instantanément.

---

## Ressources superuser uniquement

Les ressources `groupes` et `groupes_droits` ne peuvent être accédées que par un `is_superuser`. Aucun groupe ne peut débloquer leur accès pour un staff — règle fixe du framework.

Cela empêche l'escalade de privilèges : un staff ne peut jamais modifier ses propres permissions.

---

## Logique d'accès

```text
authentifié ?
  └─ non → redirection /admin/login
  └─ oui → is_staff OU is_superuser ?
               └─ aucun → redirection /admin/login
               └─ is_superuser → AUTORISÉ (accès total, toutes ressources)
               └─ is_staff → can_read sur la ressource ?
                                └─ non  → ressource absente de la nav
                                └─ oui  → liste visible
                                          can_create / can_update / can_delete
                                          pour les opérations correspondantes
                                          can_update_own / can_delete_own
                                          pour ses propres enregistrements uniquement
```

---

## Notes

- La macro `admin!` ne déclare plus de `permissions:` — la configuration est entièrement en base.
- Les permissions sont agrégées par OR logique sur tous les groupes de l'utilisateur.
- Un utilisateur peut avoir `can_read` sans `can_create` — il voit la liste mais ne peut pas créer.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin, créer un superuser |
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution |

## Retour au sommaire

- [Sommaire Admin](/docs/fr/admin)
