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
