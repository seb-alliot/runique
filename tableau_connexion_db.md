# Connexion à la base : tableau de décision


| Source | Contenu |
|---|---|
| `DATABASE_URL` | l'adresse complète, par exemple `postgres://app:secret@db.local:5432/shop` |
| clés individuelles | `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME` |

Règle de départ : on lit `DATABASE_URL` en premier ; sinon on se rabat sur les clés individuelles.

---

## 1. Le tableau complet

| # | `DATABASE_URL` | Clés individuelles | Ce que fait Runique |
|---|---|---|---|
| 1 | complète et bien formée | peu importe | utilise l'adresse, puis se connecte |
| 2 | non vide mais incomplète | remplies | prévient en console, construit l'adresse avec les clés, se connecte. Si la connexion réussit, rappelle que `DATABASE_URL` est incomplète |
| 3 | non vide mais incomplète | partiellement remplies | lancement bloqué, dit ce qui manque |
| 4a | non vide mais incomplète, moteur serveur ou début non reconnu | vides | échec forcé : pas de SQLite par défaut |
| 4b | `sqlite://` sans nom de fichier | vides | le moteur est connu, seul le nom manque : on retombe sur la ligne 7, le fichier SQLite par défaut, avec un avertissement |
| 5 | absente ou vide | remplies | construit l'adresse avec les clés, puis se connecte |
| 6 | absente ou vide | partiellement remplies | lancement bloqué, dit ce qui manque |
| 7 | absente ou vide | vides | SQLite par défaut, comme la documentation l'annonce |

Dans tous les cas où Runique se connecte : **une connexion qui échoue bloque le lancement.** C'est déjà le comportement actuel.

Frontière à retenir : **Runique ne choisit SQLite tout seul que si rien n'est renseigné du tout** (ligne 7), ou si l'adresse elle-même demande SQLite sans donner de fichier (ligne 4b). Après une adresse incomplète qui vise un serveur, il ne devine rien (ligne 4a).

---

## 2. Ce que veut dire chaque mot

### « Absente », « commentée », « vide »

Une ligne du `.env` peut être dans quatre états. Les trois premiers reviennent au même pour Runique :

| Dans le `.env` | Exemple | Ce que Runique voit |
|---|---|---|
| la ligne n'existe pas | | la clé est absente |
| la ligne est commentée | `# DB_USER=app` | la clé est absente : un commentaire n'est jamais lu |
| la ligne existe sans valeur | `DB_USER=` ou `DB_USER=   ` | la clé est absente (à corriger, voir ci-dessous) |
| la ligne existe avec une valeur | `DB_USER=app` | la clé est présente |

Commenter une ligne est donc la façon normale de la désactiver. Le `.env` généré par `runique new` s'en sert : `DB_ENGINE`, `DB_NAME` et `DATABASE_URL` sont actifs, `DB_HOST`, `DB_PORT`, `DB_USER` et `DB_PASSWORD` sont commentés.

| Ce que le développeur commente | Ligne du tableau |
|---|---|
| `DATABASE_URL`, en laissant les clés actives | 5 ou 6 : l'adresse est construite avec les clés |
| les clés, en laissant `DATABASE_URL` active | 1 : l'adresse suffit |
| une clé obligatoire d'un serveur (`DB_USER`, `DB_PASSWORD`, `DB_NAME`) | 3 ou 6 : clés partielles, lancement bloqué |
| une clé facultative (`DB_HOST`, `DB_PORT`) | sans effet : la valeur par défaut est prise |
| tout | 7 : SQLite par défaut |

Point à corriger : aujourd'hui une clé présente mais sans valeur (`DB_USER=`) est prise pour une clé remplie, et l'adresse construite devient `postgres://:@localhost…`. Elle doit compter comme absente, pour `DATABASE_URL` (lignes 5 à 7) comme pour les clés individuelles.

### « Incomplète » (proposition du 15 novembre 2026, à trancher)

Une adresse a six morceaux : `moteur://utilisateur:mot_de_passe@hôte:port/base`.

#### Ce qui se passe quand un morceau manque (lu dans sqlx 0.9, le pilote de SeaORM)

