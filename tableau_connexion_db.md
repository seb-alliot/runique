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

### « Incomplète » (à trancher)

Proposition :

| Dans l'adresse | Exigé ? | Pourquoi |
|---|---|---|
| un début reconnu : `postgres://`, `postgresql://`, `mysql://`, `sqlite://` | oui | sinon on ne sait pas à quel moteur parler |
| une forme lisible : pas de port `543a`, pas d'`@` en trop, pas de guillemets autour | oui | |
| un hôte, pour un serveur | oui | |
| un nom de base | oui | |
| un utilisateur et un mot de passe | non | Postgres et MySQL en local acceptent souvent la connexion sans |

Les guillemets ou les espaces autour de la valeur sont une malformation comme une autre : ils suivent la ligne 2, 3 ou 4.

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
