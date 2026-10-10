🌍 **Langues** : [English](ROADMAP.md) | [Français](ROADMAP.fr.md)

# Runique — Feuille de route

Peu de chantiers à la fois, chacun livré fini. La carte s'agrandira quand ceux-ci seront
terminés. Ce qui est déjà livré est dans le [CHANGELOG](CHANGELOG.fr.md).

> Cette feuille de route n'est pas figée. Des oublis sont possibles, et des correctifs seront
> nécessaires : un bug ou une faille découverts passent avant les chantiers prévus, et peuvent
> donner lieu à une version corrective à tout moment.

## Ce que « fini » veut dire

Une fonctionnalité n'entre dans une version publiée que si :

- elle est testée des deux côtés : le cas accepté et le cas refusé, limite comprise ;
- une passe `cargo mutants --in-diff` sur le code modifié est faite et chaque survivant est traité ;
- les contrôles de sécurité ont lieu **avant** l'effet (écriture, redirection, requête) ;
- elle a été vérifiée en vrai sur demo-app et sur un projet créé par `runique new` ;
- la documentation (fr/en) et le CHANGELOG sont à jour.

---

## En cours — 3.x

Sans rupture : un projet en 3.0 se met à jour sans modifier son code.

- [ ] **Relations many-to-many** — la table pivot est générée à partir du DSL (`through` garde
  une table pivot écrite à la main, avec ses colonnes en plus), migration comprise, et le modèle
  gagne ses méthodes d'accès.
- [ ] **Tracing** — finir la sortie fichier (rotation) et l'identifiant de requête ; terminer le
  balayage « zéro erreur avalée ». Les replis de traduction deviennent visibles : une clé absente
  de la langue courante (repli sur l'anglais) et une clé absente partout sont tracées.
- [ ] **Langues ajoutées par un projet** — un trait pour fournir une langue que Runique ne
  propose pas, enregistré via le builder, sans modifier l'enum `Lang`.
- [ ] **Corrections de bugs** — au fil de l'eau, chacune avec un test qui échoue sans elle.

---

## 4.0.0

Les ruptures, regroupées dans une seule version majeure, avec un guide de migration.

- [ ] **Refonte de l'admin : builder `ModelAdmin<Entity>` typé** — remplace `admin!{}` et le
  code généré par le daemon par une logique générique écrite une fois, et un fichier court par
  table. Colonnes typées, listes blanches uniquement, invariants vérifiés au démarrage.
  Ébauche : [ebauche-model-admin.md](ebauche-model-admin.md).
- [ ] **Uploads typés** — les fichiers envoyés sont rangés à part (`StagedFile`), jamais parmi
  les valeurs des champs texte : un fichier ne peut plus devenir la valeur d'un champ texte, et
  aucun chemin du serveur n'apparaît dans les données d'un formulaire.

### Construit sur le builder, après la refonte : parité avec l'admin Django

Chaque point deviendra une méthode typée du builder.

- **Liste** — filtres par date (aujourd'hui, cette semaine…), filtres à travers une clé étrangère,
  `date_hierarchy`, `list_display_links`, `list_editable`, tri par défaut par ressource,
  `empty_value_display`
- **Formulaires** — `fieldsets`, `readonly_fields`, `prepopulated_fields`, `autocomplete_fields`,
  « Enregistrer comme nouveau », lien vers la vue publique, redirection après enregistrement
  configurable
- **Actions** — actions de groupe avec une logique Rust, formulaire complémentaire d'action
- **Filtres et recherche** — filtres personnalisés, recherche à travers les tables liées, index
  générés pour les colonnes filtrées ou cherchées
- **Personnalisation** — requête de base par ressource, configuration par requête, assets
  CSS/JS par ressource
- **Inlines** — édition des objets liés dans le formulaire du parent (FK et many-to-many)
- **Divers** — export CSV/JSON, « Enregistrer et continuer », confirmation par `<dialog>`,
  plusieurs sites d'admin

---

## Ensuite — sans engagement

Ces pistes remonteront quand un chantier se terminera.

- **DSL et migrations** — paliers et bornes numériques (`step`, `min`, `max`), index déclarés,
  vrai `ALTER` lors d'un changement de type, clés étrangères cycliques
- **ORM** — hooks/signals (`before_save`, `after_save`…), agrégats dans `search!`,
  `.first()` qui renvoie `Option<T>`, plusieurs valeurs pour un même champ dans `search!`
- **Framework** — redirections typées (`redirect("nom_de_route")`, `redirect_external(url)` limité aux hôtes autorisés, `?next=` vérifié contre les routes déclarées), `crud!{}` pour les vues publiques, surcharge des champs `#[form]`, plusieurs
  connexions de types différents (TypeMap), middleware de détection de la langue, accès à
  `path_params` / `query_params` par des getters
- **Authentification** — OAuth/OIDC, JWT et clés d'API, journal des connexions, rapports de
  violation CSP
- **Outils** — client de test intégré, fixtures, commandes personnalisées, sitemap et RSS,
  redimensionnement d'images, documentation complète de l'API publique
