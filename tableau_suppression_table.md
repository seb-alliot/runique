# Suppression de table par makemigrations : tous les cas

Document de travail du chantier makemigrations 4.1 (morceau 1). À supprimer quand le morceau est
codé, testé et documenté.

État au 2026-11-16 : rien n'est codé pour le morceau 1. Le morceau 0 (suppression d'un fichier
dans un lot du builder atomique, `SourceItems::Remove`) est codé et ses tests passent.

## Orientation générale du chantier (Sébastien, 2026-11-16)

La discussion sur le cas 44 a élargi le sujet au-delà de la suppression de table. Trois points
posés par Sébastien :

1. **Validé : `makemigrations` contrôle la base en premier.** Aujourd'hui il ne se fie qu'aux
   modèles et aux snapshots, donc toute incohérence n'est découverte qu'à `migration up`. La
   base réelle (A, déjà appliquée) devient le point de départ ; les modèles (B, sur le papier)
   sont l'état voulu ; `makemigrations` calcule ce qu'il faut appliquer à A pour obtenir B, sans
   rien casser.
2. **Réagir différemment selon le moteur**, à la différence d'aujourd'hui.
3. **Ajouter le code manquant** pour les changements pas encore pris en charge.

Le tout branché sur le builder atomique. Certaines étapes seront longues, avec tous les cas à
croiser par moteur.

**Snapshots (décision du 2026-11-16) : gardés pour le moment.** Leur utilité sera réévaluée une
fois le rework fini. Constat qui a mené à la question : si le diff se fait entre les modèles et
la base réelle, le snapshot ne sert plus à dire où en est la table. Ce que lui seul sait
aujourd'hui : les valeurs d'un enum sous MySQL et SQLite (colonne texte en base), les types dans
le vocabulaire de Runique, l'état après les migrations écrites et pas encore appliquées.

**Mise à jour des snapshots (décision du 2026-11-16, « pour le moment ») : à l'application par
`migration up`, et non plus à `makemigrations`.** Le snapshot ne décrit donc plus qu'un état
réellement appliqué. Situations que le code devra couvrir : un lancement où une migration sur
plusieurs échoue, `migration down`, une migration appliquée sans passer par Runique, plusieurs
bases pour un seul fichier commité.

**Le journal du builder comme sauvegarde inversée (suggestion de Sébastien, 2026-11-16).** Les
snapshots sont écrits à `migration up` par un lot du builder dont le journal reste ouvert pendant
que la base est modifiée : la migration réussit, le lot est fini ; elle échoue, le journal remet
les anciens snapshots. Ce que le builder a déjà : les deux issues (`placed: false` défait,
`placed: true` finit) et la réparation après une coupure. Ce qui manque, relevé par Claude :

- pouvoir tenir le lot ouvert entre la pose des fichiers et la fin (`start()` fait tout d'un
  trait ; `finish` et `undo` existent mais ne sont pas offerts séparément) ;
- appliquer une migration à la fois, un lot par migration (`sea-orm-migration` accepte
  `up -n 1`, vérifié dans son `cli.rs`) ;
- une coupure entre la validation de la base et le marquage du journal : la réparation doit
  demander à `seaql_migrations` si la migration est inscrite, donc le journal doit noter quelle
  migration le lot accompagnait.

**Suivre le chemin de Django (décision de Sébastien, 2026-11-16).** Motif donné : plus de vingt
ans d'expérience sur le sujet, et c'est du SQL, pas du bas niveau. Ce que ça recouvre, d'après
la discussion qui l'a précédée :

- l'historique reste dans les fichiers du projet ; la base ne donne que sa position
  (`seaql_migrations`) ;
- le calcul des changements se fait d'après les fichiers, pas d'après la base ;
- la base sert à contrôler la cohérence.

Vérifié dans la documentation de Django 5.2 (topics/migrations) :

- « Django compares changes against migration files, not the database. »
- historique incohérent avec la base : « Django will refuse to run migrations or make new
  migrations until it's fixed. »
- chaque migration déclare celles dont elle dépend (`dependencies`) ; deux migrations créées en
  parallèle sont détectées, et `makemigrations --merge` les ordonne ;
- transactions : SQLite et PostgreSQL enveloppent chaque migration ; MySQL non, « you will have
  to manually unpick the changes in order to try again » ;
- SQLite : Django reconstruit la table (créer, copier, supprimer, renommer), « it can be slow
  and occasionally buggy ».

Non trouvé dans ces pages : ce que fait `makemigrations` quand la base est injoignable.

Différence gardée avec Django : il ne stocke que les opérations et rejoue tout en mémoire ; la
piste de Sébastien garde l'état de la table après chaque migration (dossier par table, entrée
datée, sur le modèle de l'ancien `applied/`). Étudiée au §9 de l'évaluation, pas encore décidée
dans sa forme.

**MySQL : compensation par le journal (proposition de Sébastien, retenue le 2026-11-16).** MySQL
valide chaque instruction de schéma d'elle-même, donc pas de transaction possible. À la place,
le mécanisme du builder est appliqué aux instructions SQL d'une migration : chaque instruction a
son étape, le journal note où on en est, et un échec fait repasser sur ce qui a été fait, en
sens inverse. Un « semblant de transaction » (mot de Sébastien).

- La migration générée connaît déjà l'inverse de chaque opération (`up()` et `down()` en miroir) ;
  la crate `migration/` dépend de Runique, le fichier généré peut donc confier ses instructions
  à du code de Runique qui les exécute une par une.
- Les instructions destructrices sont placées **en dernier** : elles ne se compensent pas (les
  données ne reviennent pas). Aujourd'hui les suppressions de colonnes sont au début
  ([alter.rs](runique/src/migration/generate/alter.rs), étape 3 sur 11) : ordre à revoir.
- **Règle du point de non-retour, la même que celle du builder (`placed`)** : le point de
  non-retour est la première instruction destructrice. Panne avant : on annule, la table revient
  à l'état d'avant, on peut recommencer. Panne après : on continue d'après le journal et on
  finit. Ce n'est pas un choix du dev, c'est le journal qui le dit.
- Le journal est sur disque : il reste lisible quand la base ne répond plus (connexion perdue,
  la principale cause d'échec qui restait).
- Trou connu : une panne entre une instruction réussie et sa ligne dans le journal. Pour cette
  instruction seulement, la base tranche par une lecture d'existence (`has_column`,
  `has_index`).
- Limite assumée : pas d'isolation, les autres connexions voient les états intermédiaires.

Résultat visé après un échec, sur les trois moteurs : rien d'appliqué, ou tout. PostgreSQL par
la transaction du moteur ; SQLite par `use_transaction` si l'essai est concluant ; MySQL par
cette compensation. C'est au-delà de Django, qui laisse le dev réparer à la main sous MySQL.

