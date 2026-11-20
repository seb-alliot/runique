# makemigrations 4.1 : ce qui existe, ce qui reste à faire

> **Les deux sources (2026-11-20).** Le fichier de migration est la seule source du `up` et du `down` : aucune copie, pas de dossier `history/`. Le snapshot est l'état d'une table après un `migration up` ou `down` réussi, écrit à l'application.

Document de travail du chantier makemigrations 4.1. Code relu le 2026-11-19, branche
`makemigration` (commit `2ac2a986`). Chaque ligne renvoie à un cas de
[makemigrations-4-1-cas.md](makemigrations-4-1-cas.md).

Colonne « Nature » : ce qu'il faut écrire pour la partie à faire.

| Nature | Sens |
|---|---|
| appel | brancher du code qui existe déjà |
| fonction | une fonction nouvelle, sans type nouveau |
| struct | une structure de données nouvelle |
| enum | une liste fermée de cas (voir la section « Types ») |
| générateur | le texte des migrations générées |
| dépendance | une crate à déclarer |

## Ligne 0 : savoir où en est la base

| Cas | Existant | À faire | Nature | Question |
|---|---|---|---|---|
| 0.1 connexion | `DatabaseConfig::from_env` et `connect` ([db/config.rs:143](../../runique/src/db/config.rs#L143), [:262](../../runique/src/db/config.rs#L262)) ; adresse masquée dans les erreurs (`redact_url`, [:418](../../runique/src/db/config.rs#L418)). `makemigrations` ne s'en sert pas : `run` est synchrone ([cli_migration/mod.rs:47](../../runique/src/migration/cli_migration/mod.rs#L47)) | `run` asynchrone et connectée ; erreur si la base ne répond pas ; la règle de lecture de l'adresse (tableau de connexion) n'est pas codée | appel, fonction | — |
| 0.2 position | l'entité `seaql_migrations` de `sea-orm-migration` ; la liste des modules de `lib.rs` se lit déjà (`parse_lib_state`, [lib_file.rs:123](../../runique/src/migration/cli_migration/lib_file.rs#L123)) | lire les noms appliqués, les croiser avec `lib.rs`, classer chaque migration ; exposer en une liste les cinq noms fixes du framework ([auth/migrations_table.rs](../../runique/src/auth/migrations_table.rs)), aujourd'hui écrits chacun dans sa fonction | fonction, **enum** (E1, E2) | — |
| 0.3 contrôles avant tout calcul | aucun | migration inscrite en base inconnue de `lib.rs` ; snapshot illisible ou mal nommé ; un message par cas, dans les 9 langues | fonction, **enum** (E3) | — |
| 0.4 migration générée et pas appliquée | aucun | tag « en attente » en tête du fichier de migration, retiré par `migration up` ; relancé, `makemigrations` repart du snapshot et remplace les migrations en attente par le nouveau calcul (le builder sait déjà supprimer un fichier dans un lot) | fonction | — |
| 0.5 table déjà là sans inscription | `has_table` de `sea-orm-migration` ; les créations sont en `IF NOT EXISTS`, donc rien n'échoue aujourd'hui | dépend de la réponse | appel | Q13 |
| 0.6 deux devs | git signale un conflit sur le snapshot unique | chaque migration note celle dont elle part ; refus avec les tables concernées | fonction | — |
| 0.7 les deux commandes en même temps | aucun verrou entre deux commandes ; `migration up` refuse seulement quand un lot de `makemigrations` est resté interrompu ([migrate.rs:65](../../runique/src/migration/cli_migration/migrate.rs#L65)) | un verrou tenu pendant toute la commande, dans les deux sens | fonction | — |

## Ligne 1 : comparer

| Cas | Existant | À faire | Nature | Question |
|---|---|---|---|---|
| 1.1 table nouvelle | complet : `compute_main_changes` ([plan.rs:21](../../runique/src/migration/cli_migration/plan.rs#L21)), tri ([ordering.rs:8](../../runique/src/migration/changes/ordering.rs#L8)), cycles (`cycle_keys`, [plan.rs:58](../../runique/src/migration/cli_migration/plan.rs#L58)) | rien | appel | — |
| 1.2 colonne ajoutée, retirée | complet (`diff_schemas`, [diff.rs:38](../../runique/src/migration/diff.rs#L38)) | rien | — | — |
| 1.2 nullable, unique, longueur | complet, sauté sous SQLite | voir 3a.2 | — | Q5 |
| 1.2 champ obligatoire ajouté à une table remplie | `ADD COLUMN ... NOT NULL` sans valeur pour les lignes existantes ([columns.rs:4](../../runique/src/migration/generate/columns.rs#L4)) | choix en console (`dialoguer`, déjà utilisé par la création du superuser) : donner une valeur, ou ne rien faire et tout arrêter | fonction, générateur | — |
| 1.2 vers `NOT NULL` | `modify_column` direct ([alter.rs](../../runique/src/migration/generate/alter.rs), étape 4) | remplissage avant | générateur | — |
| 1.2 type changé | bloqué sans `--force` ; avec, un commentaire, et le snapshot enregistre le nouveau type | destructeur, `--force` obligatoire, puis le changement est réellement généré, par le chemin du moteur | générateur | — |
| 1.2 valeur par défaut | pas comparée : seul `has_default_now` l'est ([diff.rs:38](../../runique/src/migration/diff.rs#L38)) | l'ajouter à la comparaison et au générateur | fonction, générateur | — |
| 1.3 colonne renommée | complet (`renamed_from`) | renommée **et** modifiée : la modification est cherchée sous le nouveau nom et jamais trouvée | fonction | — |
| 1.4 enum, valeur ajoutée ou renommée | complet, par moteur ([enums.rs](../../runique/src/migration/generate/enums.rs)) | erreur quand deux modèles déclarent le même nom avec des valeurs différentes ; contrôle avant tout `DROP TYPE` | fonction, générateur | — |
| 1.4 enum, valeur retirée | un commentaire | à définir | générateur | Q25 |
| 1.4 vers ou depuis un enum | PostgreSQL et MySQL ; sauté sous SQLite | voir 3a.2 | — | Q5 |
| 1.5 clés étrangères | comparées par colonne, table et actions ; nom supposé (`fk_constraint_name`, [constraints.rs:10](../../runique/src/migration/generate/constraints.rs#L10)) | lire le nom réel en base | appel (`sea-schema`) | — |
| 1.5 index | comparés par leur nom | comparer aussi par colonnes | fonction | — |
| 1.6 table supprimée | rien : la boucle ne parcourt que les modèles ([plan.rs:26](../../runique/src/migration/cli_migration/plan.rs#L26)) | détection (table migrée par Runique sans modèle, dernier modèle compris), ordre (clés, table liste, cycles), `DROP TABLE IF EXISTS`, garde `--force`, `DROP TYPE` gardé | fonction, **enum** (E4), générateur | — |
| 1.7 renommage de table, de champ liste | rien : `Changes` n'a aucun champ pour ça ([types.rs:69](../../runique/src/migration/types.rs#L69)) ; le `meta` ne lit pas `rename_to` | lire `rename_to`, générer le renommage, faire suivre clés, index, tables liste, snapshot ; mise à jour du fichier du modèle par `migration up` | fonction, **enum** (E4), générateur | — |
| 1.8 `extend!{}` ajouté | complet (`plan_extend_changes`, [plan.rs:188](../../runique/src/migration/cli_migration/plan.rs#L188)) | rien | — | — |
| 1.8 `extend!{}` retiré | rien, les colonnes restent | devient un écart ordinaire | fonction | — |
| 1.9 garde destructif, `--force` | complet (`check_destructive`, [report.rs:29](../../runique/src/migration/cli_migration/report.rs#L29)) | y ajouter la suppression de table | fonction | — |
| 1.9 longueur des identifiants | complet, mais 63 pour tous et un message qui annonce 64 ([name_length_limit.rs:13](../../runique/src/migration/changes/name_length_limit.rs#L13)) | limite par moteur | fonction | — |
| 1.10 table en pause | le `meta` lit sept options, pas `managed` ([parser/meta.rs](../../runique/runique_dsl/src/parser/meta.rs)) | lire `managed`, écarter la table du calcul après le contrôle des doublons, rappel à chaque lancement | fonction | — |
| 1.11 modèle que le lecteur ne voit pas | c'est déjà le comportement : le scan ne lit que le premier niveau, saute `mod.rs`, ignore les commentaires ([scan.rs:32](../../runique/src/migration/cli_migration/scan.rs#L32)) | le documenter clairement ; sa table est proposée à la suppression (cas 1.6) | — | — |

## Ligne 2 : générer

| Cas | Existant | À faire | Nature | Question |
|---|---|---|---|---|
| 2.1 lot normal | complet : plan calculé avant toute écriture, un seul lot (`commit_plan`, [plan.rs:163](../../runique/src/migration/cli_migration/plan.rs#L163)) ; aujourd'hui le lot écrit aussi le snapshot | le snapshot sort du lot, il est écrit par `migration up` ; chaque migration note celle dont elle part | fonction | — |
| 2.2 projet existant | snapshots et migrations existent déjà | rien à convertir | — | — |
| 2.3 coupure pendant l'écriture | complet : `recover` et `interrupted` ([journal.rs:181](../../runique/src/atomic/journal.rs#L181), [:164](../../runique/src/atomic/journal.rs#L164)), refus de `migration up` ([migrate.rs:65](../../runique/src/migration/cli_migration/migrate.rs#L65)) | rien | — | — |
| 2.4 ordre des instructions | ordre fixe en 11 étapes, retraits de colonnes à l'étape 3 ([alter.rs](../../runique/src/migration/generate/alter.rs)) | destructrices en dernier ; `up` et `down` en étapes appariées | **enum** (E6, E7), générateur | Q22 |
| 2.5 suppression écrite, pas appliquée | la suppression de fichier dans un lot (`SourceItems::Remove`, [struct_atomic.rs:35](../../runique/src/atomic/struct_atomic.rs#L35)), pas encore utilisée | couvert par le refus du cas 0.4 | — | — |

## Ligne 3a : restaurer en revenant en arrière

| Cas | Existant | À faire | Nature | Question |
|---|---|---|---|---|
| 3a.1 PostgreSQL | transaction par migration, fournie par `sea-orm-migration` | une transaction passée à SeaORM autour de tout le lancement | fonction | — |
| 3a.2 SQLite, transaction | aucune par défaut | une transaction passée à SeaORM autour du lancement, et autour de chaque `down` | fonction | — |
| 3a.2 SQLite, colonne modifiée | sautée avec un commentaire ([alter.rs:195](../../runique/src/migration/generate/alter.rs#L195)) | destructeur, `--force`, puis reconstruction de la table ; clés étrangères coupées avant d'ouvrir la transaction | générateur | — |
| 3a.2 SQLite, clé sur une table existante | aucun garde, arrêt brutal dans `sea-query` | reconstruction de la table, comme pour une colonne modifiée | générateur | — |
| 3a.3 MySQL | rien : le fichier généré enchaîne ses instructions seul | exécuteur pas à pas, journal d'instructions, reprise, lecture d'existence | fonction, **struct** (journal), **enum** (E6, E7, E8) | — |
| 3a.4 contrôles sur les données | rien | rien : décidé, la base renvoie l'erreur, puis transaction ou compensation | — | — |
| 3a.5 après une migration réussie ou échouée | `migration up` sait si la crate a réussi ([migrate.rs:47](../../runique/src/migration/cli_migration/migrate.rs#L47)) ; aujourd'hui le snapshot est écrit par `makemigrations` | snapshot retranscrit depuis l'entité (donc `migration up` lit `entities/`) ; journal du builder gardé le temps de la migration ; fichier du modèle mis à jour pour un `rename_to` ; snapshot d'une table supprimée | fonction | — |
| 3a.6 `migration down` | passe la main à la crate ([migrate.rs:18](../../runique/src/migration/cli_migration/migrate.rs#L18)) ; SeaORM défait la dernière migration **appliquée** ; aucun fichier n'est touché | envelopper SeaORM : viser la migration la plus récente du projet ; pas appliquée, supprimer fichier et ligne de `lib.rs` ; appliquée, la défaire, ramener le snapshot par son `down`, puis supprimer | fonction (appliquer une liste d'instructions à un état), **enum** (E6) | — |

## Ligne 3b : restaurer en reconstruisant depuis une base

Hors de la 4.1 : commande `runique import-db`, prévue en 4.2 ou 4.3. Rien n'existe dans le code.
Elle demandera : lire le schéma (`sea-schema`, déjà dans `Cargo.lock`), écrire un fichier par table
dans `src/entities/` avec un builder, un fichier d'appel dans `cli/`. Ni correspondance des types,
ni écriture depuis l'AST.

## Types : des enums à écrire, ou autre chose ?

Types qui existent et qui servent : `ParsedSchema`, `ParsedColumn`, `ParsedFk`, `ParsedIndex`,
`Changes` ([types.rs](../../runique/src/migration/types.rs)) ; `ActionAtomic`, `AtomicStep`,
`SourceItems` ([struct_atomic.rs](../../runique/src/atomic/struct_atomic.rs)) ; `Engine`
([init_migrate.rs:19](../../runique/src/migration/cli_migration/init_migrate.rs#L19)) ; `DbBackend`
de SeaORM ; `FormFieldKind` et le reste de l'AST dans `runique_dsl`.

`Changes` est une structure de douze listes plus un booléen `is_new_table`. Le générateur en tire
du texte dans un ordre fixe.

### Enums à écrire

Un enum se justifie là où il y a une liste fermée de cas exclusifs, et où un booléen ou une
chaîne permettrait un état impossible.

| # | Ce qu'il décrit | Cas | Pourquoi un enum | Ligne |
|---|---|---|---|---|
| E1 | la place d'une migration | appliquée, en attente, inscrite en base et inconnue des fichiers | trois états exclusifs, nés du croisement de `lib.rs` et de `seaql_migrations` | 0 |
| E2 | l'état de la base | neuve, à jour, en retard, en avance, peuplée sans inscription | il choisit le chemin : calcul, refus, inscription | 0 |
| E3 | un refus de la chaîne | point de départ absent, deux dernières entrées, migration sans entrée, entrée sans migration, migration inconnue, entrée illisible | un message par cas | 0 |
| E4 | ce qui arrive à une table | créée, modifiée, supprimée (renommée si Q12) | remplace `is_new_table` : un second booléen « supprimée » rendrait possible « créée et supprimée » | 1 |
| E5 | un changement de type | sûr, à convertir, destructeur, impossible | aujourd'hui tout changement de type donne un commentaire | 1 |
| E6 | une instruction de migration | ajout, retrait, modification, renommage de colonne ; ajout, retrait de clé ou d'index ; création, suppression de table ; opérations d'enum | chaque cas porte son inverse | 2, 3a |
| E7 | le caractère d'une instruction | réversible, destructrice | fixe l'ordre et le point de non-retour | 2, 3a |
| E8 | une instruction dans le journal MySQL | en attente, faite, en doute | le pendant de `AtomicStep` pour des instructions | 3a |

E6 est le plus lourd : c'est la forme que prend la migration avant de devenir du texte. Sans lui,
ni « destructrices en dernier » ni la compensation sous MySQL ne se codent sans relire le texte
généré. C'est un changement de structure du générateur, pas un ajout : question Q22.

### Ce qui n'est pas un enum

| Besoin | Forme |
|---|---|
| le lien « part de » d'une migration | une ligne dans le fichier de migration |
| le journal d'instructions MySQL | une structure, sur le modèle du journal du builder |
| savoir quelles migrations touchent une table | le nom du fichier le dit (`m…_create_TABLE_table`, `m…_alter_TABLE_table`), fonctions existantes dans [paths.rs](../../runique/src/migration/paths.rs) |
| le choix de l'état de départ | une fonction pure : entrées d'une table + noms appliqués |
| la lecture de `seaql_migrations`, `has_table`, `has_column`, `has_index` | des appels, tout existe |
| les messages | des clés de traduction, dans les 9 langues |

## Ce que l'exercice fait apparaître

Points qui n'étaient ni dans le tableau des 53 cas ni dans l'évaluation.

| # | Constat | Question |
|---|---|---|
| 1 | **Un type enum PostgreSQL peut servir à deux tables.** Son nom est celui de l'enum, sans la table, et il n'est créé que s'il n'existe pas ([enums.rs:49](../../runique/src/migration/generate/enums.rs#L49)). `DROP TYPE` est généré à trois endroits ([generate/mod.rs:61](../../runique/src/migration/generate/mod.rs#L61), [:228](../../runique/src/migration/generate/mod.rs#L228), [enums.rs:151](../../runique/src/migration/generate/enums.rs#L151)) sans vérifier qu'une autre table l'utilise : PostgreSQL refuse, la migration échoue. Rien n'est généré pour une colonne enum ou une table supprimée. Deux modèles avec un enum du même nom et des valeurs différentes : le second reçoit les valeurs du premier, sans message | décidé le 2026-11-19 : contrôle à l'exécution avant tout `DROP TYPE`, et erreur à `makemigrations` pour un même nom avec d'autres valeurs |
| 2 | **`makemigrations` ne marche plus sans base.** Conséquence de « base injoignable = erreur ». Sous PostgreSQL et MySQL la base doit avoir été créée avant | décidé le 2026-11-19 : voulu |
| 3 | **Le générateur doit changer de forme** (E6) pour porter l'ordre et la compensation | Q22 |
| 4 | **`migration status`** existe ([migrate.rs:25](../../runique/src/migration/cli_migration/migrate.rs#L25)) et devrait montrer l'historique croisé avec la base ; aucun cas ne le couvrait | à ajouter aux cas |
| 5 | **Après un `drop` appliqué puis un `migration down`**, la table revient vide | à documenter ou à traiter |
| 6 | **Les 106 tests sans base** (81 dans `tests_pipeline.rs`, 25 dans `cli_migration/tests.rs`) appellent `run` | décidé le 2026-11-19 : ils tournent sur les trois moteurs, par les bases Docker déjà en place (`docker-compose.yml`, services de la CI, variables `DATABASE_URL_PG` et `DATABASE_URL_MYSQL`) |

## Compte

Le compte « déjà complet / à compléter / entièrement à faire » fait le matin du 2026-11-19
(43 lignes : 7, 19, 17) ne tient plus après les décisions de l'après-midi. Il sera refait quand
le fichier de questions sera rempli.
