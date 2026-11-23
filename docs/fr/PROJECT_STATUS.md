# État du projet — 4.1 et 4.2

Ce qui reste à faire pour les deux prochaines versions. Un élément peut passer d'une version à l'autre en cours de route, s'il faut les livrer ensemble. La suite du projet est dans la [feuille de route](https://github.com/seb-alliot/runique/blob/main/ROADMAP.fr.md).

**Dernière mise à jour** : 21 novembre 2026

## 4.1 : infrastructure base de données (en cours)

### makemigrations

| État | Étape |
|:---:|---|
| ✓ | Plan, validation, commit ou rollback atomique |
| ✓ | Renommage simple d'une colonne (`renamed_from`) |
| ✓ | Refus sans `--force` des opérations destructives : `DROP COLUMN`, changement de type, nullable vers `NOT NULL`, longueur ou décimale réduite, `DROP FK`, nouvelle FK `CASCADE` |
| ✓ | Limite de longueur des noms d'index et de clés étrangères : refus au-delà de 63 caractères, sur tous les moteurs |
| ✓ | Connexion à la base avant tout calcul : `makemigrations` s'arrête avec une erreur si elle ne répond pas |
| ✓ | Une seule règle de connexion pour l'application et les commandes : `DATABASE_URL` d'abord, repli sur les clés `DB_*` quand elle est incomplète ou ne répond pas ; clés sans `DB_ENGINE` refusées ; utilisateur et mot de passe des clés encodés |
| ✓ | Refus d'une base qui ne correspond pas au projet : une migration inscrite en base et absente de `lib.rs` arrête la commande |
| ✓ | Contrôle des snapshots avant tout calcul : un snapshot illisible, ou nommé autrement que sa table, arrête la commande |
| ✓ | Migration générée marquée « en attente » : relancé avant qu'elle soit appliquée, `makemigrations` la recalcule au lieu d'en empiler une autre |
| ✓ | Deux modifications parallèles d'une même table : refus, avec les tables concernées (chaque migration note celle dont elle part) |
| ✓ | Suppression de table quand le modèle disparaît : refus sans `--force`, `DROP TABLE IF EXISTS` dans `up`, recréation dans `down`, ordre inverse des FK, types enum retirés quand plus aucune colonne ne s'en sert ; sous SQLite, le retrait de la clé d'une table qui reste attend la reconstruction de table |
| ✓ | `managed: false` dans le `meta` d'un modèle : `makemigrations` ne génère et ne supprime plus rien pour sa table, et rappelle les tables en pause à chaque lancement |
| ✓ | Renommage de table et de champ liste (`renamed_from`), avec index, clés et tables liste ; la directive est retirée du modèle après `migration up` |
| ☐ | Changements destructeurs : indiquer ce qu'ils touchent, relations comprises |
| ☐ | Vrai `ALTER` lors d'un changement de type, après `--force` (`USING` sur PostgreSQL et MySQL, snapshot fidèle) |
| ☐ | SQLite : reconstruction de la table quand une colonne est modifiée, ou qu'une clé étrangère est ajoutée ou retirée (aujourd'hui la modification de colonne est sautée) |
| ☐ | Passage de nullable à `NOT NULL` avec défaut : remplir les `NULL` existants avant la contrainte |
| ☐ | Champ obligatoire ajouté à une table qui contient déjà des lignes : choix en console, donner la valeur des lignes existantes ou ne rien faire |
| ☐ | Colonne à la fois renommée et modifiée (aujourd'hui seul le renommage est appliqué, et le snapshot enregistre la modification comme faite) |
| ☐ | Détection d'un changement de valeur par défaut |
| ☐ | Noms de contraintes lus en base au lieu d'être supposés |
| ☐ | Enums : erreur quand deux modèles déclarent le même nom avec des valeurs différentes ; valeur retirée prise en charge |
| ☐ | Bloc `extend!{}` retiré : ses colonnes sont retirées de la base, la table du framework reste |

### migration up et down

| État | Étape |
|:---:|---|
| ☐ | `migration up` en tout ou rien : si une migration échoue, aucune de celles du lancement ne reste appliquée (transaction sous PostgreSQL et SQLite, compensation sous MySQL) |
| ✓ | Snapshots écrits à l'application : `makemigrations` n'en écrit plus, `migration up` écrit celui de chaque table touchée une fois le lancement réussi et retire la marque « en attente » |
| ☐ | Snapshots ramenés en arrière par `migration down` |
| ☐ | Tables déjà présentes en base sans être inscrites : vérifiées, puis inscrites sans être recréées |
| ☐ | `migration down` : défait la dernière migration, sans rien laisser à moitié fait, et supprime la migration générée ; retire aussi une migration générée et pas encore appliquée |
| ☐ | `makemigrations` et `migration up` ne peuvent pas tourner en même temps |

### Requêtes

| État | Étape |
|:---:|---|
| ☐ | `get_or_create` : récupérer une ligne selon des conditions, ou la créer avec des valeurs par défaut |
| ☐ | `update_or_create` : mettre à jour une ligne selon des conditions, ou la créer |
| ☐ | `exists()` et `last()` |

### Limiteurs

| État | Étape |
|:---:|---|
| ☐ | Plafond du nombre de clés gardées en mémoire par `RateLimiter` et `LoginGuard` |
| ☐ | Blocage par paliers : une durée qui s'allonge à chaque récidive (5 min, 15 min, 1 h par défaut), chaque palier configurable (`lockout_secs` déprécié, supprimé en 5.0) |

## 4.2 : relations

| État | Étape |
|:---:|---|
| ☐ | Création et suppression automatiques des tables de relation du many-to-many (DSL, migrations, macro) |
| ☐ | Respect de `cascade`, `restrict` et `set_null` sur ces tables |
| ☐ | Chargement des relations en une requête, pour le many-to-many |
| ☐ | Méthodes d'accès du many-to-many sur le modèle |
| ☐ | `search!` : groupes ET dans un OU, négation d'une condition ou d'un groupe |

### Admin : many-to-many

| État | Étape |
|:---:|---|
| ☐ | Sélection des valeurs dans le formulaire |
| ☐ | Affichage et filtre dans la liste |
| ☐ | Enregistrement des liens dans la transaction de la ligne, par la porte des droits, tracé dans l'historique |
