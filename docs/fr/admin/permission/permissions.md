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

## Ressources `groupes` et `groupes_droits`

Ces deux ressources ne bénéficient d'**aucun traitement spécial** : elles suivent exactement la même logique générique que n'importe quelle ressource (`can_read`/`can_create`/`can_update`/`can_delete` via `eihwaz_groupes_droits`). Un superuser peut donc délibérément accorder `can_update` sur `"groupes"` à un groupe "Admin", qui pourra alors gérer les permissions d'autres groupes (par exemple pour déléguer à des modérateurs) — c'est un usage légitime, pas un contournement.

**Ce que ça implique en pratique :** accorder des droits sur `groupes`/`groupes_droits` à un groupe non-superuser lui donne un contrôle réel sur les permissions — y compris, potentiellement, les siennes propres si son utilisateur appartient aussi à ce groupe. Il n'existe pas de garde-fou framework contre l'auto-élévation dans ce cas précis : c'est à qui configure les groupes de ne pas accorder ces droits à un rôle qui ne doit pas les avoir. Seul `is_superuser` reste un bypass total et non délégable (bit sur l'utilisateur, jamais accordé via un groupe).

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
