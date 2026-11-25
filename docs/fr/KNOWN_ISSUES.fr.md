🌍 **Langues** : [English](../en/KNOWN_ISSUES.md) | [Français](KNOWN_ISSUES.fr.md)

# Problèmes connus

Les défauts connus de chaque version publiée de Runique, leur contournement et la version qui les corrige. Les failles de sécurité sont dans [SECURITY.fr.md](../../SECURITY.fr.md), les corrections dans [CHANGELOG.fr.md](CHANGELOG.fr.md).

## 4.0.0

### L'admin ne peut pas créer de ligne avec une clé UUID (`pk-uuid`)

- **Ce que ça fait** : sur la page de création d'un `model!{}`, l'enregistrement est refusé avec « Action non autorisée », quels que soient les droits du compte. L'admin insère la ligne sans sa clé, et aucune base ne génère une clé UUID.
- **Qui est concerné** : les projets compilés avec la feature `pk-uuid` qui utilisent l'admin. Les clés entières (par défaut et `big-pk`) ne sont pas concernées. La modification et la suppression fonctionnent.
- **Contournement** : créer ces lignes depuis une vue du projet, en fournissant la clé (`Uuid::now_v7()`).
- **Correctif** : Runique 4.1.0. Un modèle à clé UUID reçoit sa clé à l'insertion quand elle est absente.

### L'admin n'enregistre pas les champs liste (`checkbox`, `multichoice`)

- **Ce que ça fait** : les valeurs cochées dans un champ liste ne sont pas enregistrées, sans message d'erreur, et le champ n'apparaît pas sur la page de modification.
- **Qui est concerné** : les projets dont un `model!{}` enregistré dans l'admin déclare un champ `checkbox` ou `multichoice`.
- **Contournement** : écrire ces valeurs depuis une vue du projet, avec la méthode `set_<champ>` du modèle.
- **Correctif** : Runique 4.1.0. L'admin écrit, affiche et trace les champs liste comme les colonnes, avec les mêmes droits.

### Un formulaire refusé avant l'écriture garde le chemin du fichier envoyé

- **Ce que ça fait** : quand `save` refuse un formulaire sans avoir tenté l'écriture, le formulaire rendu garde dans son champ fichier le chemin où l'envoi serait allé (`avatars/photo.png`), alors que le fichier n'a jamais quitté le dossier d'attente. Réaffiché, le formulaire annonce un fichier qui n'existe pas à cet endroit. Rien n'est écrit en base ni déplacé sur le disque.
- **Qui est concerné** : les formulaires de modification qui portent un champ fichier, quand le paramètre de route de la ligne (`{<formulaire>_<clé>}`) est illisible pour le type de clé : `abc` pour une clé entière, un nombre pour une clé UUID. Une ligne absente dont la clé est lisible n'est pas concernée.
- **Contournement** : faire vérifier le paramètre avant le formulaire, en le demandant typé dans le handler (`Path(id): Path<Pk>`) : une valeur illisible est alors refusée avant `save`.
- **Correctif** : Runique 4.1.0. L'envoi est annulé sur tout refus de `save`, de `delete` et de l'écriture par la porte : le champ reprend sa valeur d'avant.

### SQLite : une clé étrangère ajoutée ou retirée sur une table existante fait planter `migration up`

- **Ce que ça fait** : `runique migration up` s'arrête sur un plantage de la bibliothèque SQL (« Sqlite does not support modification of foreign key constraints to existing tables »). Les instructions placées avant dans la même migration sont déjà appliquées.
- **Qui est concerné** : les projets sous SQLite dont une migration ajoute, retire ou modifie un `belongs_to` sur une table déjà créée. PostgreSQL et MySQL ne sont pas concernés. Les migrations qui ne font que créer des tables non plus.
- **Contournement** : sur une base dont les données ne sont pas à garder, effacer la base et les fichiers de migration, puis relancer `runique makemigrations` : les migrations réécrites depuis les modèles ne contiennent que des créations, que SQLite accepte. Sur une base à garder, la table doit être reconstruite à la main.
- **Correctif** : Runique 4.1.0 pour le plantage : la migration est refusée dès sa première instruction, avec un message qui nomme la table, et rien n'est appliqué. Les migrations écrites par la 4.0.0 gardent leur ancien texte et doivent être régénérées pour en profiter. La reconstruction de table, qui rendra ce changement possible sous SQLite, n'est pas encore écrite.
