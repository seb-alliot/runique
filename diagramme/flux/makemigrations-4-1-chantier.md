# Flux — makemigrations 4.1 : fonctionnement décidé

Document de travail du chantier makemigrations 4.1, à jour au 2026-11-20. Il décrit le
fonctionnement **visé**, pas le code actuel. Plus aucune question de fonctionnement n'est
ouverte. À remplacer par le flux réel une fois le chantier codé.

| Pour | Fichier |
|---|---|
| une séquence par cas | [makemigrations-4-1-cas.md](makemigrations-4-1-cas.md) |
| les séquences regroupées et le contrôle de chaque combinaison | [makemigrations-4-1-regroupement.md](makemigrations-4-1-regroupement.md) |
| ce qui existe déjà dans le code | [makemigrations-4-1-existant.md](makemigrations-4-1-existant.md) |
| les décisions, une ligne datée chacune | `tableau_suppression_table.md`, section « Décisions prises » |

## Les deux sources (décidé le 2026-11-20)

| Source | Ce que c'est | Écrit par | Quand |
|---|---|---|---|
| **le fichier de migration** | la seule source du `up` et du `down` d'une modification. Il n'en existe aucune copie : pas de dossier `history/` | `makemigrations` | à la génération |
| **le snapshot** | l'état d'une table après un `migration up` ou `down` réussi : la retranscription de l'entité | `migration up` et `down` | à l'application, seulement si elle réussit |

Le dossier `history/`, envisagé les 16 et 19 novembre, est abandonné : il recopiait le `up` et le
`down` que le fichier de migration contient déjà. C'est la raison pour laquelle l'ancien dossier
`applied/` avait été retiré à la 3.0.

## Les règles de fond