| Morceau absent | PostgreSQL | MySQL |
|---|---|---|
| utilisateur | `PGUSER` de la machine, sinon le nom du compte système | `root` |
| mot de passe | `PGPASSWORD` de la machine, sinon aucun | aucun |
| hôte | `PGHOST` de la machine, sinon `localhost` | `localhost` |
| port | `PGPORT` de la machine, sinon 5432 | 3306 |
| base | `PGDATABASE` de la machine, sinon **une base du nom de l'utilisateur** | **aucune base choisie** : la connexion réussit, puis chaque requête échoue |

Un serveur peut accepter une connexion sans mot de passe, ou sans utilisateur déclaré : ça dépend de son réglage, et Runique ne peut pas le savoir en lisant l'adresse.

#### La règle proposée

Le cas A est déjà décidé : une connexion qui échoue se replie sur les clés. Donc tout ce qui peut s'essayer s'essaie, et c'est le serveur qui tranche. « Incomplète » ne couvre que ce qu'on ne peut pas essayer sans risque.

| # | Adresse serveur | Exemple | Verdict | Pourquoi |
|---|---|---|---|---|
| S1 | tous les morceaux | `postgres://app:secret@db.local:5432/shop` | complète | |
| S2 | sans port | `postgres://app:secret@db.local/shop` | complète | le port du moteur est pris |
| S3 | sans mot de passe | `postgres://app@db.local/shop` | complète, on essaie | certains serveurs l'acceptent ; un refus suit le cas A |
| S4 | sans utilisateur ni mot de passe | `postgres://db.local/shop` | complète, on essaie | même raison |
| S5 | sans hôte | `postgres://app:secret@/shop` | complète, on essaie | `localhost` est pris ; un refus suit le cas A |
| S6 | **sans base** | `postgres://app:secret@db.local` | **incomplète** | la connexion peut réussir sur une autre base que celle voulue (PostgreSQL), ou sur aucune (MySQL) |
| S7 | début non reconnu ou absent | `mariadb://…`, `app:secret@db.local/shop` | **incomplète** | on ne sait pas à quel moteur parler |
| S8 | illisible | port `543a`, `@` en trop, guillemets ou espaces autour | **incomplète** | SeaORM ne rend pas une erreur, il fait paniquer le programme |

| # | Adresse SQLite | Exemple | Verdict | Pourquoi |
|---|---|---|---|---|
| L1 | avec un fichier | `sqlite://data.db` | complète | |
| L2 | avec un fichier et des options | `sqlite://data.db?mode=rwc` | complète | le fichier est créé s'il manque |
| L3 | en mémoire | `sqlite::memory:` | complète | |
| L4 | fichier qui n'existe pas, sans `mode=rwc` | `sqlite://absent.db` | complète, on essaie | la connexion échoue : cas A |
| L5 | **sans fichier** | `sqlite://`, `sqlite:` | **incomplète**, ligne 4b | SQLite ouvrirait une base temporaire, effacée à la fermeture : la connexion réussirait et rien ne serait gardé |
| L6 | avec utilisateur, mot de passe ou hôte | `sqlite://app:secret@db.local/data.db` | complète | SQLite lit tout ce qui suit `sqlite://` comme un chemin de fichier |

Résumé : une adresse est incomplète dans quatre cas seulement, **début non reconnu, illisible, serveur sans nom de base, SQLite sans fichier**. Le reste est tenté, et un échec suit le cas A.

Point à connaître : pour PostgreSQL, les morceaux absents sont complétés par les variables `PG*` de la machine, pas par les clés `DB_*` du `.env`.

Décidé (Sébastien, 15 novembre 2026) : S3 à S5 sont acceptées. Runique reproduit là le comportement de SeaORM, et la documentation doit le dire cas par cas, de façon exhaustive (tableau « ce qui se passe quand un morceau manque » compris).

#### Cas relevés ensuite, à trancher

Vérifiés dans sqlx 0.9, SeaORM 2.0.4 et dotenvy 0.15.7.

