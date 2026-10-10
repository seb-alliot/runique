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

## Entretien et sécurité — en continu

Ce travail n'a pas de fin : il passe avant les chantiers ci-dessous.

- **Contrôles de sécurité réguliers** sur l'existant : relecture des parcours sensibles (de
  l'entrée à l'effet), entrées hostiles, `cargo mutants` sur le code concerné.
- **Audit des dépendances** : `cargo audit` à chaque push, bloquant en CI.
- **Corrections de bugs**, chacune avec un test qui échoue sans elle.
- Ces correctifs sont regroupés dans une **version corrective** (3.x.**y**). Une faille grave
  est publiée sans attendre.

Correctifs déjà identifiés, pour une prochaine 3.0.x :

- [ ] **Réinitialisation du mot de passe sans email dans le lien** — l'email est aujourd'hui
  chiffré avec une clé tirée du jeton, présent dans le même lien : ça ne protège rien. Comparer
  l'email saisi à celui du compte en base et retirer l'email du lien ; `encrypt_email` /
  `decrypt_email` marqués dépréciés (supprimés en 4.0).
- [ ] **Fichiers en transit d'un formulaire refusé** — supprimés tout de suite, même quand c'est
  un autre champ qui rend le formulaire invalide (aujourd'hui : au nettoyage suivant, 1 h).
- [ ] **Compteurs en mémoire plafonnés** — `RateLimiter` et `LoginGuard` bornent leur nombre de
  clés entre deux purges.
- [ ] **Sessions d'un superutilisateur** — protégées comme son compte : un staff ne peut pas les
  supprimer.
- [ ] **Tests d'open redirect** — passer les charges utiles de l'antisèche PortSwigger
  (*URL validation bypass*).
- [ ] **Suppression refusée par une clé étrangère** — l'admin affiche un message clair (« impossible
  de supprimer : 3 commandes y sont liées ») au lieu de l'erreur brute de la base. Par défaut, un
  `belongs_to` sans action refuse (`NoAction`) : jamais de cascade implicite.

Le détail du cycle de publication est dans [CONTRIBUTING](CONTRIBUTING.fr.md#versions-et-publication).

---

## En cours — 3.x

Sans rupture : un projet en 3.0 se met à jour sans modifier son code. Une fonctionnalité à la
fois, dans cet ordre : chacune sort en version mineure (3.**x**.0), passe ensuite un contrôle de
sécurité ciblé, puis la suivante commence.

- [ ] **Relations many-to-many** — la table pivot est générée à partir du DSL (`through` garde
  une table pivot écrite à la main, avec ses colonnes en plus), migration comprise, et le modèle
  gagne ses méthodes d'accès.
- [ ] **makemigrations finalisé** — à terminer avant la refonte de l'admin, qui en dépend.
  Trous constatés le 2026-10-10 en migrant un vrai projet (Campanile) vers la 3.0 :
  - une colonne **renommée et modifiée** en même temps (`renamed_from` + `NOT NULL` / défaut)
    ne reçoit que le renommage ; la modification est perdue, et l'instantané l'enregistre comme
    faite — un nouveau `makemigrations` ne la verra jamais ;
  - les noms de contraintes sont **supposés** (`{table}_{colonne}_{cible}_fkey`) au lieu d'être
    **lus en base** : une base créée avec une autre convention fait échouer la migration ;
  - une colonne qui passe en `NOT NULL` avec une valeur par défaut doit voir ses `NULL`
    existants remplis avant la contrainte (aujourd'hui : refus, à faire à la main) ;
  - la limite de longueur des identifiants doit dépendre du moteur (63 pour Postgres, 64 pour
    MariaDB/MySQL) — aujourd'hui 63 partout, avec un message qui annonce 64 ;
  - vrai `ALTER` lors d'un changement de type ; table pivot du many-to-many.
- [ ] **Tracing** — finir la sortie fichier (rotation) et l'identifiant de requête ; terminer le
  balayage « zéro erreur avalée ». Les replis de traduction deviennent visibles : une clé absente
  de la langue courante (repli sur l'anglais) et une clé absente partout sont tracées.
- [ ] **Langues ajoutées par un projet** — un trait pour fournir une langue que Runique ne
  propose pas, enregistré via le builder, sans modifier l'enum `Lang`.

---

## 4.0.0

Les ruptures, regroupées dans une seule version majeure, avec un guide de migration.

- [ ] **Refonte de l'admin : builder `ModelAdmin<Entity>` typé** — remplace `admin!{}` et le
  code généré par le daemon par une logique générique écrite une fois, et un fichier court par
  table. Colonnes typées, listes blanches uniquement, invariants vérifiés au démarrage.
  Ébauche : [ebauche-model-admin.md](ebauche-model-admin.md).
- [ ] **Action de suppression obligatoire** — un `belongs_to` sans `[cascade]`, `[set_null]` ou
  `[restrict]` ne compile plus, comme `on_delete` dans Django : chaque relation est un choix
  explicite.
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
  clés étrangères cycliques
- **ORM** — hooks/signals (`before_save`, `after_save`…), agrégats dans `search!`,
  `.first()` qui renvoie `Option<T>`, plusieurs valeurs pour un même champ dans `search!`
- **Framework** — redirections typées (`redirect("nom_de_route")`, `redirect_external(url)` limité aux hôtes autorisés, `?next=` vérifié contre les routes déclarées), `crud!{}` pour les vues publiques, surcharge des champs `#[form]`, plusieurs
  connexions de types différents (TypeMap), middleware de détection de la langue, accès à
  `path_params` / `query_params` par des getters
- **Stockage** — un point d'extension `StorageBackend` (disque local par défaut, S3/R2 ou autre
  service en option) partagé par les médias, les uploads en transit et les futurs exports.
  En attendant, un stockage externe monté comme dossier (`s3fs`, `rclone mount`, NFS) sert
  de `MEDIA_ROOT`
- **Authentification** — protection contre la force brute à deux niveaux, sans rupture : blocage par nom d'utilisateur + IP (le vrai utilisateur, ailleurs, peut toujours se connecter), et ralentissement par nom d'utilisateur toutes adresses confondues (contre les attaques réparties sur beaucoup d'IP) ; une seule fonction partagée, appelée par `LoginGuard` et par la connexion à l'admin. Puis OAuth/OIDC, JWT et clés d'API, journal des connexions, rapports de
  violation CSP
- **Outils** — client de test intégré, fixtures, commandes personnalisées, sitemap et RSS,
  redimensionnement d'images, documentation complète de l'API publique
