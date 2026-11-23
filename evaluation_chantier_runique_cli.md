# Évaluation du chantier `runique_cli`

Rédigé le 2026-11-23, à partir d'une lecture du code de la branche `makemigration` (4.1.0-preview).
Rien n'est codé. Les chiffres sont des comptages de lignes et de `use`, pas une lecture de chaque
usage : ce qui n'a pas été vérifié jusqu'au bout est marqué « à vérifier ».

## 1. Ce qui est décidé

Par Sébastien, le 2026-11-23 :

- Le chantier `makemigrations` est mis en pause. Point d'arrêt : renommage terminé et commité ;
  le morceau suivant (`Pk` lu dans le projet, puis refus quand la clé primaire change) n'est pas
  commencé.
- La CLI sort dans une crate séparée, dans `runique/runique_cli/`.
- Pour les tests, elle est utilisée en dossier local, en version 0.1.0.
- Principe : c'est le `Cargo.toml` du projet du dev qui transmet les valeurs à la CLI (type de
  clé primaire, moteur), « d'entrée et de partout », et non les features de son installation.
- But : que l'installation ne serve « qu'aux pilotes et aux CLI, et non à avoir Runique en
  entier », et en profiter pour alléger.

À évaluer ici, à sa demande : l'export des dépendances vers `runique_cli`, le nettoyage du
prelude, et l'idée que `runique` importe ses dépendances via `runique_cli`.

## 2. État des lieux

La lib `runique` fait environ 78 900 lignes (tests internes compris).

| Module | Lignes | Rôle |
|---|---:|---|
| `migration/` | 13 674 | **deux choses mélangées**, voir plus bas |
| `auth/` | 12 072 | exécution |
| `admin/` | 10 979 | exécution (feature `admin`) |
| `forms/` | 10 607 | exécution |
| `utils/` | 6 744 | exécution, dont les traductions |
| `app/` | 5 155 | exécution |
| `atomic/` | 4 781 | lot de fichiers : outillage **et** exécution |
| `middleware/` | 3 839 | exécution |
| `macros/` | 2 723 | exécution |
| `db/` | 2 310 | exécution, dont la configuration de connexion |
| `new_project/` | 885 | outillage |
| `cli/` | 702 | outillage (dont `test_runner.rs`, 644 lignes de logique) |
| `bin/runique.rs` | 113 | outillage (lecture des arguments) |

### Le constat qui structure tout : `migration/` contient deux codes différents

| Partie de `migration/` | Lignes | Qui s'en sert |
|---|---:|---|
| `cli_migration/` | 6 223 | `makemigrations`, `migration up/down/status` |
| `tests_pipeline.rs` | 1 951 | tests de l'outillage |
| `parse/` | 1 383 | lecture des modèles et des snapshots |
| `generate/` | 1 250 | écriture des fichiers de migration |
| `changes/`, `diff.rs`, `paths.rs`, `types.rs`, `helpers.rs` | 1 159 | comparaison |
| **Outillage** | **≈ 11 970** | **la CLI seulement** |
| `schema/` | 1 445 | `ModelSchema`, `ColumnDef`, `PrimaryKeyDef` : l'admin, la porte des droits, les formulaires, le code généré par `model!{}` |
| `runtime/` | 233 | appelé par les fichiers de migration générés, donc par le crate `migration` de chaque projet |
| **Exécution** | **≈ 1 680** | **toutes les applications** |

Avec `new_project/`, `cli/` et le binaire, l'outillage pèse environ **13 700 lignes, soit 17 % de
la lib**, que chaque application compile aujourd'hui sans jamais s'en servir.

### Les liens entre l'outillage et le reste

Ce dont l'outillage a besoin dans `runique` :

- `atomic` (lot de fichiers et journal) ;
- `db::DatabaseConfig` (règle de connexion unique) ;
- `utils::trad` (`t`, `tf`, et les fichiers de traduction, qui contiennent ses messages) ;
- `utils::runique_log`, `utils::private_file::write`, `utils::config::env` ;
- `auth::migrations_table::FRAMEWORK_MIGRATION_NAMES` ;
- `auth::create_superuser` (entité des comptes, hachage, validation).

