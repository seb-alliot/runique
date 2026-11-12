# État du projet — 4.1 et 4.2

Ce qui reste à faire pour les deux prochaines versions. Un élément peut passer d'une version à l'autre en cours de route, s'il faut les livrer ensemble. La suite du projet est dans la [feuille de route](https://github.com/seb-alliot/runique/blob/main/ROADMAP.fr.md).

**Dernière mise à jour** : 12 novembre 2026

## 4.1 : infrastructure base de données (en cours)

### makemigrations

| État | Étape |
|:---:|---|
| ✓ | Plan, validation, commit ou rollback atomique, snapshots |
| ✓ | Renommage simple d'une colonne (`renamed_from`) |
| ✓ | Refus sans `--force` des opérations destructives : `DROP COLUMN`, changement de type, nullable vers `NOT NULL`, longueur ou décimale réduite, `DROP FK`, nouvelle FK `CASCADE` |
| ✓ | Limite de longueur des noms d'index et de clés étrangères : refus au-delà de 63 caractères, sur tous les moteurs |
| ☐ | Suppression de table quand le modèle disparaît : refus sans `--force`, `DROP TABLE` dans `up`, recréation dans `down`, snapshot retiré, ordre inverse des FK |
| ☐ | Changements destructeurs : indiquer ce qu'ils touchent, relations comprises |
| ☐ | Vrai `ALTER` lors d'un changement de type (`USING` sur PostgreSQL et MySQL, snapshot fidèle ; reconstruction de table sous SQLite en option) |
| ☐ | Passage de nullable à `NOT NULL` avec défaut : remplir les `NULL` existants avant la contrainte |
| ☐ | Colonne à la fois renommée et modifiée (aujourd'hui seul le renommage est appliqué, et le snapshot enregistre la modification comme faite) |
| ☐ | Détection d'un changement de valeur par défaut |
| ☐ | Noms de contraintes lus en base au lieu d'être supposés |
| ☐ | Renommage de table (`renamed_from`), et message explicite pour un modèle renommé |

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