Faits vérifiés dans `sea-orm-migration` 2.0.4 pendant cette discussion :

- la transaction de `migration up` couvre **une migration**, pas le lancement entier, et par
  défaut seulement sous PostgreSQL (`migrator/exec.rs`, `should_use_transaction`) ;
- `MigrationTrait::use_transaction() -> Some(true)` force la transaction sur tout moteur : piste
  pour SQLite, **pas encore essayée** ;
- MySQL valide chaque instruction de schéma d'elle-même : une migration peut s'arrêter entre deux
  instructions, jamais au milieu d'une ;
- une migration n'est inscrite dans `seaql_migrations` qu'après la fin de son `up()` : on sait
  toujours laquelle a échoué ;
- SeaORM a une feature `schema-sync` (lecture de la base par `sea-schema`, comparaison aux
  entités), expérimentale et qui n'ajoute que : ni `ALTER` ni `DROP`.

Causes d'échec sous MySQL : données qui ne respectent pas la nouvelle règle, base dans un autre
état que celui supposé, limites du moteur, panne pendant l'exécution, migration retouchée à la
main. Les trois premières se voient en lisant la base visée avant d'appliquer ; il reste à
Runique la panne pendant l'exécution.

## La règle proposée

Un snapshot à la racine de `migration/src/snapshots/` dont aucun modèle scanné ne produit la
table est une table à supprimer : `makemigrations` écrit une migration
`m{date}_drop_{table}_table` et supprime le snapshot, dans le même lot.

Ce que le scan lit aujourd'hui, et dont la règle dépend :