| Règle | Source |
|---|---|
| un modèle = une table, seule source de vérité ; deux modèles pour la même table = erreur | Sébastien, 2026-11-19 |
| `makemigrations` lit la base et écrit des fichiers ; il n'applique jamais rien | Sébastien, 2026-11-19 |
| `makemigrations` exige une base ; base injoignable = erreur | Sébastien, 2026-11-19 |
| Runique ne voit que les tables passées par ses migrations ; les autres sont ignorées. Pour reprendre une base existante, une commande à part | Sébastien, 2026-11-19 |
| une migration sortie de `seaql_migrations` n'a plus sa place dans `lib.rs` | Sébastien, 2026-11-19 |
| sur un `down`, Runique ne touche pas aux entités ; sur un `up`, il peut | Sébastien, 2026-11-19 |
| 4.1 = tout `makemigrations`, sans le plusieurs-à-plusieurs ; 4.2 = le plusieurs-à-plusieurs ; `runique import-db` (reprise d'une base existante) en 4.2 ou 4.3 | Sébastien, 2026-11-19 |
| le code propre à un moteur vit dans son fichier, sous sa feature : `db/engine/` avec `postgres.rs`, `mysql.rs`, `sqlite.rs` et un contrat commun ; l'enum du moteur a ses variantes sous feature | Sébastien, 2026-11-20 |
| une migration générée porte un tag « en attente » en tête de fichier ; relancé, `makemigrations` repart du snapshot et la remplace, il n'empile pas | Sébastien, 2026-11-20 |
| `migration up` est en tout ou rien sur le lancement entier ; chaque `down` est encapsulé lui aussi, selon le moteur | Sébastien, 2026-11-20 |
| `makemigrations` et `migration up` ne tournent jamais en même temps | Sébastien, 2026-11-20 |
| un élément destructeur passe par `--force`, qui l'écrit dans le fichier de migration : le dev en accepte les conséquences | Sébastien, 2026-11-20 |

## Vue d'ensemble (découpage de Sébastien)

| Ligne | Contenu | Séquence | Questions ouvertes |
|---|---|---|---|
| 0. Savoir où en est la base | connexion, migrations appliquées, contrôles, migrations en attente (tag) | 1, 3 | — |
| 1. Comparer | écarts entre le snapshot et les modèles | 1 | — |
| 2. Générer | migrations et `lib.rs`, en un lot atomique | 1 | — |
| 3a. Restaurer : revenir en arrière | transaction, compensation sous MySQL, `migration down` | 2, 4 | — |
| 3b. Restaurer : reconstruire le projet depuis une base existante | `runique import-db`, **hors 4.1** (4.2 ou 4.3) | 5 | — |

## Légende

| Marque | Sens |
|---|---|
| `[EXISTANT]` | le code le fait déjà |
| `[DECIDE]` | tranché par Sébastien, pas encore codé |
| `[ETUDE]` | étudié par Claude, pas décidé |

## Séquence 1 : `runique makemigrations`

```mermaid
sequenceDiagram
    autonumber
    actor D as Dev
    participant MM as makemigrations
    participant BA as Builder atomique
    participant F as Fichiers du projet
    participant DB as Base de données

    D->>MM: runique makemigrations (avec ou sans --force)

    alt migration up est en cours, ou son journal est resté ouvert
        MM-->>D: refus, les deux commandes ne tournent jamais en même temps [DECIDE]
    end

    MM->>BA: un lot précédent est-il resté interrompu ?
    BA-->>MM: réparé si besoin [EXISTANT]

    MM->>F: scan de src/entities [EXISTANT]
    alt dossier introuvable ou model invalide
        MM-->>D: erreur avant tout, rien n'est écrit [EXISTANT]
    else deux modèles nomment la même table
        MM-->>D: erreur, un modèle = une table [EXISTANT]
    else deux modèles déclarent un enum du même nom avec des valeurs différentes
        MM-->>D: erreur qui nomme les deux modèles [DECIDE]
    end
    Note over MM,F: un modèle que le lecteur ne voit pas (commenté, sous-dossier, mod.rs,<br/>fichier renommé) est absent [DECIDE]<br/>un modèle avec managed false met sa table en pause [DECIDE]

    MM->>DB: connexion [DECIDE]
    alt base injoignable
        MM-->>D: erreur, sans l'adresse (elle porte le mot de passe) [DECIDE]
    end
    Note over MM,DB: makemigrations ne fait que lire la base, il n'applique rien [DECIDE]

    MM->>DB: quelles migrations sont appliquées ? (seaql_migrations)
    DB-->>MM: liste des noms appliqués
    MM->>F: lecture des snapshots, des fichiers de migration et de lib.rs

    rect rgb(240, 240, 240)
        Note over MM,F: Contrôles avant tout calcul
        MM->>MM: toute migration inscrite en base est connue de lib.rs [DECIDE]
        MM->>MM: aucun snapshot illisible ou mal nommé (cas 36, 37) [DECIDE]
        MM->>MM: pas deux modifications parallèles d'une même table [DECIDE]
        alt un contrôle échoue
            MM-->>D: refus avec un message précis et les tables concernées, rien n'est écrit
        end
    end

    Note over MM,DB: une table de la base que Runique n'a pas migrée est ignorée, sans message [DECIDE]<br/>reprendre une base existante = runique import-db, hors 4.1, séquence 5 [DECIDE]

    MM->>F: quelles migrations portent le tag en attente ? (lu en tête de fichier) [DECIDE]
    MM->>MM: état de départ = le snapshot, tel quel [DECIDE]
    MM->>MM: calcul des écarts entre le snapshot et les modèles [EXISTANT, fonction pure]
    Note over MM: modèle sans snapshot = table à créer<br/>table migrée par Runique sans modèle = table à supprimer [DECIDE]<br/>managed false = aucun écart généré, aucune suppression [DECIDE]

    alt aucun écart
        MM->>F: les migrations en attente qui n'ont plus lieu d'être sont retirées [DECIDE]
        MM-->>D: aucun changement, avec le rappel des tables en pause [DECIDE]
    end

    rect rgb(240, 240, 240)
        Note over MM: Gardes avant d'écrire
        alt changement destructeur sans --force (suppression de table comprise)
            MM-->>D: liste des pertes de données, rien n'est écrit [EXISTANT, étendu]
        end
        alt identifiant trop long
            MM-->>D: erreur, jamais levée par --force [EXISTANT]
        end
        alt champ obligatoire ajouté à une table existante, sans valeur par défaut
            MM-->>D: choix en console, donner une valeur pour les lignes existantes, ou ne rien faire [DECIDE]
            Note over MM,D: « ne rien faire » arrête toute la commande, rien n'est écrit [DECIDE]
        end
        Note over MM: table renommée déclarée par rename_to = renommage, pas suppression + création [DECIDE]<br/>valeur retirée d'un enum et encore portée par des lignes = question en console [DECIDE]
        Note over MM: changement de type = destructeur, puis réellement généré dans la migration [DECIDE]<br/>sous SQLite, colonne modifiée ou clé étrangère changée = destructeur, la table sera reconstruite [DECIDE]
        Note over MM: élément visé par d'autres tables = avertissement, modification impossible,<br/>avec --force les autres tables sont mises à jour [DECIDE]
    end

    MM->>MM: liste d'instructions, chacune avec son inverse, destructrices en dernier [DECIDE]

    MM->>BA: un seul lot
    BA->>F: migrations en attente d'un lancement précédent, remplacées par le nouveau calcul [DECIDE]
    BA->>F: fichiers de migration m_DATE_..., avec le tag en attente en tête [DECIDE]
    Note over BA,F: le fichier de migration est la seule source du up et du down, aucune copie [DECIDE]
    Note over BA,F: chaque migration note celle dont elle part, pour sa table [DECIDE]
    BA->>F: lib.rs
    Note over BA,F: tout ou rien, réparé après une coupure [EXISTANT]
    Note over BA,F: le snapshot n'est pas touché, c'est migration up qui l'écrit [DECIDE]

    MM-->>D: rapport des changements, rien sur la connexion [DECIDE]
```

## Séquence 2 : `runique migration up`

Décidé le 2026-11-20 : tout ou rien sur le lancement entier. Si une migration échoue, aucune de
celles du lancement ne reste appliquée.

```mermaid
sequenceDiagram
    autonumber
    actor D as Dev
    participant UP as migration up
    participant J as Journal
    participant CR as crate migration (sea-orm-migration)
    participant DB as Base visée
    participant F as Fichiers du projet

    D->>UP: runique migration up

    alt makemigrations est en cours
        UP-->>D: refus, les deux commandes ne tournent jamais en même temps [DECIDE]
    end

    UP->>J: un lancement précédent a-t-il été coupé ?
    alt oui
        UP->>DB: ses migrations sont-elles inscrites dans seaql_migrations ?
        alt inscrites
            UP->>F: écriture des snapshots qui manquaient [DECIDE]
        else pas inscrites
            Note over UP,DB: rien n'a été validé, on repart de zéro
        end
    end
    Note over UP,J: le journal est une sécurité contre la coupure de courant<br/>entre la validation de la base et l'écriture des snapshots [DECIDE]

    UP->>CR: cargo run sur la crate migration [EXISTANT]
    CR->>DB: connexion, lecture de seaql_migrations
    DB-->>CR: migrations déjà appliquées

    Note over UP,DB: tables déjà là sans inscription : modifications rejouées en interne,<br/>si tables et colonnes existent les migrations sont inscrites, sinon Runique ne fait rien [DECIDE]
    Note over UP,DB: pas de contrôle des données à l'avance : la base renvoie l'erreur,<br/>puis tout est annulé [DECIDE]

    UP->>J: migrations de ce lancement notées avant de commencer [DECIDE]
    CR->>DB: quel moteur ? (get_database_backend sur la connexion) [EXISTANT]

    alt PostgreSQL
        CR->>DB: BEGIN, une seule transaction pour tout le lancement [DECIDE]
        loop pour chaque migration en attente
            CR->>DB: instructions de up(), inscription dans seaql_migrations
        end
        CR->>DB: COMMIT
        Note over CR,DB: une migration échoue = tout le lancement est annulé [DECIDE]<br/>aujourd'hui la transaction ne couvre qu'une migration à la fois [EXISTANT]
    else SQLite
        Note over CR,DB: une transaction pour tout le lancement, testé : SQLite défait les changements de schéma [DECIDE]<br/>colonne modifiée, clé étrangère ajoutée ou retirée = table reconstruite, après --force [DECIDE]
        Note over CR,DB: clés étrangères coupées AVANT d'ouvrir la transaction :<br/>SQLite ignore ce réglage une fois dedans (testé le 2026-11-20)
        loop pour chaque migration en attente
            CR->>DB: instructions de up(), inscription dans seaql_migrations
        end
    else MySQL
        Note over CR,DB: testé : MySQL garde les changements de schéma après une annulation<br/>pas d'enveloppe de transaction ici, compensation sur tout le lancement [DECIDE]<br/>le down entier d'une migration ne convient pas : seules les instructions déjà faites sont défaites
        loop pour chaque migration en attente, instructions destructrices en dernier
            CR->>DB: instruction
            CR->>J: instruction faite
        end
        alt panne avant la première instruction destructrice du lancement
            CR->>CR: inverses pris dans la liste d'instructions de chaque migration [DECIDE]
            CR->>DB: instructions inverses, de la dernière faite à la première
        else panne après
            CR->>DB: suite des instructions, d'après le journal
        end
        Note over CR,DB: doute sur une instruction = la base tranche par une lecture d'existence [DECIDE]<br/>si la compensation échoue, message clair sur ce qui n'a pas pu être défait [DECIDE]
    end

    alt tout le lancement a réussi
        UP->>F: snapshots des tables touchées retranscrits depuis les entités [DECIDE]
        UP->>F: tag en attente retiré des migrations appliquées [DECIDE]
        opt une migration renommait une table (rename_to)
            UP->>F: fichier du modèle mis à jour, nouveau nom de table, rename_to retiré [DECIDE]
        end
        opt une migration supprimait une table
            UP->>F: snapshot de cette table retiré [DECIDE]
        end
        opt un bloc extend a été retiré en entier
            UP->>F: son snapshot, devenu vide, est supprimé [DECIDE]
        end
        UP->>J: journal effacé
    else le lancement a échoué
        CR-->>UP: erreur, aucune migration de ce lancement n'est inscrite
        Note over UP,F: aucun snapshot n'est touché [DECIDE]
    end

    Note over UP,F: si le modèle a changé entre makemigrations et migration up,<br/>le snapshot retranscrit est faux, la documentation doit le dire [DECIDE]

    UP-->>D: résultat
```

## Séquence 3 : relancer `makemigrations` avant d'avoir appliqué

Décidé le 2026-11-20. Une migration générée porte un tag « en attente » en tête de fichier.
`makemigrations` n'empile pas : il repart du snapshot, recalcule tout l'écart, et remplace les
migrations en attente. Il n'applique jamais rien.

```mermaid
sequenceDiagram
    autonumber
    actor D as Dev
    participant MM as makemigrations
    participant UP as migration up
    participant BA as Builder atomique
    participant F as Fichiers du projet

    D->>MM: premier lancement
    MM->>F: migration M1, avec le tag en attente
    Note over F: migration up n'est pas lancé, le snapshot n'a pas bougé

    D->>MM: second lancement
    MM->>F: lecture des têtes de fichier, M1 est en attente
    MM->>MM: état de départ = le snapshot, calcul de tout l'écart jusqu'au modèle [DECIDE]

    alt le modèle n'a pas changé depuis M1
        MM-->>D: le calcul redonne M1, rien n'est réécrit, pas de doublon [DECIDE]
    else le modèle a encore changé
        MM->>BA: un seul lot
        BA->>F: M1 remplacée par M1b, qui contient tous les changements [DECIDE]
    else le changement de M1 a été annulé dans le modèle (Ctrl+Z)
        MM->>BA: un seul lot
        BA->>F: M1 retirée, l'écart est nul [DECIDE]
    end

    D->>UP: runique migration up
    UP->>F: lancement réussi, le tag en attente est retiré, le snapshot est écrit [DECIDE]

    Note over D,F: une migration en attente retouchée à la main est écrasée au lancement suivant,<br/>la documentation le dit [DECIDE]
```

## Séquence 4 : `runique migration down`

Une seule commande, qui enveloppe SeaORM. Elle vise toujours la migration la plus récente du
projet, appliquée ou non.

```mermaid
sequenceDiagram
    autonumber
    actor D as Dev
    participant DN as migration down
    participant BA as Builder atomique
    participant F as Fichiers du projet
    participant DB as Base de données

    D->>DN: runique migration down
    DN->>F: quelle est la migration la plus récente du projet ? porte-t-elle le tag en attente ? [DECIDE]

    alt elle n'est pas appliquée (tag en attente)
        Note over DN,DB: rien n'est défait en base
        DN->>BA: un seul lot
        BA->>F: fichier de migration supprimé, ligne retirée de lib.rs [DECIDE]
        DN-->>D: migration en attente retirée
    else elle est appliquée
        Note over DN,DB: le down est celui du fichier de migration, seule source [DECIDE]
        alt PostgreSQL
            DN->>DB: down dans une transaction, ouverte par SeaORM [EXISTANT]
        else SQLite
            DN->>DB: down dans une transaction, ouverte par Runique [DECIDE]
        else MySQL
            DN->>DB: down instruction par instruction, compensation en cas de panne [DECIDE]
        end
        DN->>DB: ligne retirée de seaql_migrations
        alt le down réussit
            DN->>F: snapshot ramené à l'état d'avant, par le même down appliqué au snapshot [DECIDE]
            DN->>BA: un seul lot
            BA->>F: fichier de migration supprimé, ligne retirée de lib.rs [DECIDE]
            DN-->>D: retour arrière fait
        else le down échoue
            DN-->>D: erreur, la base est revenue comme avant, aucun fichier n'est touché
        end
    end

    Note over D,F: l'entité n'est jamais remise à son état d'origine,<br/>c'est au dev de reprendre son modèle ou de régénérer [DECIDE]
    Note over DN,DB: aujourd'hui la commande passe la main à la crate, SeaORM défait la dernière migration appliquée,<br/>et aucun fichier n'est touché [EXISTANT]
    Note over D,F: à documenter : une migration appliquée sur une autre base,<br/>ou retouchée à la main, est perdue par cette suppression
```

## Séquence 5 : `runique import-db`, reprendre une base existante

Décidé le 2026-11-19. **Hors de la 4.1 : prévue en 4.2 ou 4.3.** La reprise n'est pas
déclenchée par `makemigrations` : c'est une commande à part, `runique import-db`. Elle écrit un
semblant de modèle par table, avec les types de la base recopiés tels quels ; `runique_dsl` les
refuse tant que le dev ne les a pas repris à la main. La documentation doit le dire clairement :
le typage est à refaire à la main.

```mermaid
sequenceDiagram
    autonumber
    actor D as Dev
    participant CLI as runique import-db
    participant MM as makemigrations
    participant UP as migration up
    participant BA as Builder atomique
    participant F as Fichiers du projet
    participant DB as Base de données

    D->>CLI: runique import-db
    CLI->>DB: connexion, lecture du schéma (sea-schema)
    DB-->>CLI: tables, colonnes avec leur type, clés, index
    Note over CLI,DB: toutes les tables sont parcourues, sauf seaql_migrations et celles du framework [DECIDE]

    CLI->>BA: un lot, dans src/entities
    BA->>F: un semblant de modèle par table, types de la base recopiés (email varchar) [DECIDE]
    Note over BA,F: un fichier déjà présent n'est pas écrasé [ETUDE]
    CLI-->>D: modèles écrits, à reprendre à la main, puis lancer makemigrations [DECIDE]

    D->>D: adapte chaque modèle à Runique (text, email, password...), retire ceux qu'il ne veut pas
    Note over D,MM: tant qu'un type de la base reste dans un modèle,<br/>runique_dsl le refuse avec sa suggestion [EXISTANT]

    D->>MM: runique makemigrations
    MM->>F: modèles sans snapshot = migrations de création, lib.rs
    D->>UP: runique migration up
    UP->>DB: les tables existent déjà et correspondent
    UP->>DB: migrations inscrites dans seaql_migrations, sans être exécutées [DECIDE]
    UP->>F: snapshots retranscrits depuis les entités [DECIDE]
    Note over D,DB: à partir de là, ces tables sont des tables de Runique comme les autres
```

## Questions tranchées

Gardées pour mémoire, avec la réponse retenue. Les sept trous trouvés en regroupant les
séquences sont dans [makemigrations-4-1-regroupement.md](makemigrations-4-1-regroupement.md).

| Q | Question | État |
|---|---|---|
| Q2 | migration générée et pas appliquée | **décidé le 2026-11-20** : elle porte un tag « en attente » ; relancé, `makemigrations` repart du snapshot et la remplace par le nouveau calcul (cumul, sans doublon) ; il n'applique rien |
| Q3 | base qui n'a pas reçu les migrations | **décidé** : le calcul ne dépend plus de la base, il part du snapshot ; ce qui est appliqué ou non se lit au tag |
| Q4 | contrôles sur les données avant d'appliquer | **décidé** : aucun contrôle à l'avance ; la base renvoie l'erreur, puis transaction ou compensation |
| Q5 | SQLite, colonne modifiée ou clé étrangère changée sur une table existante | **décidé** : élément destructeur, bloqué sans `--force` avec le risque expliqué ; la table est reconstruite |
| Q6 | portée de la transaction | **décidé, revu le 2026-11-20** : tout ou rien sur le lancement entier ; une seule transaction pour tout le lancement (PostgreSQL, SQLite si l'essai passe) ; sinon compensation sur tout le lancement (MySQL), avec un message clair sur ce qui n'a pas pu être compensé |
| Q8 | suppression écrite et pas appliquée, puis relance | **décidé** : recalculée à l'identique si le modèle est toujours absent ; retirée si le modèle est revenu |
| Q9 | deux devs sur la même table | **décidé** : refus, avec les tables concernées ; pas de commande de réunion |
| Q10 | table à ne pas toucher, modèle invisible | **décidé** : `managed: false` dans le `meta` pour la pause ; un modèle que le lecteur ne voit pas est absent |
| Q11 | aucun modèle trouvé | **décidé** : pas de cas à part, les tables migrées par Runique et sans modèle sont proposées à la suppression avec `--force` |
| Q12 | renommage de table ou de champ liste | **décidé** : `rename_to` dans le modèle, retiré par Runique une fois le renommage fait |
| Q13 | table déjà là sans inscription à `migration up` | **décidé** : `migration up` rejoue les modifications en interne ; si les tables et colonnes attendues existent en base, il inscrit les migrations à la date du `migration up` ; les types ne sont pas comparés à la base, ils sont contrôlés dans le modèle ; si la base ne correspond qu'en partie, Runique ne fait rien |
| Q14 | table de la base sans modèle, que Runique n'a pas migrée | **décidé** : ignorée sans message par `makemigrations` ; pour la reprendre, `runique import-db` |
| Q15 | reprise : ce qui est écrit | **décidé** : `runique import-db` (4.2 ou 4.3) écrit un semblant de modèle par table, types de la base recopiés ; les migrations viennent ensuite par `makemigrations`, une fois les modèles repris |
| Q16 | reprise : quelles tables | **décidé** : toutes ; le dev retire les modèles qu'il ne veut pas |
| Q17 | reprise : champs que la base ne dit pas | **décidé** : aucun type n'est choisi par Runique ; `runique_dsl` refuse le modèle tant que le dev n'a pas repris chaque champ |
| Q18 | correspondance des types par moteur | **sans objet** en 4.1 : ni la reprise ni `migration up` n'en ont besoin, le typage est contrôlé dans le modèle |
| Q19 | reprise : inscription du point de départ | **décidé** par Q13 et Q15 : c'est `migration up` qui inscrit quand les tables correspondent, sans option à part |
| Q20 | question en console sans personne pour répondre | **décidé** : la commande s'arrête avec une erreur qui explique quoi faire ; la reprise n'en pose plus, c'est une commande à part |
| Q21 | reprise : lot sur deux dossiers | **sans objet** : `runique import-db` n'écrit que dans `src/entities/`, un seul builder suffit |
| Q22 | forme du générateur | **décidé** : liste d'instructions, chacune avec son inverse ; un retour arrière reprend à un rang donné |
| Q23 | type enum PostgreSQL partagé | **décidé** : contrôle à l'exécution avant tout `DROP TYPE`, jamais de cascade ; même nom et valeurs différentes = erreur |
| Q24 | `makemigrations` sans base | **décidé** : il exige une base, tests sur les trois moteurs par Docker |
| Q25 | valeur retirée d'un enum | **décidé** : s'il ne reste aucune ligne avec la valeur, elle est retirée ; sinon question en console pour la valeur de remplacement, et sans choix rien n'est appliqué |
| Q26 | migrations du framework et refus | sans objet depuis Q3 : il n'y a plus de refus général ; leur nom fixe sert toujours à lire leur état |
| Q27 | champ obligatoire ajouté à une table remplie | **décidé** : choix en console (`dialoguer`), donner une valeur ou ne rien faire ; pas de valeur par défaut d'office ; « ne rien faire » arrête toute la commande ; le garde existant pour « facultatif vers obligatoire » est étendu aux colonnes ajoutées |
| Q28 | `migration down` | **décidé** : une seule commande, qui enveloppe SeaORM et vise la migration la plus récente du projet. Pas appliquée : fichier et ligne de `lib.rs` supprimés. Appliquée : `down` du fichier de migration, snapshot ramené par ce même `down`, puis fichier et ligne supprimés. Entités jamais touchées |
| Q29 | savoir ce que le snapshot contient déjà | **décidé** : c'est le tag « en attente » des migrations qui le dit ; aucune marque dans le snapshot |
