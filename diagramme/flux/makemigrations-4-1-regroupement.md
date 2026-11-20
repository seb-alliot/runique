# makemigrations 4.1 : regroupement des séquences et contrôle des cas

Document de travail, 2026-11-20. Il regroupe les séquences par sujet, puis reprend chaque
endroit où le code devra choisir entre plusieurs cas (un `match`), pour vérifier que chaque
combinaison a une issue décidée.

Séquences d'ensemble : [makemigrations-4-1-chantier.md](makemigrations-4-1-chantier.md).
Séquences par cas : [makemigrations-4-1-cas.md](makemigrations-4-1-cas.md).

## 1. Les séquences, regroupées

| Groupe | Séquence d'ensemble | Séquences par cas |
|---|---|---|
| A. Avant de calculer : verrou entre commandes, connexion, position, contrôles | 1, 2 | 0.1, 0.2, 0.3, 0.6, 0.7 |
| B. De quel état on part | 1, 3 | 0.4, 2.5 |
| C. Ce que `makemigrations` compare | 1 | 1.1 à 1.8, 1.10, 1.11 |
| D. Ce qui bloque avant d'écrire | 1 | 1.9 |
| E. Ce qui est écrit | 1 | 2.1, 2.2, 2.3, 2.4 |
| F. Appliquer, par moteur | 2 | 0.5, 3a.1, 3a.2, 3a.3, 3a.4, 3a.5 |
| G. Revenir en arrière | 4 | 3a.6 |
| H. Reprendre une base existante (hors 4.1) | 5 | 3b.1 à 3b.5 |

## 2. Contrôle des cas, tableau par tableau

Colonne « Issue » : ce qui est décidé. Colonne « État » : `ok` quand la combinaison a une issue,
`TROU n` quand elle n'en a pas, avec son numéro dans la liste du bas.

### M1. Le modèle, face à l'état de départ de sa table

| Modèle | La table est dans l'état de départ | Issue | État |
|---|---|---|---|
| présent, géré | oui | comparaison, écarts générés | ok |
| présent, géré | non | création | ok |
| présent, `managed: false` | oui | rien, rappel « en pause » | ok |
| présent, `managed: false` | non | rien, rappel ; la table n'est pas créée | ok |
| absent | oui | suppression proposée, bloquée sans `--force` | ok |
| absent | non | rien, table ignorée | ok |
| deux modèles pour la même table | — | erreur | ok |

### M2. La place d'une migration, face à chaque commande

| Migration | `makemigrations` | `migration up` | `migration down` | État |
|---|---|---|---|---|
| appliquée | déjà dans le snapshot | sautée | défaite, snapshot ramené, fichier et ligne supprimés | ok |
| générée, pas appliquée (tag « en attente ») | remplacée par le nouveau calcul fait depuis le snapshot | appliquée, tag retiré | fichier et ligne supprimés, rien en base | ok |
| générée, pas appliquée, et c'est une suppression dont le modèle est revenu | retirée : l'écart est nul | appliquée si le dev la lance | fichier et ligne supprimés | ok |
| inscrite en base, inconnue du projet | refus (contrôle) | refus, déjà fait par SeaORM (« file is missing ») | refus, déjà fait par SeaORM | ok |
| générée, pas appliquée, et retouchée à la main | écrasée par le nouveau calcul, la documentation le dit | appliquée telle qu'elle est | fichier et ligne supprimés | ok |

### M3. Le moteur, face à chaque opération de `migration up`

| Opération | PostgreSQL | SQLite | MySQL | État |
|---|---|---|---|---|
| tout ou rien du lancement | une transaction | une transaction | compensation | ok |
| création, suppression de table | direct | direct | direct | ok |
| ajout, retrait, renommage de colonne | direct | direct | direct | ok |
| modification de colonne | direct | reconstruction de la table, après `--force` | direct | ok |
| clé étrangère ajoutée ou retirée sur une table existante | direct | reconstruction de la table, comme pour une colonne modifiée | direct | ok |
| renommage de table | direct | direct | direct | ok |
| type enum : création, valeur ajoutée, valeur renommée | type du moteur | texte | texte | ok |
| type enum : suppression | seulement si plus aucune colonne ne s'en sert | sans objet | sans objet | ok |
| valeur d'enum retirée | lignes comptées, question en console | pareil | pareil | ok |
| table déjà là sans inscription | rejouer, vérifier, inscrire, ou ne rien faire | pareil | pareil | ok |

