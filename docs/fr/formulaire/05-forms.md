# Formulaires

## Sommaire

- [Vue d'ensemble](#vue-densemble)
- [Extracteur Prisme](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/prisme/prisme.md)
- [Trait RuniqueForm](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/trait/trait.md)
  - Structure de base
  - Méthodes du trait
  - Pipeline `is_valid()`
- [Helpers de conversion typée](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/helpers/helpers.md)
- [Types de champs](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/champs/champs.md)
  - TextField, NumericField, BooleanField, ChoiceField, RadioField…
  - Récapitulatif
- [Erreurs de base de données](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/erreurs/erreurs.md)
- [Rendu dans les templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/templates/templates.md)
- [Exemple complet & pièges courants](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/exemple/exemple.md)

---

<a id="vue-densemble"></a>

## Vue d'ensemble

Runique fournit un système de formulaires puissant, inspiré de Django. Il existe **deux approches** :

1. **Manuelle** — Définir les champs via le trait `RuniqueForm`.
2. **Automatique** — Dériver un formulaire depuis un schéma `model!` avec `#[form(...)]`.

Les formulaires sont extraits automatiquement des requêtes via l'extracteur **Prisme**, gèrent la validation (y compris via le crate `validator` pour les emails/URLs), le CSRF, le hachage Argon2 des mots de passe, et peuvent être sauvegardés directement en base de données.

---

## Prochaines étapes

← [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/fr/routing/04-routing.md) | [**Templates**](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/06-templates.md) →
