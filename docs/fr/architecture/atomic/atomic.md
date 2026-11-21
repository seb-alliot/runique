# Écriture atomique des fichiers

Quand Runique écrit plusieurs fichiers d'un coup, il les écrit comme un seul lot : **tous, ou aucun**. Une erreur au milieu, ou une coupure de courant, ne laisse jamais un projet à moitié écrit.

Vous n'appelez jamais ce mécanisme vous-même. Cette page explique ce qu'il fait, les fichiers qu'il peut laisser derrière lui, et quoi faire quand une commande refuse de continuer.

---

## Où Runique s'en sert

| Commande ou fonction | Ce qui est écrit en un lot | Journal |
| --- | --- | --- |
| `runique new` | tous les fichiers du projet, la crate `migration/` et le `.env` | dans le dossier du projet |
| `runique makemigrations`, premier lancement | la crate `migration/` si le projet n'en a pas | dans `migration/` |
| `runique makemigrations` | les migrations, la suppression de celles qu'il remplace, et `lib.rs` | dans `migration/src/` |
| `runique migration up`, après application | les snapshots des tables touchées et le retrait du tag des migrations | dans `migration/src/` |
| envoi de fichiers (`FileField`, `MediaField`) | les fichiers d'une même requête | aucun |

---

## Comment un lot est écrit

1. Runique note dans un **journal** la liste des fichiers qu'il va écrire.
2. Chaque fichier est d'abord écrit sous un nom provisoire, à côté de sa destination.
3. Si un fichier existe déjà à cet endroit, il est mis de côté dans une sauvegarde.
4. Les fichiers provisoires prennent leur place.
5. Le journal est marqué « tout est en place ».
6. Les provisoires et les sauvegardes sont supprimés, puis le journal, en dernier.

La règle qui en découle :

| Moment de l'incident | Ce que fait Runique |
| --- | --- |
| avant la marque de l'étape 5 | il annule : le projet revient exactement à son état d'avant |
| après la marque | il termine le ménage : tous les fichiers sont déjà à leur place |

---

## Les fichiers que vous pouvez croiser

Ils n'existent que pendant un lancement. Si vous en voyez un alors qu'aucune commande ne tourne, un lancement a été interrompu.

| Fichier | Ce que c'est |
| --- | --- |
| `.runique-journal` | le journal du lot, dans le dossier indiqué plus haut |
| `<fichier>.runique-tmp` | le nouveau contenu, pas encore à sa place |
| `<fichier>.runique-bak` | l'ancien contenu d'un fichier que le lot remplace |

Ces trois noms sont réservés à Runique : ne créez pas de fichier qui se termine par `.runique-tmp` ou `.runique-bak`, ni de fichier nommé `.runique-journal`. Ils n'ont pas à être versionnés.

---

## Après une erreur

Disque plein, droit d'écriture refusé, fichier verrouillé : la commande s'arrête, annule tout ce qu'elle avait écrit et affiche l'erreur, avec le fichier en cause. Le projet est dans l'état où il était avant. Il n'y a rien à nettoyer : corrigez la cause, puis relancez.

---

## Après une coupure

Une coupure de courant, ou un processus tué, ne laisse pas à Runique le temps d'annuler. C'est le journal resté sur le disque qui permet de réparer, **au lancement suivant de la même commande** :

| Vous relancez | Ce que fait Runique d'abord |
| --- | --- |
| `runique new mon_projet` | il retire le projet inachevé, puis le crée normalement |
| `runique makemigrations` | il remet `migration/src/` dans son état d'avant, puis calcule les migrations |

La réparation est annoncée par une ligne :

```text
Un lancement interrompu a été réparé dans 'migration/src'.
```

Si la coupure est arrivée pendant le ménage final, les fichiers étaient déjà tous en place : Runique termine simplement le ménage. Pour `runique new`, le projet est alors complet, et la commande vous répond que le dossier existe déjà.

### `runique migration` attend la réparation

`runique migration up`, `down` et `status` ne font que lire les migrations : elles ne réparent rien. Tant qu'un journal est présent dans `migration/` ou `migration/src/`, elles refusent de partir, pour ne pas appliquer des migrations à moitié écrites :

