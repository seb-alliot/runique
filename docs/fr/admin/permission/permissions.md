# Rôles et permissions

## Vue d'ensemble

Le système de permissions admin repose sur deux niveaux distincts :

| Niveau | Contrôle | Effet |
| --- | --- | --- |
| **Permission (view)** | Droit scopé `access_type = "view"` | Ressource visible dans la nav |
| **Droit (write)** | Droit scopé `access_type = "write"` | Accès create / edit / delete |
| **Superuser** | `is_superuser = true` | Bypass les deux niveaux |

---

## Champs de contrôle d'accès

| Champ | Type | Rôle |
| --- | --- | --- |
| `is_staff` | `bool` | Autorise l'accès à l'interface admin |
| `is_superuser` | `bool` | Accès total, bypass toutes les vérifications |
| `is_active` | `bool` | Bloque les comptes inactifs |

---

## Droits scopés

Chaque ressource enregistrée dispose automatiquement de deux droits dans `eihwaz_droits`, créés au démarrage par le framework si absents :

| Nom | `resource_key` | `access_type` | Effet |
| --- | --- | --- | --- |
| `blog.view` | `"blog"` | `"view"` | Voir la ressource blog dans la nav |
| `blog.write` | `"blog"` | `"write"` | Créer / modifier / supprimer dans blog |

Ces droits sont des entrées ordinaires dans `eihwaz_droits` — l'admin les assigne aux utilisateurs ou aux groupes depuis le panel, exactement comme n'importe quel autre droit.

---

## Configuration via le panel

### Accorder la visibilité d'une ressource

1. Aller dans **Admin → Droits**
2. Trouver `blog.view` (créé automatiquement au démarrage)
3. Aller dans **Admin → Utilisateurs**, ouvrir le profil du staff
4. Assigner le droit `blog.view`

L'utilisateur verra désormais la ressource `blog` dans la navigation admin.

### Accorder l'accès en écriture

Même procédure avec `blog.write`. Un utilisateur peut avoir `blog.view` sans `blog.write` — il voit la liste mais ne peut pas créer/modifier/supprimer.

### Révocation immédiate

Retirer un droit d'un utilisateur prend effet à sa prochaine requête — aucune déconnexion requise. Supprimer un droit de `eihwaz_droits` vide le cache de permissions de **tous** les utilisateurs instantanément.

---

## Ressources superuser uniquement

Les ressources `droits` et `groupes` ne peuvent être accédées que par un `is_superuser`. Aucun droit scopé ne peut débloquer leur accès pour un staff — c'est une règle fixe du framework.

Cela empêche l'escalade de privilèges : un staff ne peut jamais modifier ses propres droits ou ceux d'autres utilisateurs.

---

## Logique d'accès (état actuel)

```text
authentifié ?
  └─ non → redirection /admin/login
  └─ oui → is_staff OU is_superuser ?
               └─ aucun → redirection /admin/login
               └─ is_superuser → AUTORISÉ (accès total, toutes ressources)
               └─ is_staff → ressource visible si droit .view assigné
                              opération write si droit .write assigné
                              droits/groupes → refusé (superuser uniquement)
```

---

## Notes

- La macro `admin!` ne déclare plus de `permissions:` — la configuration est entièrement en base.
- Les droits scopés sont créés automatiquement : le dev n'a rien à faire côté code.
- Un utilisateur sans aucun droit scopé ne voit aucune ressource dans la nav (sauf superuser).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin, créer un superuser |
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
