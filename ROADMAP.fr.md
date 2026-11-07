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

- **Publication de la 4.0** : finalisation avant la mise en ligne sur crates.io, avec des tests
  sur des comptes à faibles droits et des groupes, et une revue OWASP complète.

## Prévu

- **Correctifs de sécurité (4.0.x)** : plafonner le nombre de clés gardées en mémoire par
  `RateLimiter` et `LoginGuard`, et tester la protection contre les open redirects avec des
  charges connues de contournement d'URL.
- **4.1, makemigrations** : générer toutes les opérations de schéma sans étape manuelle : suppression
  de tables, création et suppression des tables de relation, respect de cascade, restrict et
  set_null, vrai `ALTER` lors d'un changement de type, détection des changements de valeur par
  défaut, et colonnes à la fois renommées et modifiées. Les noms de contraintes sont lus en base,
  et les changements destructeurs indiquent ce qu'ils touchent.
  La table pivot du many-to-many est générée depuis le DSL du modèle ; une table écrite à la main,
  avec des colonnes en plus, reste possible.
- **4.2, base de données** : `get_or_create`, `update_or_create`, `exists()`, `last()`,
  chargement des relations en une requête,
  méthodes d'accès du many-to-many et sa gestion dans l'admin. `search!` accepte aussi des groupes
  ET dans un OU, et la négation d'une condition ou d'un groupe, toujours avec des requêtes typées.
- **Tracing** : finir la sortie fichier avec rotation, ajouter un identifiant de requête et ne
  laisser aucune erreur ignorée en silence. Les clés de traduction manquantes seront journalisées.
- **Langues ajoutées par un projet** : permettre à un projet de fournir une langue absente de
  Runique, enregistrée par le builder.
- **Admin** : filtres par date et par clé étrangère, navigation par date, `fieldsets`, champs en
  lecture seule, autocomplétion, édition inline, actions de groupe en Rust, export CSV/JSON et
  « enregistrer et continuer ».

## Plus tard

Sans engagement : ces pistes remonteront quand un chantier se terminera.

- **Authentification** : protection à deux niveaux contre la force brute, puis OAuth/OIDC, JWT,
  clés d'API, journal des connexions et rapports de violation CSP.
- **ORM et DSL** : hooks de modèle, agrégats dans `search!`, index déclarés, bornes et paliers
  numériques.
- **Outils et stockage** : vues CRUD publiques, client de test intégré, fixtures, commandes
  personnalisées, sitemap et RSS, redimensionnement d'images, et point d'extension de stockage
  pour les services compatibles S3.