- les fichiers `.rs` du **premier niveau** de `src/entities/`, sauf `mod.rs`
  ([scan.rs:32](runique/src/migration/cli_migration/scan.rs#L32)) ;
- un fichier sans `model!{}` est ignoré sans message ;
- chaque champ liste (`checkbox` / `multichoice`) produit sa table `{table}_{champ}`
  ([to_schema.rs:136](runique/src/migration/parse/builder/to_schema.rs#L136)) ;
- les tables du framework (`eihwaz_*`) sont écartées du scan.

## Colonne « Statut »

| Valeur | Sens |
|---|---|
| existant | le code le fait déjà, le morceau 1 n'y change rien |
| règle | découle de la règle proposée, sans choix à faire |
| proposé | choix de conception proposé par Claude, pas encore validé |
| à trancher | plusieurs réponses possibles, décision de Sébastien attendue |
| indétectable | `makemigrations` ne lit pas la base, il ne peut pas le voir |
| décidé | tranché par Sébastien, voir « Décisions prises » |
| fait | déjà codé et testé |

## Tous les cas

| # | Famille | Situation | Résultat | Statut |
|---|---|---|---|---|
| 1 | Base | modèle présent, snapshot présent | diff, `ALTER` s'il y a un écart | existant |
| 2 | Base | modèle présent, pas de snapshot | `CREATE TABLE` | existant |
| 3 | Base | pas de modèle, pas de snapshot | rien ; si la table existe en base, elle y reste | indétectable |
| 4 | Base | pas de modèle, snapshot présent | `DROP TABLE` + snapshot supprimé | règle |
| 5 | Retrait voulu | le fichier `menu.rs` est supprimé | cas 4 | règle |
| 6 | Retrait voulu | le fichier reste, le `model!{}` est retiré | cas 4 | règle |
| 7 | Retrait voulu | un champ `checkbox` / `multichoice` est retiré | `DROP TABLE {table}_{champ}` | règle |
| 8 | Retrait voulu | un modèle avec champ liste est retiré | les deux tables, la table liste d'abord | règle |
| 9 | Retrait voulu | la table de jointure m2m écrite à la main est retirée | cas 4 | règle |
| 10 | Retrait voulu | tous les modèles sont retirés | toutes les tables que Runique a migrées, avec la liste et `--force` | décidé |
| 11 | Sans effet | fichier renommé, `table_name` inchangé | rien | existant |
| 12 | Sans effet | un autre modèle reprend le même `table_name` | diff entre l'ancien et le nouveau schéma, pas de `DROP` | règle |
| 13 | Sans effet | `makemigrations` relancé avant d'appliquer le `drop` | même résultat : la migration en attente est recalculée à l'identique, rien n'est réécrit | décidé |
| 14 | Retrait non voulu | `table_name` changé | avec `rename_to` : renommage, sans perte ; sans : `DROP` + `CREATE`, bloqués sans `--force` | décidé |
| 15 | Retrait non voulu | `table_name` changé sur un modèle avec champ liste | comme le cas 14, la table liste suit le renommage | décidé |
| 16 | Retrait non voulu | champ liste renommé | avec `rename_to` : renommage ; sans : `DROP` + `CREATE`, bloqués sans `--force` | décidé |
| 17 | Retrait non voulu | champ liste changé en champ simple | `DROP` de la table liste + colonne ajoutée, valeurs perdues | règle |
| 18 | Retrait non voulu | `menu.rs` déplacé dans un sous-dossier de `entities/` | `DROP TABLE` : le scan ne lit que le premier niveau | décidé |
| 19 | Retrait non voulu | `model!{}` mis en commentaire | absent pour le lecteur : `DROP TABLE`, bloqué sans `--force` ; la pause se demande par `managed: false` | décidé |
| 20 | Retrait non voulu | modèle déplacé dans un fichier `mod.rs` | `DROP TABLE` : le scan saute `mod.rs` | décidé |
| 21 | Retrait non voulu | extension du fichier changée (`menu.rs.bak`) | `DROP TABLE` | décidé |
| 22 | Mauvais dossier | dossier `entities` sans aucun modèle, historique présent | même traitement que le cas 10 : liste affichée, rien sans `--force` | décidé |
| 23 | Mauvais dossier | dossier `entities` d'un autre projet, passé à la main | aucun code : `makemigrations` se lance à la racine du projet ; pour un mauvais chemin, la liste affichée et `--force` suffisent, c'est la responsabilité du dev | décidé |
| 24 | Mauvais dossier | dossier `entities` introuvable | erreur avant tout | existant |
| 25 | Mauvais dossier | un `model!{}` invalide dans le dossier | erreur avant tout | existant |
| 26 | Relations | la table supprimée est visée par le `belongs_to` d'un modèle qui reste | erreur du scan, et la macro ne compile plus | existant |
| 27 | Relations | le `belongs_to` a été retiré du modèle qui reste | `ALTER … DROP FOREIGN KEY` d'abord, `DROP TABLE` ensuite | règle |
| 28 | Relations | la table supprimée vise une table qui reste | `DROP TABLE` direct | règle |
| 29 | Relations | la table supprimée vise une table du framework (`eihwaz_users`) | `DROP TABLE` direct | règle |
| 30 | Relations | deux tables supprimées, l'une vise l'autre | celle qui vise d'abord (ordre inverse de la création) | décidé, si le flux ne casse rien |
| 31 | Relations | tables supprimées en cycle de clés | la clé qui ferme le cycle retirée avant, sauf sous SQLite (réutilise `cycle_keys`) | décidé, si le flux ne casse rien |
| 32 | Relations | table supprimée qui se vise elle-même | `DROP TABLE` direct | règle |
| 33 | Relations | table supprimée avec des colonnes enum | `DROP TYPE` après le `DROP TABLE` (PostgreSQL), seulement si plus aucune colonne ne l'utilise | décidé |
| 34 | Contenu de `snapshots/` | fichier qui n'est pas un `.rs` | ignoré | décidé, si le flux ne casse rien |
| 35 | Contenu de `snapshots/` | dossier `snapshots/runique/` (les `extend!{}`) | les tables du framework ne sont jamais supprimées ; les colonnes ajoutées par un `extend!{}`, elles, sont gérées (voir cas 48) | décidé |
| 36 | Contenu de `snapshots/` | `.rs` illisible ou qui n'est pas un snapshot | erreur, rien n'est écrit | décidé |
| 37 | Contenu de `snapshots/` | `menus.rs` qui décrit la table `plats` | erreur, rien n'est écrit | décidé |
| 38 | Contenu de `snapshots/` | snapshot `eihwaz_*` à la racine | jamais supprimé : seul le snapshot d'une table supprimée est retiré | décidé |
| 39 | Contenu de `snapshots/` | snapshot supprimé à la main, modèle retiré aussi | cas 3 : la table reste en base | indétectable |
| 40 | Contenu de `snapshots/` | snapshot supprimé à la main, modèle présent | `CREATE TABLE … IF NOT EXISTS` | existant |
| 41 | `--force` | suppression sans `--force` | liste `menus: DROP TABLE (data loss)`, rien n'est écrit | décidé |
| 42 | `--force` | suppression + changements sans danger dans le même lancement | tout est bloqué ensemble (un seul garde) | existant |
| 43 | `--force` | suppression avec `--force` | les instructions destructrices sont écrites dans le fichier de migration ; le snapshot n'est pas touché par `makemigrations` | décidé |
| 44 | Base de données | `drop` généré, pas appliqué, puis le modèle est remis | au lancement suivant l'écart est nul : la migration de suppression en attente disparaît | décidé |
| 45 | Base de données | la table n'existe pas en base quand le `drop` passe | `DROP TABLE IF EXISTS`, pour ne pas faire échouer `migration up` | décidé |
| 46 | Base de données | `migration down` après le `drop` | la table revient vide ; le fichier de migration généré est supprimé et le snapshot suit ; l'entité n'est pas remise en état, à dire dans la doc | décidé |
| 47 | Base de données | la table contient des lignes | perdues : le dev en accepte les conséquences en validant `--force` | décidé |
| 48 | `extend!{}` | le bloc `extend!{}` d'une table du framework est retiré, ou une de ses colonnes | les colonnes ajoutées sont retirées (destructeur, `--force`) ; la table du framework reste ; un snapshot d'`extend!{}` devenu vide est supprimé à l'application | décidé |
| 49 | Écriture | coupure pendant l'écriture du lot | réparé au lancement suivant | fait |
| 50 | Écriture | le snapshot disparaît entre le calcul et l'écriture | le lot échoue et tout est remis | fait |
| 51 | Restes hors base | droits `eihwaz_*` de la ressource | élagués au démarrage | existant |
| 52 | Restes hors base | historique admin de la ressource | lignes gardées, à signaler dans la sortie | morceau 2 |
| 53 | Restes hors base | fichiers `media/` des champs `file` | gardés sur le disque, à signaler dans la sortie | morceau 2 |

## Points à trancher

Dans l'ordre proposé. La colonne « Décision » reste vide tant que Sébastien n'a pas tranché.

| Cas | Question | Décision |
|---|---|---|
| 18, 19, 20, 21, 23 | Le blocage sans `--force` suffit-il comme seul filet contre un retrait non voulu, ou faut-il un garde de plus ? | |
| 10, 22 | Refuser quand aucun modèle n'est trouvé : garder ce refus même avec `--force`, au prix du cas « je retire mon dernier modèle » ? | non (2026-11-19) : suppression normale, liste et `--force` |
| 36, 37 | Un snapshot illisible, ou dont le nom ne correspond pas à sa table : erreur et arrêt ? | oui (2026-11-16) |
| 44 | Le `drop` non appliqué suivi d'un modèle remis : limite à documenter, ou autre chose ? | réglé (2026-11-19) : `makemigrations` refuse tant que le `drop` est en attente, le dev le retire |
| 45 | `DROP TABLE IF EXISTS` plutôt que `DROP TABLE` ? | oui (2026-11-16) |
| 14, 15, 16 | Renommage de table et de champ liste : accepté comme suppression + création jusqu'au `renamed_from` de table (morceau 7), ou à traiter avant ? | |

## Décisions prises

| Date | Cas | Décision de Sébastien |
|---|---|---|
| 2026-11-16 | 36, 37 | Un snapshot illisible, ou dont le nom de fichier ne correspond pas à la table qu'il décrit, arrête la commande : rien n'est écrit. |
| 2026-11-16 | 45 | La migration générée utilise `DROP TABLE IF EXISTS`. |
| 2026-11-16 | général | `makemigrations` contrôle la base en premier (voir « Orientation générale »). |
| 2026-11-16 | général | Les snapshots sont gardés pour le moment ; à revoir une fois le rework fini. |
| 2026-11-16 | général | Pour le moment, les snapshots sont mis à jour à l'application par `migration up`, plus à `makemigrations`. |
| 2026-11-16 | général | Le journal du builder sert de sauvegarde inversée quand une migration échoue : il remet les snapshots d'avant. |
| 2026-11-16 | général | Suivre le chemin de Django : historique en fichiers, calcul d'après les fichiers, la base donne sa position et sert au contrôle. |
| 2026-11-16 | MySQL | Compensation par le journal : instructions exécutées une par une, destructrices en dernier ; panne avant la première destructrice = on annule, après = on continue d'après le journal. |
| 2026-11-16 | historique | L'état de chaque table après chaque migration est gardé dans `migration/src/history/{table}/{date}.rs` (forme reprise de l'ancien `applied/`). Le reste de la forme est à trancher, voir le §10 de l'évaluation. |
| 2026-11-16 | historique | Chaque entrée note celle dont elle part. |
| 2026-11-16 | historique, cas 4 | Table supprimée : tout ce qui la concerne dans `history/` est supprimé, par `migration up`, une fois la migration `drop` appliquée ; tant qu'elle ne l'est pas, l'historique reste et permet le retour arrière. |
| 2026-11-16 | historique, cas 48 | `extend!{}` : même rangement `history/{table}/`, greffé derrière la migration du framework que Runique connaît ; l'entrée ne garde que les colonnes greffées. |
| 2026-11-16 | historique | `makemigrations` n'affiche rien sur la connexion, sauf une erreur si elle est impossible. |
| 2026-11-16 | historique | Pas d'empreinte du contenu dans les entrées : un dev qui retouche l'historique à la main en est responsable. |
| 2026-11-16 | historique | Projets existants : `makemigrations` convertit en une fois ; la première entrée de chaque table part de son état actuel. |
| 2026-11-19 | général | Base injoignable : `makemigrations` renvoie une erreur, pas de repli sur les fichiers. Il contrôle donc la connexion dès l'entrée. |
| 2026-11-19 | général, cas 10 et 22 | Base déjà peuplée et `entities/` vide : `makemigrations` pose la question en console, puis recrée ce qu'il faut à partir de la base (idée de Sébastien). Fait partie de la 4.1 : la version sort complète, sans échéance, pas de report à une version suivante. Le détail reste à définir (questions Q14 à Q20 de `diagramme/flux/makemigrations-4-1-chantier.md`). |
| 2026-11-19 | adoption | Les modèles recréés à partir d'une base passent par l'AST de `runique_dsl`. Ce que la base ne dit pas (un e-mail, par exemple) est écrit dans son type simple, et un message « mettez à jour vos entités » liste ces cas. |
| 2026-11-19 | général | `makemigrations` exige une base, c'est voulu ; ses tests tournent sur les trois moteurs avec les bases Docker déjà en place. |
| 2026-11-19 | cas 33, enums | Avant de supprimer un type enum PostgreSQL, la migration demande à la base si une colonne l'utilise encore, et le laisse si oui ; jamais de cascade. Règle unique pour toutes les suppressions de type, celles qui existent et celles à écrire. |
| 2026-11-19 | enums | Deux modèles qui déclarent un enum du même nom avec des valeurs différentes : erreur à `makemigrations`. Même nom et mêmes valeurs : autorisé, type partagé. |
| 2026-11-19 | général, cas 44 | Tant qu'une migration générée est écrite et pas appliquée, `makemigrations` refuse de continuer. Il n'applique jamais rien lui-même : c'est `migration up` qui applique. Une commande retire la dernière migration en attente. Le cas 44 se règle par ce retrait. |
| 2026-11-19 | général | Le refus « migration en attente » ne vise que les migrations générées. Les cinq du framework n'y comptent pas : elles se reconnaissent à leur nom fixe, que `seaql_migrations` inscrit tel quel (observation de Sébastien). |
| 2026-11-19 | cas 19 | Un `model!{}` mis en commentaire met sa table en pause : rien n'est généré pour elle et rien n'est supprimé. Seul un modèle absent mène à la suppression. Comportement à documenter clairement, que le code suit strictement. (Claude avait d'abord noté l'inverse, « commenté = retiré » : corrigé par Sébastien.) |
| 2026-11-19 | général | Deux devs sur la même table : `makemigrations` refuse et affiche les tables concernées. Pas de commande de réunion. |
| 2026-11-19 | général, cas 18 à 21 | Option `managed: false` dans le `meta` d'un modèle (nom de Django, vrai par défaut) : `makemigrations` ne génère et ne supprime rien pour sa table. C'est la seule façon de mettre une table en pause : un modèle que le lecteur ne voit pas (commenté, sous-dossier, `mod.rs`, fichier renommé) est absent, donc sa table est proposée à la suppression, bloquée sans `--force`. Remplace la règle « commenté = en pause » notée plus haut le même jour. |
| 2026-11-19 | général | Un modèle = une table, seule source de vérité. Deux modèles pour la même table restent une erreur, que l'un soit en pause ou non. (Claude avait proposé d'en autoriser plusieurs si un seul est géré : écarté.) |
| 2026-11-19 | cas 10, 22 | Table que Runique a migrée et qui n'a plus d'entité : suppression proposée (liste, `--force`), y compris quand c'était la dernière. Entité avec `managed: false` : rien. Entités en double : erreur. Runique ne voit que les tables passées par ses migrations ; les autres tables de la base sont ignorées. Le refus « même avec `--force` » proposé par Claude pour le cas 22 est abandonné. |
| 2026-11-19 | history et snapshot | Deux choses distinctes (définitions de Sébastien). `history/` : les modifications à appliquer, une copie par table des fichiers de migration déclarés dans `lib.rs`, écrite par `makemigrations`. Snapshot : l'état d'une table après un `migration up` ou `down` réussi, retranscription de l'entité, écrit par `migration up`. Les lignes du 2026-11-16 qui parlent de « l'état de chaque table gardé dans `history/` » et de snapshots convertis en première entrée viennent d'une lecture de Claude : elles sont fausses. Les trois décisions du matin du 16 sur les snapshots tiennent. |
| 2026-11-19 | snapshot | Pas de fichier d'attente ni de contrôle supplémentaire : à `migration up` réussi, le snapshot est retranscrit depuis l'entité telle qu'elle est. Si le modèle a changé entre `makemigrations` et `migration up`, le résultat est faux : c'est à la documentation de le dire clairement, Runique ne peut pas couvrir tous les cas. |
| 2026-11-19 | Q3, remplace le refus décidé le matin | `makemigrations` génère par-dessus les migrations en attente : l'état de départ est le snapshot plus les modifications de `history/` pas encore appliquées, donc pas de doublon. Il ne refuse que si une suppression de table est en attente et que son modèle est revenu. La commande de retrait reste, pour ce cas. |
| 2026-11-19 | Q4 | Aucun contrôle des données avant d'appliquer : la base renvoie déjà l'erreur, puis c'est transaction ou compensation. |
| 2026-11-19 | Q6 | Transaction atomique partout où c'est possible (PostgreSQL, SQLite si l'essai passe), une migration à la fois ; sinon compensation (MySQL), avec un message clair sur ce qui n'a pas pu être compensé. |
| 2026-11-19 | Q16, adoption | Toutes les tables sont reprises. La documentation doit dire que toute reprise est à revoir à la main pour adapter les modèles à Runique (un champ texte à repasser en `email`, par exemple). |
| 2026-11-19 | Q29 | Aucune marque dans le snapshot. « Pas encore appliquées » se lit sur la base connectée. Runique est piloté par un dev : générer sur une base qui n'a pas reçu les migrations du projet est de sa responsabilité, la documentation dit d'appliquer d'abord. (Claude avait proposé une marque dans le snapshot : écarté.) |
| 2026-11-19 | Q13, Q19 | Tables déjà là sans inscription : `migration up` rejoue les modifications en interne ; si les tables et colonnes de la base correspondent, il inscrit les migrations à la date du `migration up`. Une différence de type donne une erreur ou un message clair. Si la base ne correspond qu'en partie, Runique ne fait rien : au dev de recréer sa base et d'accorder ses entités. C'est aussi ce qui inscrit le point de départ d'une base adoptée. |
| 2026-11-19 | Q15, adoption | `makemigrations` écrit les modèles, les migrations de création, leurs copies dans `history/` et `lib.rs` ; pas les snapshots, retranscrits au `migration up` suivant. |
| 2026-11-19 | Q17, adoption | Pas de suggestion de type ni d'avertissement à part : revue à la main, documentée. C'est au dev de remettre `password` sur son champ ; il entre alors dans les colonnes secrètes que l'admin n'affiche pas. |
| 2026-11-19 | Q25, enum | Valeur retirée d'un enum : Runique regarde s'il reste des lignes qui la portent. S'il n'y en a pas, il retire la valeur. S'il y en a, il pose la question en console, par quelle valeur les remplacer ; si le dev ne choisit rien, aucune modification n'est appliquée. |
| 2026-11-19 | Q14 | Une table de la base sans modèle, que Runique n'a pas migrée, est ignorée sans message : les tables mortes sont de la responsabilité du dev. Confirme la règle du matin. |
| 2026-11-19 | Q20 | Sans console, une commande qui devait poser une question s'arrête avec une erreur. Vérifié : `dialoguer` renvoie déjà une erreur quand il n'y a pas de terminal, il reste à donner un message qui explique quoi faire. |
| 2026-11-19 | Q5, SQLite | SQLite ne sait pas modifier une colonne : c'est traité comme un élément destructeur (bloqué sans `--force`, avec le risque expliqué), et la table est reconstruite. |
| 2026-11-19 | Q21, adoption | Deux builders, chacun sa variable et son dossier : un pour `src/entities/`, un pour `migration/src/` (proposition de Sébastien). Précision de Claude, à confirmer : les entités d'abord ; si le second lot échoue, il reste des modèles sans migration, que le `makemigrations` suivant traite comme des tables à créer, donc l'état se répare de lui-même. |
| 2026-11-19 | Q27, précision | Le garde existe déjà pour une colonne qui passe de facultative à obligatoire (« requires a default or backfill », `destructive_changes.rs`). Il est à étendre à une colonne **ajoutée** obligatoire et sans valeur par défaut, que rien ne signale aujourd'hui. |
| 2026-11-19 | adoption, typage strict | L'adoption reconstruit un semblant de modèle : les champs sont écrits avec le type que la base donne, recopié tel quel (`email: varchar`). `runique_dsl`, strict sur les types, les refuse tant que le dev ne les a pas repris à la main. La revue est donc forcée par le lecteur lui-même, pas seulement par la documentation. Conséquences relevées par Claude, à confirmer : la correspondance des types par moteur n'est plus à écrire pour l'adoption ; le brouillon ne peut pas passer par l'AST, qui ne connaît que les types de Runique ; l'adoption s'arrête après avoir écrit les entités, et les migrations viennent au lancement suivant, par le déroulé normal. |
| 2026-11-19 | adoption, commande à part | La reprise d'une base existante est une commande de la CLI, pas un déclenchement dans `makemigrations` (remplace la décision du matin et sa question en console). Elle parcourt toutes les tables et crée les entités ; le dev passe derrière, les adapte à Runique, lance `makemigrations` puis `migration up` ; `seaql_migrations` se crée et à partir de là tout est propre. `makemigrations` n'a donc plus aucun cas particulier : il ne voit que les tables qu'il a migrées. Le nom de la commande reste à choisir. |
| 2026-11-19 | types, reprise et Q13 | Le contrôle des types se fait dans le modèle : la macro refuse un type qu'elle ne connaît pas (`varchar`), donc le dev choisit chaque type lui-même. Runique ne compare pas les types du modèle à ceux de la base ; à `migration up`, seule l'existence des tables et des colonnes est vérifiée. Aucune correspondance des types par moteur n'est à écrire en 4.1. Un type valide mais mal choisi relève de la revue à la main, à dire dans la documentation. |
| 2026-11-19 | `runique import-db` | Nom de la commande de reprise d'une base existante : `runique import-db`. Elle sort de la 4.1 et viendra en 4.2 ou 4.3. La documentation doit dire clairement que le typage est à refaire à la main : les types écrits sont ceux de la base, et aucun modèle ne passe tant que le dev ne les a pas repris. La 4.1 garde ce qui lui servira : `migration up` inscrit sans exécuter quand les tables existent déjà. |
| 2026-11-20 | Q6, revu | Tout ou rien sur le lancement entier de `migration up`, et non plus une migration à la fois (ce « une à la fois » venait de la recommandation de Claude, cochée dans le fichier de questions le 19). Si une migration échoue, aucune de celles du lancement ne reste appliquée. Le snapshot est donc retranscrit depuis l'entité une fois tout le lancement réussi, comme décidé. |
| 2026-11-20 | journal à `migration up` | Le journal sert en cas de coupure de courant : c'est une sécurité entre le moment où la base a validé et celui où le snapshot est écrit. Au lancement suivant, Runique lit le journal, demande à `seaql_migrations` si les migrations sont inscrites, et termine l'écriture du snapshot si oui. (Claude l'avait jugé redondant : faux, retiré.) |
| 2026-11-20 | contrôle de `history/` | Le contrôle ne va que dans un sens : toute entrée de `history/` doit avoir sa migration dans `lib.rs`, pas l'inverse. Les migrations d'une table supprimée, et celles d'un projet qui existait avant `history/`, restent dans `lib.rs` sans copie, et c'est normal. |
| 2026-11-20 | rappel | « Chaque entrée de `history/` note celle dont elle part » (décision du 16) tient toujours : c'est ce qui permet de voir deux devs sur la même table. Claude l'avait oublié en refaisant les diagrammes. |
| 2026-11-20 | faits testés, par moteur | Test fait le 2026-11-20 (transaction, création de table, ajout de colonne, annulation) : PostgreSQL 16 et SQLite 3.46 défont les changements de schéma ; MySQL 8.4 les garde, la transaction n'y protège rien. Conséquences : tout ou rien par une transaction autour du lancement sous PostgreSQL et SQLite ; compensation sous MySQL seulement, sans l'enveloppe de transaction (une instruction de schéma la fermerait). Deux chemins distincts à `migration up`. |
| 2026-11-20 | SQLite, reconstruction de table | Testé : SQLite ignore la coupure des clés étrangères demandée à l'intérieur d'une transaction. L'ordre est donc imposé : couper les clés étrangères, ouvrir la transaction, reconstruire, vérifier les clés, valider, les rétablir. |
| 2026-11-20 | détection du moteur | Rien à écrire : `get_database_backend()` sur la connexion ouverte (SeaORM, exposé par `ADb`, [db/adb.rs](runique/src/db/adb.rs)) dit le moteur réel ; `DatabaseEngine` ([db/engine.rs](runique/src/db/engine.rs)) le déduit de la configuration avant la connexion. |
| 2026-11-20 | code par moteur | L'enum du moteur a une variante par moteur, chacune sous sa feature (les trois avec `all-databases`). La détection lit le `.env`, compare aux features compilées et renvoie la variante ou le message d'erreur actuel. Le code propre à un moteur est rangé dans son fichier, sous la même feature : `db/engine/` avec `mod.rs`, `postgres.rs`, `mysql.rs`, `sqlite.rs`, et un contrat commun (trait) que les trois fichiers remplissent. À prévoir : rupture sur l'enum publique (CHANGELOG), tests lancés sous chaque feature seule (aujourd'hui la CI les compile sans les exécuter), conversion depuis le type de SeaORM. |
| 2026-11-20 | plus de dossier `history/` | Le fichier de migration contient déjà le `up` et le `down` : en garder une copie par table ferait deux sources pour la même chose. Le dossier `history/` est abandonné, pour la raison qui avait fait retirer `applied/` à la 3.0. Le fichier de migration est la seule source du `up` et du `down` ; le snapshot reste l'état de la table. Sont donc sans objet : la copie des migrations, la suppression d'un dossier d'historique (table supprimée, `down`), le contrôle « toute entrée de `history/` a sa migration ». « Chaque migration note celle dont elle part » se met dans le fichier de migration. |
| 2026-11-20 | compensation sous MySQL | PostgreSQL et SQLite passent par l'atomicité de la transaction ; MySQL non, et sa compensation s'adapte : elle ne peut pas se servir du `down` entier d'une migration, qui défait tout, alors qu'une panne en cours de route n'a fait qu'une partie des instructions. Elle défait les instructions déjà faites, une par une, d'après le journal, avec l'inverse que chaque instruction porte dans le fichier de migration. |
| 2026-11-20 | cas 23, CLI | Pas de garde en plus contre un mauvais dossier `entities` : `makemigrations` se lance à la racine du projet en cours. Une option `-p` (un seul chemin, celui du projet) est prévue pour une version ultérieure de Runique, pas pour la 4.1. |
| 2026-11-20 | cas 30, 31, 34 | D'accord tant que le flux ne casse rien ; sinon on réfléchit à une autre façon de faire. |
| 2026-11-20 | cas 35, 48, `extend!{}` | Les tables du framework ne sont jamais supprimées. Les colonnes ajoutées par un `extend!{}` sont gérées : les retirer du bloc les retire de la base, comme tout retrait de colonne. Sébastien est contre le fait de les laisser sans gestion. |
| 2026-11-20 | cas 38, snapshots | Seul le snapshot d'une table supprimée est retiré, quand la suppression est appliquée. |
| 2026-11-20 | cas 41, 43, 47, `--force` | `--force` écrit les éléments destructeurs dans les fichiers de migration : en le validant, le dev en accepte les conséquences. |
| 2026-11-20 | cas 46 | `migration down` d'une suppression : la table revient vide, le fichier généré est supprimé, le snapshot suit, l'entité n'est pas remise en état (déjà tranché), et la documentation le dit. |
| 2026-11-20 | `extend!{}`, snapshot | Les tables de Runique ne sont jamais supprimées (ça irait contre la gestion de l'admin). Les colonnes d'un `extend!{}` sont gérées. Fait du code : le snapshot d'un `extend!{}` (`snapshots/runique/TABLE.rs`) ne contient que les colonnes ajoutées ; une colonne retirée d'un bloc qui existe encore est déjà détectée ; un bloc retiré en entier ne l'est pas, parce que la boucle ne parcourt que les blocs présents. À faire : détecter le bloc retiré, retirer ses colonnes (destructeur, `--force`), et supprimer son snapshot devenu vide une fois la migration appliquée. |
| 2026-11-20 | `runique migration down`, une seule commande | Pas de commande de retrait à part : `runique migration down` enveloppe SeaORM et fait les deux. Il vise toujours la migration la plus récente du projet. Si elle n'est pas appliquée, il supprime seulement son fichier et sa ligne de `lib.rs`. Si elle l'est, il la défait en base par SeaORM, ramène le snapshot, puis supprime le fichier et la ligne. Fait vérifié : le `down` de SeaORM, laissé seul, défait la dernière migration **appliquée** et ignorerait une migration en attente posée par-dessus ; c'est l'enveloppe qui garde l'ordre. |
| 2026-11-20 | SQLite, clés étrangères | Une clé étrangère ajoutée ou retirée sur une table existante sous SQLite suit le chemin de ce moteur : le code s'adapte au moteur par son enum, donc la table est reconstruite, comme pour une colonne modifiée. |
| 2026-11-20 | `migration down`, transaction | Chaque `down` est encapsulé lui aussi, selon le moteur : transaction sous PostgreSQL (SeaORM le fait déjà) et sous SQLite (à faire par Runique, SeaORM n'en ouvre pas), compensation sous MySQL. Même règle que pour `up`. |
| 2026-11-20 | `rename_to`, fichiers | Le renommage se fait dans le fichier courant de la table : pas de second fichier. Le snapshot de la table suit, sous son nouveau nom, sans qu'il reste de fichier à l'ancien. Documentation stricte là-dessus. |
| 2026-11-20 | élément visé par d'autres | Si une table pointe sur ce qu'on veut modifier ou supprimer : avertissement, modification impossible. Si aucune ne pointe dessus : suppression. Avec `--force` : les autres tables sont mises à jour sur le changement. |
| 2026-11-20 | journaux | `makemigrations` est encapsulé dans le builder et a son propre journal ; `migration up` a le sien. |
| 2026-11-20 | changement de type | Un changement de type est destructeur : il passe obligatoirement par `--force`, et il est alors réellement généré dans les fichiers de migration (aujourd'hui seulement un commentaire). |
| 2026-11-20 | tag « en attente » et cumul | Une migration générée porte un tag « en attente » en tête de fichier (pour ne pas avoir à lire le fichier en entier) ; `migration up` le retire quand le lancement a réussi. Quand `makemigrations` est relancé alors que des migrations sont en attente, il ne les empile pas : il repart du snapshot, recalcule l'écart jusqu'au modèle, et remplace les migrations en attente par ce résultat, sans doublon. Une migration en attente retouchée à la main est écrasée, et la documentation le dit : garder la retouche créerait un écart entre les entités et la base. Remplace : la génération par-dessus (Q3 du 19), le refus sur une suppression en attente dont le modèle est revenu (le cas se règle tout seul), et toute marque dans le snapshot. |
| 2026-11-20 | marqueurs reconnus partout | Le tag « en attente » (`// runique: waiting migrate up`) et le marqueur des longueurs d'un snapshot sont reconnus où qu'ils soient dans le fichier, à condition d'être seuls sur leur ligne (Sébastien : « il faudrait chercher le marqueur seul, si un dev décide de le mettre à la fin »). Runique continue de les écrire en première ligne. Remplace la raison donnée pour « en tête de fichier » : le fichier est lu en entier de toute façon. |
| 2026-11-21 | connexion, P4 (ancien tableau de connexion) | Fermé sans décision à prendre. Constat vérifié dans SeaORM 2.0.4 (`Database::connect`) : une adresse illisible rend une erreur (« cannot be parsed ») avant toute connexion, elle ne fait pas paniquer le programme comme l'ancien tableau le notait à la ligne S8 ; Runique masque déjà le mot de passe dans ce message. Un mot de passe avec `/`, `#` ou `?` non encodé dans `DATABASE_URL` donne donc une erreur et un arrêt, pas une connexion ailleurs. Rien à réparer ni à deviner : la voie « réparation » tombe. Reste à faire, déjà décidé le 2026-11-15 : encoder l'utilisateur et le mot de passe quand Runique fabrique l'adresse depuis les clés `DB_*`, et le dire dans la documentation. À vérifier contre le code actuel avant de coder : chaque autre ligne de l'ancien tableau. |
| 2026-11-21 | connexion, rappel : « incomplète » est DÉFINIE | La définition a été donnée par Sébastien le 2026-11-15 (17 h 25 à 17 h 36) : il a demandé le tableau de tous les cas (base précisée ou non, mot de passe, utilisateur, nom de base réclamés ou non), puis l'a validé. Une adresse est incomplète dans quatre cas seulement : début non reconnu, illisible, serveur sans nom de base, SQLite sans fichier ; tout le reste est tenté et un échec suit le cas A. Ses réponses du même jour : S3 à S5 acceptées et à documenter comme reprise du comportement de SeaORM ; P1 : un refus est une erreur de connexion ; P2 : barre finale sans nom = dossier pour SQLite (la connexion échoue), incomplète pour un serveur ; P4 : ce qui vient de Runique, Runique le règle ; P7 : on réessaie par les clés. La section 6 de l'ancien fichier (« en discussion ») n'avait pas été remise à jour après ces réponses : NE PAS s'y fier. Plus rien n'est à trancher sur la connexion. |
| 2026-11-21 | code par moteur, remplace la décision du 2026-11-20 | Le moteur se lit sur la connexion de SeaORM (`get_database_backend()`, type `DbBackend` à trois variantes), y compris dans une migration (`manager.get_connection()`). Pas d'enum de Runique à variantes sous feature, pas de dossier `db/engine/` à quatre fichiers avec contrat commun, pas de rupture sur `DatabaseEngine` (proposition de Sébastien : « et si au lieu de l'enum on utilisait la connexion de sea ? »). Les moteurs se regroupent opération par opération dans un `match` (`Postgres \| MySql` d'un côté, `Sqlite` de l'autre pour le retrait de clé). Reste de la décision du 20 : le code propre à un moteur va dans son fichier quand il y en a vraiment, et les tests sont lancés sous chaque moteur seul. |
| 2026-11-21 | SQLite, clé à retirer ou ajouter sur une table existante | En attendant la reconstruction de table : la branche SQLite refuse avec un message clair, rien n'est appliqué. Le fichier de migration généré appelle une fonction de Runique, pour que la reconstruction vienne plus tard remplir cette branche sans régénérer les migrations (« on verra plus tard pour éventuellement le faire évoluer autrement »). Motif, essayé le même jour : sous SQLite, supprimer une table visée par une clé en `ON DELETE CASCADE` efface les lignes de la table qui reste. |
| 2026-11-21 | table supprimée encore visée par un modèle qui reste (cas 26) | Confirmé par Sébastien : si le modèle qui reste garde son `belongs_to` vers le modèle retiré, « c'est au dev de rendre ses modèles cohérents ». `makemigrations` s'arrête en erreur à la lecture des modèles, avec ou sans `--force` (comportement existant). `--force` ne retire pas la clé d'un modèle qui la garde. Il ne « force la cohésion » que dans le cas 27 : le `belongs_to` a été retiré du modèle, donc la clé est retirée de la table qui reste, puis la table visée est supprimée. La décision du 2026-11-20 « élément visé par d'autres » se lit ainsi pour une table. |
| 2026-11-20 | exclusion entre commandes | `makemigrations` et `migration up` ne doivent pas pouvoir tourner en même temps. Fait du code : aucun verrou n'existe aujourd'hui entre deux commandes (limite déjà notée dans l'évaluation), et un journal n'est ouvert que pendant la phase d'écriture ; il faudra un verrou tenu pendant toute la durée de la commande. Couvre aussi le cas d'un journal resté ouvert après une coupure : l'autre commande refuse. |
| 2026-11-20 | Q27, champ obligatoire ajouté | Pas de valeur par défaut appliquée d'office. `makemigrations` propose un choix en console (`dialoguer`) : donner une valeur pour les lignes existantes, ou ne rien faire. « Ne rien faire » arrête toute la commande : rien n'est écrit, pas même les autres changements, et le message dit qu'aucun changement n'a été appliqué. Une erreur fait tout rater, c'est le tout ou rien du lot. |
| 2026-11-22 | renommage, remplace Q12 du 2026-11-19 sur le mot, garde le nettoyage du fichier | Un seul mot, `renamed_from`, aux trois endroits : colonne (existant), champ liste (`styles: checkbox [enum(Genre), renamed_from: "genres"]`) et table (`table: "boulangerie", renamed_from: "magasin"`). À gauche le nom d'arrivée, entre guillemets l'ancien. Le dev renomme lui-même son champ ou sa table et son code ; Runique ne change jamais un nom dans le fichier. À `migration up` réussi, Runique retire du fichier du modèle les `renamed_from` qui viennent d'être appliqués, dans le même lot que les snapshots (choix initial de Sébastien : le fichier ne reste pas pollué). Index et clés étrangères dont le nom contient le nom de table sont recréés sous le nouveau nom dans la migration de renommage, y compris les clés des tables qui pointent dessus (Sébastien : « les index sont à mettre à jour en db », confirmé pour les clés). Écartés ce jour : `rename_to` (obligerait à changer un nom dans le fichier, le code du dev resterait sur l'ancien), reconnaissance de la table par le nom du modèle noté dans le snapshot (proposition de Claude, ferait deux façons de renommer), lecture des vrais noms en base pour ce morceau. |
| 2026-11-22 | renommage, retrait du `renamed_from` : un seul lot | Le journal du lot des snapshots refusait les fichiers hors de `migration/src`. Choix de Sébastien : un seul lot quand même, le journal couvre un second dossier (celui des entités), donné de nouveau par la commande à la réparation et jamais écrit dans le journal. Écarté : un second lot sans journal juste après celui des snapshots (proposition de Claude). Sont retirés tous les `renamed_from` du modèle ou du bloc `extend!{}` dont la table vient d'être migrée, colonnes comprises ; si le fichier obtenu ne se relit pas jeton pour jeton comme l'ancien moins les directives, il est laissé tel quel et un message le dit. |
| 2026-11-23 | clé primaire et alias `Pk` : `makemigrations` suit la feature du projet | Constat : `Pk` est résolu par `#[cfg(feature)]` dans `runique_dsl`, donc par les features du binaire `runique` installé pour `makemigrations`, et par celles du projet pour la macro ; les deux peuvent diverger sans message. Sébastien : « il faudrait donc que makemigration soit connecté à la feature du cargo du projet », comme la feature `pk-uuid` du projet pilote déjà la macro et SeaORM (`sea-orm/with-uuid`). Accord sur l'ordre proposé par Claude : 1) `makemigrations` lit les features de `runique` réellement activées dans le projet et résout `Pk` avec ; 2) ensuite seulement, détection d'un changement de nom ou de type de clé primaire par rapport au snapshot, refusé avec la liste des tables. La vraie migration d'une clé primaire attend le changement de type et la reconstruction SQLite. Le moyen de lire les features : validé par Sébastien le 2026-11-23 (« on passe à la lecture des features non pas via runique mais via le projet du dev ») ; la CLI demande à cargo (`cargo metadata` limité à la plateforme de la machine, proposition de Claude) et sait produire les trois types sans feature de compilation. L'installation et les features de moteur ne changent pas. Écartés ce jour : faire tourner `makemigrations` dans une compilation du projet (fermerait la porte à une CLI autonome), et la crate `runique-cli`, repoussée (« une 4.3 peut-être »). |
| 2026-11-24 | enums, remplace la décision du 2026-11-19 | Un nom d'enum ne peut être déclaré qu'une fois dans le projet (Sébastien : « accepter un seul enum du même nom ne serait pas plus propre ? »). Deux modèles, ou un modèle et un bloc `extend!{}`, qui déclarent un enum du même nom : erreur à `makemigrations`, que les valeurs soient les mêmes ou non, `--force` ou non. Plusieurs champs d'un même modèle sur le même enum restent permis. Faits du code : un modèle ne peut pas utiliser l'enum d'un autre (`find_enum` ne cherche que dans le modèle), donc le « type partagé » du 19 n'existait qu'en recopiant la liste ; la migration ne crée le type que s'il n'existe pas, donc le second modèle recevait en silence les valeurs du premier. Les tables en pause (`managed: false`) restent hors du contrôle, comme de tout le reste. Pour plus tard (Sébastien) : un enum déclaré une fois dans son fichier (`enum.rs`) et utilisé par plusieurs modèles ; la règle d'aujourd'hui ne l'empêche pas, c'est toujours une seule déclaration. |
| 2026-11-19 | `migration down` | Une migration dont la ligne est retirée de `seaql_migrations` n'a plus rien à faire dans `lib.rs` (principe de Sébastien, qui voit le comportement de SeaORM comme un manque). Un `down` réussi supprime donc la migration générée, sa copie dans `history/` et sa ligne dans `lib.rs`, et le snapshot revient à l'état d'avant. L'entité ne suit pas : c'est au dev de reprendre son modèle ou de régénérer. À documenter : une migration appliquée sur une autre base, ou retouchée à la main, est perdue par cette suppression. |
| 2026-11-19 | `history/` et `down` | Une entrée de `history/` contient le `up` et le `down` de la migration. `migration down` prend son `down` dans `history/` ; le snapshot d'avant s'obtient en appliquant ce même `down` au snapshot actuel. Aucun ancien snapshot n'est gardé à part. (Claude avait proposé d'en ranger une copie dans l'entrée : inutile, retiré.) |
| 2026-11-19 | `down`, précision 1 | Quand un `down` réussit, la dernière entrée de `history/` est supprimée en entier, son `up` comme son `down`. Le `makemigrations` suivant reprend alors proprement, et le suivi reste cohérent avec SeaORM. |
| 2026-11-19 | `down`, précision 2 | Sur un `down`, les entités ne sont jamais remises à leur état d'origine. Runique ne peut pas le faire proprement, donc il n'y touche pas. La documentation doit être stricte là-dessus. La règle ne vaut que pour un `down`, pas pour un `up`. |
| 2026-11-19 | découpage | 4.1 = tout `makemigrations`, adoption d'une base comprise, sans le plusieurs-à-plusieurs ; 4.2 = le plusieurs-à-plusieurs. (Fichier de questions, A1.) |
| 2026-11-19 | Q22 | Le générateur passe à une liste d'instructions dont chacune connaît son inverse et sait si elle est destructrice ; un retour arrière reprend à partir d'un rang donné de la liste (idée de Sébastien). |
| 2026-11-19 | Q27 | Champ obligatoire ajouté à une table déjà remplie : `makemigrations` pose la question en console et demande la valeur à mettre dans les lignes existantes, comme Django. `dialoguer`, déjà utilisé par la création du superuser, sert à ça. |
| 2026-11-19 | Q12 | Le renommage d'une table ou d'un champ liste se déclare par `rename_to` dans le modèle (exemple : table `magasin`, `rename_to` : `boulangerie`). Une fois le renommage fait, Runique met lui-même le fichier du modèle à jour : la table devient `boulangerie` et `rename_to` disparaît. Sans déclaration : suppression + création, bloquées sans `--force`. |
| 2026-11-19 | Q8, cas 44 | Une suppression écrite et pas appliquée bloque le lancement suivant. Cas à couvrir (Sébastien) : le modèle revient après un retour arrière dans l'éditeur ; le message doit alors dire de retirer la suppression en attente, pas de l'appliquer. |