| # | Cas | Exemple | Ce qui se passe aujourd'hui | Proposition |
|---|---|---|---|---|
| P1 | un morceau donné en paramètre (PostgreSQL seulement) | `postgres://db.local?dbname=shop&user=app` | le pilote accepte `host`, `port`, `dbname`, `user`, `password` après le `?` | décidé (Sébastien) : on essaie, et un refus est une erreur de connexion, cas A. Le contrôle S6 lit donc `?dbname=` comme une base donnée, sinon l'adresse serait refusée avant l'essai |
| P2 | barre finale sans nom | `postgres://app@db.local/` | lu comme « sans base » | serveur : S6, incomplète. SQLite (`sqlite://media/`) : c'est un dossier, la connexion échoue, cas A (P11) |
| P3 | paramètre inconnu | `?foo=bar` | ignoré (PostgreSQL le signale dans les journaux, MySQL non) ; SQLite le refuse | laisser le pilote faire, documenter |
| P4 | caractère spécial non encodé dans le mot de passe | `/`, `#`, `?` | l'adresse est coupée au mauvais endroit : illisible, ou un autre hôte | décidé (Sébastien) : ce qui vient de Runique, Runique le règle. Par les clés, c'est Runique qui fabrique l'adresse : il encode l'utilisateur et le mot de passe (cas E), le développeur écrit le mot de passe tel quel. Dans `DATABASE_URL`, c'est le développeur qui a écrit l'adresse : reste à trancher entre S8 (avertissement, repli sur les clés) et une réparation par Runique |
| P5 | `$` dans une valeur du `.env` | `DB_PASSWORD=pa$$word` | le lecteur du `.env` remplace `$nom` par une autre variable : la valeur change sans prévenir | documenter : mettre la valeur entre apostrophes |
| P6 | `DATABASE_URL` définie dans le terminal et dans le `.env` | | celle du terminal l'emporte, le `.env` n'écrase jamais une variable déjà définie | documenter ; le message de repli peut dire d'où vient l'adresse |
| P7 | l'adresse et les clés désignent la même base, et la connexion échoue | | cas A : second essai identique, second délai | décidé (Sébastien) : on réessaie quand même. Les deux sources peuvent désigner la même base, et l'essai par les clés montre si l'échec venait de `DATABASE_URL` |
| P8 | plusieurs hôtes | `postgres://h1,h2/shop` | illisible | S8 |
| P9 | début en majuscules | `POSTGRES://…` | accepté : le début est ramené en minuscules | rien à faire |
| P10 | autres écritures SQLite | `sqlite:data.db`, `sqlite:///chemin/absolu.db`, `sqlite://:memory:` | acceptées | L1 ou L3 ; les lister dans la doc |
| P11 | le chemin SQLite est un dossier, ou son dossier parent n'existe pas | `sqlite://media/` | la connexion échoue | cas A |
| P12 | hôte en IPv6 ou connexion par socket | `postgres://[::1]/shop`, `?host=/var/run/postgresql`, `?socket=` pour MySQL | acceptés | S1 à S5 ; documenter |

### « Clés remplies », « partielles », « vides »

Tout dépend du moteur nommé par `DB_ENGINE`.

| Valeur de `DB_ENGINE` | Moteur |
|---|---|
| `postgres`, `postgresql` | PostgreSQL |
| `mysql` | MySQL |
| `sqlite` | SQLite |
| autre chose | erreur, avec la liste des noms acceptés |

La casse et les espaces ne comptent pas.

| Clé | Serveur (PostgreSQL, MySQL) | SQLite |
|---|---|---|
| `DB_USER` | obligatoire | sans objet |
| `DB_PASSWORD` | obligatoire | sans objet |
| `DB_HOST` | facultatif, `localhost` par défaut | sans objet |
| `DB_PORT` | facultatif, celui du moteur par défaut | sans objet |
| `DB_NAME` | obligatoire : le nom de la base | facultatif : le chemin du fichier, `local_base.sqlite` par défaut |

| État | Serveur | SQLite |
|---|---|---|
| remplies | `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_NAME` présents | `DB_ENGINE=sqlite` présent |
| partielles | `DB_ENGINE` présent, une clé obligatoire manque | impossible : rien n'est obligatoire |
| vides | aucun `DB_ENGINE` | aucun `DB_ENGINE` |

Avec `DB_ENGINE=sqlite`, les clés `DB_USER`, `DB_PASSWORD`, `DB_HOST` et `DB_PORT` sont ignorées si elles sont présentes.

---

## 3. Le cas de SQLite

