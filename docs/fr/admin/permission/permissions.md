# Rôles et permissions

## Vue d'ensemble

L'accès à l'admin repose sur le **rang** du compte, trois drapeaux de `eihwaz_users` dont au plus un est vrai (contrainte `CHECK` en base) :

| Rang | Drapeau | Ce qu'il peut faire |
| --- | --- | --- |
| **Superuser** | `is_superuser` | Tout, seul à contourner les droits (jamais les invariants). Compte du développeur, à supprimer à la livraison |
| **Dirigeant** | `is_dirigeant` | Tous les droits sur les modèles, sans contournement. Donne les droits des admins |
| **Admin** | `is_admin` | Ce que le dirigeant lui coche **sur son compte**, table par table et colonne par colonne |
| **Lambda** | aucun | Compte du site. N'entre dans l'admin que s'il est membre d'un groupe staff |

`can_access_admin()` répond vrai pour un rang admin ou supérieur, ou pour un lambda membre d'un groupe `is_staff`. Un compte doit aussi être actif et activé.

---

## D'où viennent les droits

`CurrentUser::rights_on("ressource")` est la seule façon de demander ce qu'un compte peut faire, dans l'admin comme dans vos vues. La source dépend du rang seul :

| Rang | Source des droits |
| --- | --- |
| Superuser | Tout, avec contournement |
| Dirigeant | Tout, sans contournement |
| Admin | Sa propre grille, posée par le dirigeant (`eihwaz_groupes_droits`, lignes rattachées à son compte) |
| Lambda | L'union (OU logique) des droits de ses groupes — **jamais** sur les tables du framework (comptes, groupes, droits, appartenances) |

Les droits se règlent par table et par colonne :

| Droit | Niveau | Effet |
| --- | --- | --- |
| `can_read` | table + colonnes | Voir la ressource et les colonnes cochées |
| `can_create` | table + colonnes | Créer, en remplissant les colonnes cochées |
| `can_update` | table + colonnes | Modifier les colonnes cochées |
| `can_delete` | table | Supprimer |
| `can_update_own` / `can_delete_own` | table | Seulement ses propres lignes, quand le modèle déclare un propriétaire (`meta: { owner: … }`) |

Une colonne ajoutée plus tard au modèle n'est lisible ou modifiable qu'une fois cochée. Les colonnes secrètes (mot de passe) ne sont jamais proposées.

Les droits ne se déclarent **pas** dans `ModelAdmin` : toute la grille vit en base.

---

## Groupes et staff

Les groupes servent aux **comptes lambda** : rôles du site public, et droits sur les tables **du développeur** dans l'admin. Un lambda membre d'un groupe marqué `is_staff` entre dans l'admin et y voit ses ressources autorisées, ainsi qu'un résumé de ses propres groupes et droits, en lecture seule. Aucun groupe n'a jamais de droit sur les tables du framework, quel que soit celui qui le configure.

### Révocation immédiate

Le compte (actif, rang), ses groupes et ses droits sont relus en base à chaque requête admin : la session ne garde que l'identité de l'utilisateur. Retirer un droit ou un groupe, désactiver un compte ou baisser son rang prend donc effet à sa requête suivante, quel que soit le chemin de la modification (admin, CLI, SQL, autre instance). Un compte supprimé ou désactivé voit sa session fermée. Un compte qui perd seulement l'accès à l'admin (rang baissé, groupe staff retiré) garde sa session sur le site : l'admin le renvoie vers `/`.

---

## Tables du framework : `users`, `groupes`, `groupes_droits`

Les comptes, les groupes, les droits et les appartenances sont réservés aux **rangs** : le superuser, le dirigeant, et un admin seulement pour les tables et colonnes que le dirigeant lui a cochées. Aucun groupe ne peut recevoir de droit sur ces tables, et personne ne peut le lui accorder, superuser compris : un lambda, même staff, n'y fait aucune opération.

Seule exception : un lambda qui lit une ligne liée à un compte (par une clé étrangère) voit le `username` de ce compte, en lecture seule.

---

## Logique d'accès

```text
connecté ?
  └─ non → page de connexion de l'admin
  └─ oui → compte encore présent, actif et activé ?
             └─ non → session fermée, page de connexion
             └─ oui → can_access_admin() ? (rang ≥ admin, ou membre d'un groupe staff)
               └─ non → retour au site (/) avec un message ; la session reste ouverte
               └─ oui → rights_on(ressource)
                          superuser → tout, contournement compris
                          dirigeant → tout
                          admin     → sa grille (table × colonne)
                          lambda    → union de ses groupes, jamais une table du framework
                        └─ can_read ? non → ressource absente de la nav
                                      oui → ressource dans la nav
                                            au moins une colonne lisible ?
                                              non → liste refusée (rien à montrer)
                                              oui → liste, colonnes cochées seulement
                                                    can_create / can_update / can_delete
                                                    (_own : ses propres lignes)
```

---

## Drapeaux de rang

`is_superuser`, `is_dirigeant` et `is_admin` ne sont pas des champs du formulaire admin. Le boot refuse de les déclarer dans `fields`. Le rang se change depuis la fiche du compte : un dirigeant peut donner admin ou lambda, un superuser peut donner dirigeant, admin ou lambda. Le rang donné reste strictement sous celui qui l'écrit. Cette action reprend les groupes en cas de perte de rang, retire la grille de droits d'un admin qui redescend, et garde au moins un dirigeant actif tant qu'il reste un admin. `is_superuser` ne se donne pas par là : il se pose avec `runique create-superuser`, par un insert direct.

Le filtre de rang limite les lignes qu'une preuve peut toucher, selon le rang déjà enregistré : un admin ne touche ni un admin, ni un dirigeant, ni un superuser ; un dirigeant ne touche ni un dirigeant ni un superuser ; un superuser ne touche pas un autre superuser.

`write::insert` refuse un `Set(true)` sur un de ces drapeaux. `write::update` refuse tout `Set`, `true` ou `false` : une promotion ou une rétrogradation qui passerait par là sauterait les règles ci-dessus. Un `insert` ou un `update` SeaORM appelé directement sur la connexion ne passe pas par le gate.

## Notes

- `ModelAdmin` ne déclare pas de permissions : la configuration est entièrement en base.
- Pour un lambda, les droits sont l'union (OU logique) de ses groupes, recalculée à chaque requête.
- Un utilisateur peut avoir `can_read` sans `can_create` — il voit la liste mais ne peut pas créer.
- Cocher `can_read` sur une table **sans cocher aucune colonne** rend la liste inaccessible : il n'y a rien à montrer (une page de clés nues), donc elle est refusée. La ressource apparaît quand même dans le menu. Il faut cocher au moins une colonne en lecture.
- Les colonnes secrètes et les rangs restent hors formulaire, y compris pour un superuser (refus au boot). Le détail des drapeaux est au-dessus.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin, créer un superuser |
| [Déclaration](/docs/fr/admin/declaration) | `AdminStaging`, `AdminSite`, enregistrement |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution |

## Retour au sommaire

- [Sommaire Admin](/docs/fr/admin)