### M4. Le moteur, face à un `migration down` qui échoue en cours

| Moteur | Issue | État |
|---|---|---|
| PostgreSQL | transaction ouverte par SeaORM, tout le `down` est annulé | ok |
| SQLite | transaction ouverte par Runique autour du `down` (SeaORM n'en ouvre pas) | ok |
| MySQL | compensation, comme pour `up` | ok |

### M5. Ce qui arrive à une table, face au snapshot, quand le lancement a réussi

| La migration | Snapshot | Fichier du modèle | État |
|---|---|---|---|
| crée la table | écrit depuis l'entité | inchangé | ok |
| la modifie | réécrit depuis l'entité | inchangé | ok |
| la supprime | retiré | déjà retiré par le dev | ok |
| la renomme (`rename_to`) | le même fichier, sous le nouveau nom, sans reste à l'ancien | mis à jour par Runique, dans le fichier courant | ok |
| ajoute ou retire des colonnes d'un `extend!{}` | réécrit, ou supprimé s'il devient vide | inchangé | ok |

### M6. Une question en console

| Question | Réponse donnée | Aucune réponse | Pas de terminal | État |
|---|---|---|---|---|
| valeur de remplacement d'une valeur d'enum retirée | lignes mises à jour, valeur retirée | rien n'est appliqué | erreur | ok |
| champ obligatoire ajouté à une table remplie : « donner une valeur » ou « ne rien faire » | colonne ajoutée avec cette valeur | « ne rien faire » : toute la commande s'arrête, rien n'est écrit | erreur | ok |

### M7. Le journal de `migration up`, au lancement suivant

| Ce qu'on trouve | Qui est lancé | Issue | État |
|---|---|---|---|
| pas de journal | n'importe quelle commande | normal | ok |
| journal, migrations inscrites en base | `migration up` | snapshots écrits, journal effacé | ok |
| journal, migrations pas inscrites | `migration up` | rien n'a été validé, on recommence | ok |
| journal, instructions faites en partie (MySQL) | `migration up` | compensation, ou suite, selon le point de non-retour | ok |
| journal de `migration up` resté ouvert | `makemigrations` | refus : les deux commandes ne tournent jamais en même temps | ok |
| `migration up` en cours | `makemigrations` (ou l'inverse) | refus, par un verrou tenu pendant toute la commande | ok |
| lot de `makemigrations` interrompu | `migration up` | refus, relancer `makemigrations` | ok, existant |

### M8. Le changement de type d'une colonne

Décidé le 2026-11-20 : un changement de type est destructeur, il passe obligatoirement par
`--force`, et il est alors réellement généré dans le fichier de migration. Le chemin dépend du
moteur (reconstruction de la table sous SQLite).

## 3. Les trous

| N° | Où | État au 2026-11-20 |
|---|---|---|
| 1 | M2 | réglé : tag « en attente » en tête des migrations, et `makemigrations` repart toujours du snapshot ; plus rien à rejouer |
| 2 | M3 | réglé : sous SQLite, la table est reconstruite |
| 3 | M4 | réglé : chaque `down` est encapsulé selon le moteur |
| 4 | M5 | réglé : le renommage se fait dans le fichier courant, le snapshot suit sans reste |
| 5 | M6 | réglé : choix en console, sans valeur par défaut d'office ; « ne rien faire » arrête toute la commande |
| 6 | M7 | réglé : `makemigrations` et `migration up` ne peuvent pas tourner en même temps ; aucun verrou n'existe aujourd'hui, il est à écrire |
| 7 | M8 | réglé : destructeur, `--force`, généré dans le fichier de migration |