Les 12 usages de `forms` relevés dans `migration/` sont tous dans `schema/`, donc côté exécution :
l'outillage lui-même ne touche ni aux formulaires, ni au serveur, ni aux templates, ni à l'admin.
Les `crate::views` et `crate::forms` de `new_project/` sont dans le texte des fichiers qu'il
écrit, pas dans son code.

Ce que le reste de la lib prend à l'outillage : rien de relevé. `atomic` sert aussi à l'exécution
(`forms/fields/media.rs` pour les fichiers envoyés, `db/config.rs`).

À vérifier au moment de couper : que `schema/` et `runtime/` n'utilisent rien de `types.rs`,
`helpers.rs` ou `paths.rs`.

## 3. Trois façons de faire

### A. La CLI dépend de `runique`, et `runique` met « le reste » derrière une feature

`runique_cli` → `runique` sans les features par défaut. Tout ce qui n'est pas commande (environ
55 000 lignes, Axum, Tera…) passe derrière une feature `web` activée par défaut.

- Pour : aucun code ne déménage.
- Contre : des `#[cfg]` sur presque toute la lib, à tenir à chaque ajout ; une combinaison de plus
  en CI ; `create-superuser` a besoin d'une partie d'`auth`, qu'il faudrait découper ;
  `runique` reste aussi lourde pour les applications (les 13 700 lignes d'outillage y restent).

### B. L'outillage déménage dans `runique_cli`, qui dépend de `runique`

Les 13 700 lignes quittent `runique`. `runique_cli` → `runique`.

- Pour : c'est un déplacement, pas une réécriture ; **toutes les applications s'allègent tout de
  suite** (17 % de code en moins et les dépendances du tableau de la section 4) ; la frontière
  devient nette ; aucun `#[cfg]` ajouté.
- Contre : l'installation de la CLI compile encore `runique` en entier ; ce que l'outillage
  utilise en `pub(crate)` doit devenir `pub` (exemple vérifié : `AtomicBuilder` et `SourceItems`
  sont `pub(crate)`), donc une surface publique en plus à assumer.

### C. B, puis un socle commun pour que la CLI n'ait plus besoin de `runique` en entier

Ce que les deux côtés partagent (lot atomique, configuration de connexion, traductions, journal
d'exécution) va dans un socle, et la CLI ne dépend plus que de lui.

- **C1 : le socle est `runique_cli`, et `runique` l'importe** (l'idée « importer via
  `runique_cli` »). Voir la section 6 : cargo interdit les dépendances en cercle, et
  `create-superuser` a besoin d'`auth`.
- **C2 : le socle est une crate à part**, dont dépendent `runique` et `runique_cli`.

### Recommandation

**B d'abord.** C'est le plus petit pas, il allège immédiatement toutes les applications, et il ne
ferme aucune porte : A, C1 et C2 partent tous de B. Le choix entre eux se fera avec des mesures
(temps de compilation de la CLI avant et après), une fois B en place.

## 4. Export des dépendances vers `runique_cli`

Comptage des fichiers qui utilisent chaque crate, outillage contre reste de la lib.

| Dépendance | Outillage | Reste de la lib | Après B |
|---|---|---|---|
| `clap` | le binaire | aucun | **part** dans `runique_cli` |
| `crossterm` | `cli_migration/report.rs` | aucun | **part** |
| `syn`, `quote`, `proc-macro2` | 7, 2 et 1 fichiers | aucun | **partent** (commentées « Admin daemon » dans le manifeste, un reste de l'ancien admin) |
| `rpassword` | aucun | aucun (vérifié : le mot n'apparaît dans aucun `.rs` de `runique` ni de `demo-app`) | **à retirer**, elle ne sert plus |
| `dialoguer` | aucun | `auth/create_superuser.rs` | **part** si la saisie du superuser passe dans la CLI |
| `runique_dsl` | 4 fichiers | `migration/schema/` (2 fichiers) | **reste**, et `runique_cli` l'ajoute |
| `sea-orm-migration` | 6 fichiers | `runtime/`, `auth/migrations_table.rs` | **reste**, et `runique_cli` l'ajoute |
| `toml` | aucun (mais `build.rs` pour `runique new`) | `errors/error.rs` | **reste** ; le `build.rs` suit `new_project` |
| `glob`, `walkdir`, `libc` | aucun | 1 fichier chacun | restent |

`syn` reste dans le graphe de compilation de toute application, puisque `derive_form` (la macro)
en a besoin ; mais `runique` elle-même ne le lierait plus.

Côté features : `all-databases` n'a de raison d'être que pour la CLI et la CI. Avec B,
`runique_cli` peut l'activer elle-même, et l'installation ne demande plus de choisir un moteur.

## 5. Le prelude

Constat ([lib.rs:106](runique/src/lib.rs#L106), environ 190 lignes) :

- des types de la bibliothèque standard (`HashMap`, `HashSet`, `Arc`, `LazyLock`) ;
- `anyhow::{Error, Result}`, qui masque le `Result` standard dans tout fichier qui importe le
  prelude ;
- des imports en bloc : `crate::auth::*`, neuf modules de `middleware::…::*`,
  `sea_orm::entity::prelude::*`, `utils::aliases::*`, `utils::csp_nonce::*` ;
- une vingtaine de noms d'Axum, plus `tokio`, `serde_json`, `tera`, `chrono`, `uuid`, `regex`,
  `hmac`, `sha2` ;
- `dotenvy` exporté deux fois (racine et prelude) ; un `use crate::chrono;` isolé ;
- `sea_orm_migration::sea_query` exporté sans condition alors que la dépendance est optionnelle.

Rien de tout cela ne concerne la CLI : le prelude ne contient presque rien de l'outillage
(`migrations_table`, `ModelSchema`, côté exécution).

**La contrainte :** chaque nom retiré du prelude casse le code des applications qui s'en servent.
C'est une rupture, donc soit une version majeure, soit une dépréciation annoncée. Le nettoyage du
prelude est un sujet indépendant de la sortie de la CLI, et il ne l'allège pas : les réexports ne
coûtent rien à la compilation.

**Proposition :** ne pas le mêler au déplacement. Le traiter comme un morceau à part, en trois
temps : mesurer ce que `demo-app`, le portfolio et Campanile utilisent réellement ; décider ce qui
part ; l'annoncer dans `MIGRATION` et le CHANGELOG. La décision de la version (4.x avec
dépréciation, ou 5.0) revient à Sébastien.

## 6. `runique` qui importerait via `runique_cli`

L'idée : la logique commune vit dans `runique_cli`, et `runique` la reprend de là.

Ce que ça demande :

- **Pas de cercle.** Si `runique` dépend de `runique_cli`, alors `runique_cli` ne peut plus rien
  prendre dans `runique`. Or `create-superuser` a besoin de l'entité des comptes, du hachage et de
  la validation, qui sont dans `auth`. Il faudrait soit laisser cette commande dans `runique`,
  soit descendre ces morceaux d'`auth` dans `runique_cli`.
- **Toutes les applications compileraient `runique_cli`.** Pour que ce soit acceptable, la crate
  devrait séparer son socle (toujours compilé) de son outillage et de son binaire (derrière une
  feature), ce qui ramène les `#[cfg]` qu'on voulait éviter.
- **Le nom ne dirait plus ce que c'est.** Le lot atomique, la configuration de connexion et les
  traductions sont des fondations du framework ; les ranger dans une crate nommée « cli » rend la
  lecture du projet moins évidente.

**Avis :** si un socle commun se justifie, une crate à part (C2) donne le même résultat sans ces
trois inconvénients. Et il n'est pas certain qu'il se justifie : c'est une question de temps de
compilation de la CLI, à mesurer après B.

## 7. Étapes proposées

**Étape 1 : la crate existe (petit).**
`runique/runique_cli/` en 0.1.0, membre du workspace, dépendance sur `runique` en chemin local.
Le binaire y déménage tel quel ; `clap` quitte `runique`. Même comportement.

**Étape 2 : l'outillage déménage (le gros du travail, mécanique).**
`cli_migration/`, `parse/`, `generate/`, `changes/`, `diff.rs`, `paths.rs`, `types.rs`,
`new_project/`, `cli/` et leurs tests (dont `runique/tests/migration/`, environ 10 900 lignes, à
trier entre outillage et exécution). `schema/` et `runtime/` restent dans `runique`. Les
dépendances de la section 4 suivent. Passage en `pub` de ce que l'outillage utilisait en
`pub(crate)`.

**Étape 3 : le Cargo du projet transmet les valeurs.**
`runique_cli` active tous les moteurs. Type de clé primaire et moteur lus par `cargo metadata`.
C'est ici que reprend le morceau `Pk` mis en pause.

**Étape 4 : mesurer, puis décider d'alléger l'installation de la CLI** (A, C1 ou C2).

**À part : le prelude** (section 5).

## 8. Ce que chaque étape entraîne en dehors du code

- Documentation FR et EN : commande d'installation, page CLI, README, CONTRIBUTING, TESTING.
- `CLAUDE.md` du projet : commandes de test et d'installation.
- CI (`.github/workflows/ci.yml`) et `.cargo/mutants.toml` : la nouvelle crate, ses features.
- Ordre de publication : `runique_dsl` → `derive_form` → `runique` → `runique_cli`.
- CHANGELOG FR et EN, `PROJECT_STATUS` FR et EN.
- Projets existants : demo-app, portfolio, Campanile réinstallent la CLI.

## 9. Risques

- **Surface publique.** Ce qui devient `pub` pour l'outillage devient une promesse envers les
  utilisateurs. À limiter, et à marquer comme interne dans la doc de l'API.
- **Tests à trier.** Les tests de migration mélangent outillage et exécution dans les mêmes
  fichiers ; un tri hâtif peut en perdre.
- **Deux crates, une seule version utile.** Le format des snapshots et les fichiers générés lient
  la CLI à la version exacte de la lib. La CLI devra refuser clairement un projet d'une autre
  version.
- **Le chantier en pause.** `makemigrations` reprendra dans des fichiers qui auront changé de
  crate : la checklist et le tableau des décisions devront suivre les nouveaux chemins.

## 10. Variante D : des copies, pour que la CLI ne dépende plus de `runique`

Demandée par Sébastien le 2026-11-23. L'outillage déménage comme dans B, mais au lieu de dépendre
de `runique`, `runique_cli` embarque sa propre copie de ce qu'il y prenait.

### Ce qu'il faudrait copier, morceau par morceau

| Morceau | Taille | Sert aussi à l'exécution ? | Copier ? |
|---|---|---|---|
| Messages de l'outillage (`makemigrations`, `migrate`, `cli`, `test_runner`) | 63 clés sur 366, en 9 langues | non | **pas une copie : ils déménagent** |
| Moteur de traduction (`utils/trad`) | 250 lignes | oui | copie acceptable : petit, stable |
| `FRAMEWORK_MIGRATION_NAMES` | 5 noms | oui | copie acceptable, à condition qu'un test compare les deux listes |
| `private_file::write`, `env`, niveau de journal | environ 350 lignes, 4 usages | oui | copie acceptable, ou réécriture du strict nécessaire |
| Lot atomique (`atomic/`) | environ 500 lignes de code et **4 300 lignes de tests** | oui (fichiers envoyés, `forms/fields/media.rs`) | **mauvais candidat** |
| Règle de connexion (`db/config.rs`) | 1 197 lignes, tests compris | oui | **mauvais candidat** |
| `create-superuser` | 760 lignes, plus l'entité des comptes et le hachage (`utils::password`) | oui | **à ne pas copier** |

### Pourquoi trois morceaux résistent

- **Le lot atomique** est le code le plus délicat du dépôt (coupure de courant, réparation). Deux
  copies, c'est chaque correction à faire deux fois, et 4 300 lignes de tests à dupliquer aussi,
  ou une copie qui n'est plus testée. On vient d'y ajouter le second dossier du journal : avec
  deux copies, il aurait fallu le faire deux fois.
- **La règle de connexion** fait l'objet d'une décision déjà prise et affichée dans
  `PROJECT_STATUS` : « une seule règle de connexion pour l'application et les commandes ». Deux
  copies peuvent diverger, et le jour où elles divergent, `makemigrations` lit une autre base que
  celle de l'application, sans message.
- **`create-superuser`** écrit dans `eihwaz_users`, dont la forme est définie dans `runique`, et
  hache le mot de passe. Une copie de l'entité qui retarde d'une colonne casse la commande ; une
  copie du hachage, c'est du code de sécurité à deux endroits. C'est le seul morceau où une
  divergence aurait une conséquence de sécurité.

### Ce qui rendrait des copies tenables

`runique` peut être une **dépendance de test** de `runique_cli` sans être une dépendance du
binaire installé. Des tests peuvent alors vérifier que chaque copie est identique à l'original :
mêmes noms de migrations, même URL de connexion pour les mêmes variables, et même texte de fichier
pour le code copié tel quel. Une divergence fait échouer la CI au lieu de passer inaperçue. Cela
ne supprime pas le travail en double, mais il devient impossible de l'oublier.

### Ce que D apporte

L'installation de la CLI ne compilerait plus Axum, Tera, l'envoi de mail, le traitement d'image,
le nettoyage HTML ni l'admin : seulement les pilotes de base, SeaORM, le lecteur du DSL et les
commandes. C'est le but visé. Le gain en temps de compilation n'est pas mesuré.

### D comparée au socle commun (C2)

| | D : copies | C2 : crate de socle |
|---|---|---|
| Installation légère | oui | oui |
| Code en double | environ 2 000 lignes, plus des tests | aucun |
| Crates à publier | 4 | 5 |
| Risque de divergence | réel, à contenir par des tests | aucun |
| `create-superuser` | non résolu | non résolu (il a besoin d'`auth`, qui n'est pas du socle) |

`create-superuser` est donc une question à part, quelle que soit la variante. Trois issues : la
commande reste dans `runique` et se lance depuis le projet ; ou la CLI garde une dépendance à
`runique` derrière une feature, pour cette seule commande ; ou la commande est repensée pour ne
plus avoir besoin de l'entité (insertion par sea-query typé), ce qui garde le hachage en double.

### Avis

Les copies conviennent à ce qui est petit et stable (le moteur de traduction, la liste des noms,
les petits utilitaires), et les messages n'ont même pas à être copiés. Elles conviennent mal au
lot atomique et à la règle de connexion, et pas du tout au superuser.

Un mélange est possible et me paraît le plus juste : **déménager** ce que seule la CLI utilise,
**copier sous contrôle d'un test** ce qui est petit, et **partager dans une petite crate** les
deux morceaux délicats (lot atomique, règle de connexion). Cette crate ferait environ
1 700 lignes de code, sans Axum ni Tera.

Dans tous les cas, l'ordre ne change pas : B d'abord (le déménagement), parce que D, C2 et le
mélange en partent tous, et qu'il permet de mesurer avant de choisir.

## 10 bis. Pistes de Sébastien du 2026-11-23, vérifiées dans le code

**Le lot atomique dans une crate à part.** Règle la double maintenance. Détail à régler : il
affiche 7 messages traduits par `utils::trad`, donc il lui faut le moteur de traduction.

**La connexion : garder le `.env`, lu là où la commande est lancée.** C'est déjà le cas, et ça ne
change pas. Mais ce qui doit rester unique n'est pas la lecture du fichier : c'est la règle qui
interprète les variables (`DATABASE_URL` d'abord, repli sur `DB_*`, refus sans `DB_ENGINE`, mot de
passe jamais affiché), soit `db/config.rs`, `engine.rs` et `builder.rs`. Elle dépend de SeaORM, de
`utils::trad` et du journal d'exécution, de rien d'autre dans `runique` : elle peut déménager
comme le lot atomique.

**Le superuser, si la CLI porte les tables du framework.**
- `auth/migrations_table.rs` (683 lignes, la définition des tables `eihwaz_*`) ne dépend que de
  sea-query et d'une constante : elle se déplace sans difficulté.
- Mais l'application en a besoin aussi (le crate `migration` de chaque projet applique ces
  migrations par `runique::migrations_table`). Elle doit donc aller dans une crate dont `runique`
  **et** la CLI dépendent, pas dans `runique_cli` même, sinon `runique` dépendrait de sa CLI.
- L'entité des comptes (`auth/user.rs`) est liée aux sessions, aux droits, à `schema` et au
  moteur : elle ne sort pas de `runique`. Le superuser n'en a pas besoin si la commande insère la
  ligne par sea-query typé à partir de la définition de table partagée.
- Reste le hachage. `utils/password` (626 lignes) est lié à `forms` et à `auth` ; il faudrait en
  sortir le cœur (les trois algorithmes). **À vérifier** : jusqu'où il se sépare.

**Ce vers quoi ça converge : une seule crate de socle**, plutôt qu'une par morceau, parce que
chacun de ces morceaux a besoin des traductions et du journal d'exécution :

```
runique_dsl ──► derive_form ──► runique
runique_socle ─────────────────► runique
runique_socle + runique_dsl ───► runique_cli      (plus aucune dépendance à runique)
```

Le socle contiendrait : lot atomique, règle de connexion, définition des tables du framework,
cœur du hachage, moteur de traduction, journal d'exécution. Ni Axum ni Tera.

**À vérifier avant de s'engager** : le moteur de traduction charge aujourd'hui un fichier par
langue ; il faut qu'il accepte des messages venant de plusieurs crates (ceux du socle, ceux de
`runique`, ceux de la CLI).

Remarque de Sébastien (2026-11-23) : cela rejoint l'évolution de l'i18n déjà prévue (liste du
2026-10-20 : un dossier par langue, sections par module, dossier du développeur, validation au
démarrage). Le besoin est le même mécanisme : un moteur qui assemble plusieurs sources de
messages. Seule cette partie-là est nécessaire ici ; la langue par requête et le dossier du
développeur restent dans le chantier i18n.

## 10 ter. L'i18n faite en entier (Sébastien, 2026-11-23 : « si on la fait, autant la faire bien »)

L'i18n devient une crate à part, et la langue par requête passe par un middleware.

Le partage se fait proprement, parce que les deux moitiés n'ont pas les mêmes dépendances :

| Dans la crate i18n (ni Axum ni Tera) | Dans `runique` |
|---|---|
| le moteur : plusieurs sources de messages assemblées | le middleware qui choisit la langue (URL, puis cookie, puis `Accept-Language`) |
| un dossier par langue, des sections par module | la pose de cette langue pour la durée de la requête |
| le dossier du développeur et la validation au démarrage | le filtre ou la balise de template |
| « la langue en cours » : celle de la requête si elle est posée, sinon la langue globale | |

La CLI n'a pas de requête : elle garde la langue globale, lue dans le `.env` ou l'environnement,
comme aujourd'hui.

**Point de conception à régler** : pour que `t()` lise la langue de la requête, il faut une valeur
attachée à la tâche en cours. Sous Tokio, une variable de fil ne suffit pas (une requête change de
fil d'un `await` à l'autre). Soit la crate i18n dépend de Tokio pour cela, soit elle offre
seulement « traduire dans telle langue » et c'est `runique` qui garde la valeur de la tâche.
Limite déjà notée le 2026-11-09 : un middleware ajouté par `add_custom` s'exécute après le choix
de la route, donc pas de préfixe de langue dans l'URL sans envelopper le routeur.

### Ordre qui en découle

1. **Crate i18n** : tout le reste en a besoin (le socle, la CLI, `runique`).
2. **Socle** : lot atomique, règle de connexion, tables du framework, cœur du hachage.
3. **`runique_cli`** : l'outillage déménage, plus aucune dépendance à `runique`.
4. **Le Cargo du projet transmet les valeurs** : `Pk`, moteur. Reprise de `makemigrations`.
5. **Dans `runique`** : middleware de langue par requête.

Chaque étape laisse le dépôt dans un état qui compile et dont les tests passent.

**À savoir** : ce sont cinq chantiers à la suite, pendant lesquels `makemigrations` reste à
mi-chemin et la 4.1 non publiable. Les étapes 1 à 3 ne changent rien pour un utilisateur de la
4.0 ; seule l'étape 5 lui apporte une fonction nouvelle.

## 11. À trancher par Sébastien

1. L'architecture de départ : B (recommandé), ou A.
2. Nom de la commande : `runique` inchangé (recommandé) ou autre.
3. `test_runner.rs` (644 lignes de logique dans `cli/`) : il suit dans `runique_cli` à l'étape 2 ?
4. `create-superuser` : la saisie au clavier part dans la CLI et la création reste dans `auth` ?
5. Le prelude : dans ce chantier, ou à part avec sa propre décision de version ?
6. Version et nom publié : `runique_cli` 0.1.0 en local ; quelle version à la publication ?
7. Indépendance de la CLI, après le déménagement : copies (D), crate de socle (C2), ou le mélange
   de la section 10 ?
8. `create-superuser` : reste lancé depuis le projet, dépendance à `runique` derrière une feature,
   ou commande repensée ?
