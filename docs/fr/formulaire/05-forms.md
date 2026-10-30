# Formulaires

## Sommaire

- [Vue d'ensemble](#vue-d-ensemble)
- [Extraction de formulaire — `request.form()`](/docs/fr/formulaire/prisme)
- [Trait RuniqueForm](/docs/fr/formulaire/trait)
  - Structure de base
  - Méthodes du trait
  - Pipeline `is_valid()`
- [Helpers de conversion typée](/docs/fr/formulaire/helpers)
- [Types de champs](/docs/fr/formulaire/champs)
  - TextField, NumericField, BooleanField, ChoiceField, RadioField…
  - Récapitulatif
- [Erreurs de base de données](/docs/fr/formulaire/erreurs)
- [Rendu dans les templates](/docs/fr/formulaire/templates)
- [Exemple complet & pièges courants](/docs/fr/formulaire/exemple)

---

<a id="vue-densemble"></a>

## Vue d'ensemble

Runique fournit un système de formulaires puissant, inspiré de Django. Il existe **deux approches** :

1. **Manuelle** — Définir les champs via le trait `RuniqueForm`.
2. **Automatique** — Dériver un formulaire depuis un schéma `model!` avec `#[form(...)]`.

Les formulaires sont extraits des requêtes via `request.form()` (remplissage et CSRF). La validation se fait avec `ValidationForm::try_form` (y compris via le crate `validator` pour les emails/URLs), le hachage Argon2 des mots de passe aussi, et un formulaire d'entité validé s'écrit avec `save` / `delete`.

---

## Prochaines étapes

← [**Routing**](/docs/fr/routing) | [**Templates**](/docs/fr/template) →