| Situation | Résultat |
|---|---|
| `DB_ENGINE=sqlite` et `DB_NAME=data.db` | le fichier `data.db` |
| `DB_ENGINE=sqlite` sans `DB_NAME` | le fichier `local_base.sqlite` |
| `DATABASE_URL` incomplète et `DB_ENGINE=sqlite` écrit dans le `.env` | avertissement, puis le fichier SQLite : le développeur l'a demandé |
| `DATABASE_URL` incomplète qui vise un serveur, et aucun `DB_ENGINE` | échec forcé : Runique ne prend pas SQLite à sa place |
| `DATABASE_URL=sqlite://` sans nom de fichier, et aucun `DB_ENGINE` | avertissement, puis le fichier `local_base.sqlite` : l'adresse demandait SQLite |
| rien de renseigné du tout | SQLite par défaut |

---

## 4. Cas en plus de ceux du tableau

| # | Cas | Comportement | Statut |
|---|---|---|---|
| A | une adresse correcte dont la connexion échoue | repli sur la construction par les clés, puis on suit la logique de la construction : clés remplies → connexion (échec = lancement bloqué), partielles → lancement bloqué, vides → échec forcé. Prévenu en console | décidé (Sébastien, 2026-11-15) |
| B | `DB_ENGINE` avec une valeur inconnue (`oracle`) | erreur qui liste les moteurs acceptés | existe déjà |
| C | des clés (`DB_USER`…) sans `DB_ENGINE` | erreur « `DB_ENGINE` manque » : sans le moteur on ne peut pas construire l'adresse. (Aujourd'hui : SQLite, les clés ignorées en silence.) | décidé (Sébastien, 2026-11-15) |
| D | le moteur de l'adresse n'est pas compilé dans le binaire | erreur qui nomme la feature à activer | existe déjà |
| E | un mot de passe contenant `@`, `/` ou `:` dans les clés | l'adresse construite doit l'encoder, sinon elle est cassée ou désigne un autre hôte | à corriger |
| F | les deux sources sont renseignées et se contredisent | l'adresse l'emporte ; ce n'est pas un problème | décidé |
| G | `migration up` | la crate de migration ne lit que `DATABASE_URL`. Si Runique s'est rabattu sur les clés, il lui transmet l'adresse retenue, pour que l'application et les migrations visent la même base | à faire avec le reste |

---

## 5. Qui applique cette règle

Une seule règle, à un seul endroit, pour trois appelants :

| Appelant | Ce qu'il en fait |
|---|---|
| l'application | se connecte au démarrage |
| makemigrations | choisit le moteur de la crate de migration qu'il crée |
| `migration up`, `down`, `status` | transmet l'adresse à la crate de migration |

---

## 6. Ce qui reste à décider

1. La définition d'« incomplète » de la section 2. Des cas sont ambigus ; Sébastien a des solutions à proposer. **En discussion.**

Décidé le 15 novembre 2026 :

- cas A : une adresse dont la connexion échoue se replie sur les clés, comme une adresse incomplète ;
- cas C : des clés sans `DB_ENGINE` sont une erreur ;
- MariaDB est retiré de Runique : trois moteurs, les trois de SeaORM.

## 7. Conséquences du cas A, à régler en l'écrivant

Le repli sur une connexion qui échoue ne se décide pas à la lecture du `.env` : il faut avoir tenté la connexion.

| Appelant | Ce que ça implique |
|---|---|
| l'application | tente l'adresse, puis les clés. Le premier essai coûte le délai de connexion (2 s par défaut) avant le second |
| `migration up`, `down`, `status` | la crate de migration lit une seule adresse. Pour lui transmettre celle qui marche, la commande doit tester la connexion elle-même avant de la lancer |
| makemigrations, pour créer la crate | il ne se connecte pas : il prend le moteur de l'adresse si elle est lisible, sinon celui des clés |

## 8. Constats lus dans le code de SeaORM 2.0.4 (15 novembre 2026)

Lus dans `sea-orm-2.0.4/src/database/db_connection.rs`, fonction `is_prefix_of`. Non vérifiés par une connexion réelle.

| Constat | Conséquence |
|---|---|
| SeaORM ne reconnaît que trois débuts d'adresse : `postgres` ou `postgresql`, `mysql`, `sqlite`. `mariadb://` n'en fait pas partie | une adresse `mariadb://…` est refusée (« no supporting driver ») |
| SeaORM lit l'adresse avec `Url::parse(...).expect(...)` | une adresse illisible ne rend pas une erreur : elle fait **paniquer** le programme. Runique doit donc refuser une adresse mal formée avant de la lui donner |
