🌍 **Langues** : [English](ROADMAP.md) | [Français](ROADMAP.fr.md)

# Runique — Feuille de route

Peu de chantiers à la fois, chacun livré fini. Ce qui est déjà livré est dans le
[CHANGELOG](CHANGELOG.fr.md).

Cette feuille de route ne donne ni date ni engagement. Un bug ou une faille découverts passent
avant les chantiers prévus, et peuvent donner lieu à une version corrective à tout moment.

**Ce que « fini » veut dire** : une fonctionnalité n'entre dans une version publiée que si elle
est testée des deux côtés (cas accepté et cas refusé, limite comprise), si une passe
`cargo mutants --in-diff` a traité chaque survivant, si ses contrôles de sécurité ont lieu avant
l'effet, si elle a été vérifiée sur demo-app et sur un projet créé par `runique new`, et si la
documentation (fr/en) et le CHANGELOG sont à jour.

## En cours

- **4.1, infrastructure base de données** : `makemigrations` génère les opérations de schéma sans
  étape manuelle : suppression de tables, vrai `ALTER` lors d'un changement de type, passage à
  `NOT NULL` avec remplissage des valeurs existantes, détection des changements de valeur par
  défaut, colonnes à la fois renommées et modifiées, renommage de table. Les noms de contraintes
  sont lus en base, et les changements destructeurs indiquent ce qu'ils touchent.
  Côté requêtes : `get_or_create`, `update_or_create`, `exists()` et `last()`.
  Côté sécurité : plafond du nombre de clés gardées en mémoire par `RateLimiter` et `LoginGuard`,
  et blocage par paliers contre la force brute : une durée qui s'allonge à chaque récidive (5 min,
  puis 15 min, puis 1 h par défaut), chaque palier étant configurable.

Un élément prévu pour une version peut passer dans l'autre en cours de route, s'il faut les livrer
ensemble.

## Prévu

- **4.2, relations** : la table pivot du many-to-many est générée depuis le DSL du modèle, créée
  et supprimée par `makemigrations`, avec respect de cascade, restrict et set_null ; une table
  écrite à la main, avec des colonnes en plus, reste possible. Chargement des relations en une
  requête, méthodes d'accès du many-to-many et sa gestion dans l'admin. `search!` accepte aussi
  des groupes ET dans un OU, et la négation d'une condition ou d'un groupe, toujours avec des
  requêtes typées.
- **Open redirect** : tester la protection avec des charges connues de contournement d'URL.
- **Tracing** : finir la sortie fichier avec rotation et ajouter un identifiant de requête. Les
  clés de traduction manquantes seront journalisées.
- **Langues ajoutées par un projet** : permettre à un projet de fournir une langue absente de
  Runique, enregistrée par le builder.
- **Admin** : filtres par date et par clé étrangère, navigation par date, `fieldsets`, champs en
  lecture seule, autocomplétion, édition inline, actions de groupe en Rust, export CSV/JSON et
  « enregistrer et continuer ».

## Plus tard

Sans engagement : ces pistes remonteront quand un chantier se terminera.

- **Authentification** : OAuth/OIDC, JWT, clés d'API, journal des connexions et rapports de
  violation CSP.
- **ORM et DSL** : hooks de modèle, agrégats dans `search!`, index déclarés, bornes et paliers
  numériques.
- **Outils et stockage** : vues CRUD publiques, client de test intégré, fixtures, commandes
  personnalisées, sitemap et RSS, redimensionnement d'images, et point d'extension de stockage
  pour les services compatibles S3.
