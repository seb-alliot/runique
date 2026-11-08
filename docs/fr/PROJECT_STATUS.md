# État du projet — chantier makemigrations

Ce qui reste à faire pour que `makemigrations` génère toutes les opérations de schéma. La suite du projet est dans la [feuille de route](https://github.com/seb-alliot/runique/blob/main/ROADMAP.fr.md).

**Dernière mise à jour** : 8 novembre 2026

## 4.1 : makemigrations complet (en cours)

| État | Étape |
|:---:|---|
| ✓ | Plan, validation, commit ou rollback atomique, snapshots |
| ✓ | Renommage simple d'une colonne (`renamed_from`) |
| ✓ | Refus sans `--force` des opérations destructives : `DROP COLUMN`, changement de type, nullable vers `NOT NULL`, longueur ou décimale réduite, `DROP FK`, nouvelle FK `CASCADE` |
| ☐ | Suppression de table quand le modèle disparaît : refus sans `--force`, `DROP TABLE` dans `up`, recréation dans `down`, snapshot retiré, ordre inverse des FK |
| ☐ | Changements destructeurs : indiquer ce qu'ils touchent, relations comprises |
| ☐ | Vrai `ALTER` lors d'un changement de type (`USING` sur PostgreSQL et MySQL, snapshot fidèle ; reconstruction de table sous SQLite en option) |
| ☐ | Passage de nullable à `NOT NULL` avec défaut : remplir les `NULL` existants avant la contrainte |
| ☐ | Colonne à la fois renommée et modifiée (aujourd'hui seul le renommage est appliqué, et le snapshot enregistre la modification comme faite) |
| ☐ | Détection d'un changement de valeur par défaut |
| ☐ | Noms de contraintes lus en base au lieu d'être supposés |
| ☐ | Limite de longueur des identifiants selon le moteur (63 pour PostgreSQL, 64 pour MariaDB/MySQL) |
| ☐ | Création et suppression automatiques des tables de relation du many-to-many (DSL, migrations, macro) |
| ☐ | Respect de `cascade`, `restrict` et `set_null` sur ces tables |
| ☐ | Renommage de table (`renamed_from`), et message explicite pour un modèle renommé |

## 4.2 : fonctionnalités liées à la base de données

| État | Étape |
|:---:|---|
| ☐ | `get_or_create` : récupérer une ligne selon des conditions, ou la créer avec des valeurs par défaut |
| ☐ | `update_or_create` : mettre à jour une ligne selon des conditions, ou la créer |
| ☐ | `exists()` et `last()` |
| ☐ | Chargement des relations en une requête, pour le many-to-many |
| ☐ | Méthodes d'accès du many-to-many sur le modèle |
| ☐ | `search!` : groupes ET dans un OU, négation d'une condition ou d'un groupe |

### Admin : many-to-many

| État | Étape |
|:---:|---|
| ☐ | Sélection des valeurs dans le formulaire |
| ☐ | Affichage et filtre dans la liste |
| ☐ | Enregistrement des liens dans la transaction de la ligne, par la porte des droits, tracé dans l'historique |
