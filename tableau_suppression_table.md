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
| 10 | Retrait voulu | tous les modèles sont retirés | toutes les tables, ou refus (voir 22) | à trancher |
| 11 | Sans effet | fichier renommé, `table_name` inchangé | rien | existant |
| 12 | Sans effet | un autre modèle reprend le même `table_name` | diff entre l'ancien et le nouveau schéma, pas de `DROP` | règle |
| 13 | Sans effet | `makemigrations` relancé avant d'appliquer le `drop` | rien, le snapshot n'existe plus | règle |
| 14 | Retrait non voulu | `table_name` changé | `DROP` de l'ancienne + `CREATE` de la nouvelle, données perdues | à trancher |
| 15 | Retrait non voulu | `table_name` changé sur un modèle avec champ liste | les deux tables supprimées et recréées, données perdues | à trancher |
| 16 | Retrait non voulu | champ liste renommé | `DROP {table}_{ancien}` + `CREATE {table}_{nouveau}`, valeurs perdues (aucun `renamed_from` pour ces champs) | à trancher |
| 17 | Retrait non voulu | champ liste changé en champ simple | `DROP` de la table liste + colonne ajoutée, valeurs perdues | règle |
| 18 | Retrait non voulu | `menu.rs` déplacé dans un sous-dossier de `entities/` | `DROP TABLE` : le scan ne lit que le premier niveau | à trancher |
| 19 | Retrait non voulu | `model!{}` mis en commentaire | `DROP TABLE` | à trancher |
| 20 | Retrait non voulu | modèle déplacé dans un fichier `mod.rs` | `DROP TABLE` : le scan saute `mod.rs` | à trancher |
| 21 | Retrait non voulu | extension du fichier changée (`menu.rs.bak`) | `DROP TABLE` | à trancher |
| 22 | Mauvais dossier | dossier `entities` sans aucun modèle, snapshots présents | refus, même avec `--force` | proposé |
| 23 | Mauvais dossier | dossier `entities` d'un autre projet, avec ses modèles | toutes les tables supprimées, les autres créées | à trancher |
| 24 | Mauvais dossier | dossier `entities` introuvable | erreur avant tout | existant |
| 25 | Mauvais dossier | un `model!{}` invalide dans le dossier | erreur avant tout | existant |
| 26 | Relations | la table supprimée est visée par le `belongs_to` d'un modèle qui reste | erreur du scan, et la macro ne compile plus | existant |
| 27 | Relations | le `belongs_to` a été retiré du modèle qui reste | `ALTER … DROP FOREIGN KEY` d'abord, `DROP TABLE` ensuite | règle |
| 28 | Relations | la table supprimée vise une table qui reste | `DROP TABLE` direct | règle |
| 29 | Relations | la table supprimée vise une table du framework (`eihwaz_users`) | `DROP TABLE` direct | règle |
| 30 | Relations | deux tables supprimées, l'une vise l'autre | celle qui vise d'abord (ordre inverse de la création) | proposé |
| 31 | Relations | tables supprimées en cycle de clés | la clé qui ferme le cycle retirée avant, sauf sous SQLite (réutilise `cycle_keys`) | proposé |
| 32 | Relations | table supprimée qui se vise elle-même | `DROP TABLE` direct | règle |
| 33 | Relations | table supprimée avec des colonnes enum | `DROP TYPE` après le `DROP TABLE` (PostgreSQL) | proposé |
| 34 | Contenu de `snapshots/` | fichier qui n'est pas un `.rs` | ignoré | proposé |
| 35 | Contenu de `snapshots/` | dossier `snapshots/runique/` (les `extend!{}`) | jamais lu pour les suppressions | proposé |
| 36 | Contenu de `snapshots/` | `.rs` illisible ou qui n'est pas un snapshot | erreur, rien n'est écrit | décidé |
| 37 | Contenu de `snapshots/` | `menus.rs` qui décrit la table `plats` | erreur, rien n'est écrit | décidé |
| 38 | Contenu de `snapshots/` | snapshot `eihwaz_*` à la racine | jamais supprimé | proposé |
| 39 | Contenu de `snapshots/` | snapshot supprimé à la main, modèle retiré aussi | cas 3 : la table reste en base | indétectable |
| 40 | Contenu de `snapshots/` | snapshot supprimé à la main, modèle présent | `CREATE TABLE … IF NOT EXISTS` | existant |
| 41 | `--force` | suppression sans `--force` | liste `menus: DROP TABLE (data loss)`, rien n'est écrit | proposé |
| 42 | `--force` | suppression + changements sans danger dans le même lancement | tout est bloqué ensemble (un seul garde) | existant |
| 43 | `--force` | suppression avec `--force` | migration `drop` écrite, snapshot supprimé | proposé |
| 44 | Base de données | `drop` généré, pas appliqué, puis le modèle est remis | un `create` est généré après le `drop` : à l'application, table supprimée puis recréée vide | à trancher |
| 45 | Base de données | la table n'existe pas en base quand le `drop` passe | `DROP TABLE IF EXISTS`, pour ne pas faire échouer `migration up` | décidé |
| 46 | Base de données | `migration down` après le `drop` | la table revient vide, les données non | proposé (à documenter) |
| 47 | Base de données | la table contient des lignes | perdues, c'est le sens du `--force` | proposé |
| 48 | `extend!{}` | le bloc `extend!{}` d'une table du framework est retiré | rien, les colonnes ajoutées restent en base | existant, hors morceau 1 |
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
| 10, 22 | Refuser quand aucun modèle n'est trouvé : garder ce refus même avec `--force`, au prix du cas « je retire mon dernier modèle » ? | |
| 36, 37 | Un snapshot illisible, ou dont le nom ne correspond pas à sa table : erreur et arrêt ? | oui (2026-11-16) |
| 44 | Le `drop` non appliqué suivi d'un modèle remis : limite à documenter, ou autre chose ? | en cours : voir ce que SeaORM permet de lire en base |
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