```text
Un makemigrations a été interrompu dans 'migration/src'. Relancez makemigrations avant d'appliquer les migrations.
```

---

## Quand Runique refuse de réparer

La réparation ne connaît que les états qu'une coupure peut laisser. Si un fichier du lot a été **modifié, supprimé ou ajouté à la main** depuis la coupure, Runique ne peut plus savoir quelle version est la bonne. Il ne choisit pas à votre place : **il ne touche à rien** et liste les fichiers en cause.

```text
Un lancement a été interrompu dans 'migration/src' et des fichiers ont changé depuis. Rien n'a été touché. Corrigez ceux listés ci-dessous, puis relancez la commande :
migration/src/lib.rs : modifié depuis l'interruption, et sa sauvegarde contient autre chose
```

Chaque ligne nomme un fichier et donne la raison, dans la langue de l'application :

| Raison affichée | Quoi faire |
| --- | --- |
| `modifié depuis l'interruption, et sa sauvegarde contient autre chose` | la sauvegarde `.runique-bak` contient l'ancienne version. Pour garder votre version : supprimez le `.runique-bak`. Pour revenir à l'ancienne : copiez le `.runique-bak` sur le fichier |
| `ce n'est pas le fichier que Runique était en train de créer` | déplacez ou supprimez ce fichier |
| `le fichier a disparu, et aucune sauvegarde ne le contient` | remettez-le, depuis git par exemple |
| `une sauvegarde existe, alors qu'il n'y avait rien avant le lancement` | supprimez ce `.runique-bak` |
| `ce fichier ne vient pas de Runique, dans un dossier que Runique a créé` | Runique ne supprimera pas ce dossier avec un fichier à vous dedans : déplacez ce fichier ailleurs |
| `ce fichier n'est plus dans le dossier du journal` | un dossier du projet a été remplacé par un lien vers un autre endroit : remettez le vrai dossier |

Une fois les fichiers listés corrigés, relancez la commande : la réparation reprend du début.

Pour des migrations versionnées, `git status` montre d'un coup ce qui a bougé dans `migration/src/`.

### Un provisoire ou une sauvegarde déjà là

Hors de toute coupure, une commande refuse aussi de démarrer si un `.runique-tmp` ou un `.runique-bak` occupe déjà la place dont elle a besoin :

```text
migration/src/lib.rs.runique-bak occupe la place : ce fichier ne vient pas du lancement en cours
```

S'il contient quelque chose que vous voulez garder, récupérez-le ; sinon supprimez-le, puis relancez.

---

## Ce qu'il ne faut pas faire

- **Supprimer `.runique-journal` à la main.** Sans lui, Runique ne sait plus qu'un lot a été interrompu : les fichiers déjà remplacés restent tels quels, et les `.runique-tmp` et `.runique-bak` restent sur place. Laissez la commande réparer.
- **Lancer `runique migration up` pour « passer outre ».** Elle refusera tant que le journal est là, et c'est voulu.

Si le journal a tout de même été supprimé, il faut défaire le lot à la main : pour chaque `<fichier>.runique-bak`, recopiez-le sur `<fichier>` ; puis supprimez les `.runique-bak` et les `.runique-tmp`.

---

## Les envois de fichiers

Les fichiers d'une même requête sont enregistrés dans le dossier média comme un lot : si l'un d'eux ne peut pas l'être, ceux déjà enregistrés sont retirés, et le formulaire affiche l'erreur.

Un envoi n'a pas de journal. Une coupure pendant l'enregistrement peut donc laisser un `.runique-tmp` dans le dossier média ; il peut être supprimé sans risque.

L'option `.replace()` d'un champ de fichier, qui supprime l'ancien fichier quand un nouveau le remplace, est décrite avec les [champs de formulaire](/docs/fr/formulaire/champs).

---

## Limites

- **Deux commandes en même temps** dans le même projet ne sont pas protégées l'une de l'autre : ne lancez pas deux `makemigrations` à la fois.
- **Windows** : le mécanisme est écrit pour, mais il n'a été éprouvé que sous Linux.

---

← [**Architecture**](/docs/fr/architecture) | [**Makemigrations**](/docs/fr/architecture/makemigrations) →
