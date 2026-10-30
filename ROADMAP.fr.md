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

## État actuel — 4.0.0 (non publiée)

Workspace en `4.0.0`. Le code de rupture est dans l'arbre ; la publication crates.io n'a pas
encore eu lieu.

### Livré dans cette branche

- [x] **Refonte de l'admin : builder `ModelAdmin<Entity>` typé** — remplace `admin!{}` et le
  code généré par le daemon. Colonnes typées, listes blanches, invariants au démarrage.
  Déclaration dans `src/admin.rs`, câblage `.with_admin(admin::config)`, `cargo run` suffit
  (plus de `runique start`, plus de `src/admins/`).
- [x] **Documentation alignée 4.0** — guides fr/en, README, migration, SECURITY, demo-app
  (`seed.sql`, templates hub/surcharge), INDEX et PROJECT_STATUS : l'API courante est
  `ModelAdmin`, pas l'ancien générateur. Le CHANGELOG historique 3.0.x garde les mentions
  « régénérez l'admin » là où c'était vrai à l'époque.
- [x] **Correctifs sécu déjà dans le CHANGELOG 4.0** — timing forgot-password, message générique
  sur superuser, `csrf.js` same-origin only, `password`/`is_active` hors formulaires `extend!`,
  page debug bornée à `templates/`, erreurs globales de formulaire échappées.

### Encore ouvert avant ou juste après la publication 4.0

Ruptures restantes (si elles partent avec la 4.0 ; sinon report en 5.0 avec guide) :

- [ ] **Action de suppression obligatoire** — un `belongs_to` sans `[cascade]`, `[set_null]` ou
  `[restrict]` ne compile plus, comme `on_delete` dans Django.
- [ ] **Uploads typés** — fichiers à part (`StagedFile`), jamais parmi les valeurs texte ; aucun
  chemin serveur dans les données de formulaire.

Le détail du cycle de publication est dans [CONTRIBUTING](CONTRIBUTING.fr.md#versions-et-publication).

---

## Entretien et sécurité — en continu

Ce travail n'a pas de fin : il passe avant les chantiers ci-dessous.

- **Contrôles de sécurité réguliers** sur l'existant : relecture des parcours sensibles (de
  l'entrée à l'effet), entrées hostiles, `cargo mutants` sur le code concerné.
- **Audit des dépendances** : `cargo audit` à chaque push, bloquant en CI.
- **Corrections de bugs**, chacune avec un test qui échoue sans elle.
- Correctifs regroupés en **patch** (`4.0.x` tant que la ligne 4.0 est supportée). Une faille
  grave est publiée sans attendre.

### Correctifs identifiés (prochaine patch 4.0.x)

**Ordre de travail imposé** — ces items partent **avant** toute mineure 4.x (M2M, makemigrations,
tracing, langues). Une faille grave peut sauter la file ; sinon on suit la numérotation. Chaque
item a son test de non-régression avant d'être coché.

1. [x] **Réinitialisation du mot de passe sans email dans le lien** — l'email était chiffré avec
   une clé tirée du jeton, présent dans le même lien : cela ne protégeait rien. Comparer l'email
   saisi à celui du compte en base et retirer l'email du lien. Les helpers maison
   `encrypt_email` / `decrypt_email` sont supprimés. Les tests vérifient les liens token-only et
   qu'un email incorrect ne consomme pas le jeton de reset.
2. [x] **Fichiers en transit d'un formulaire refusé** — supprimés avant le renvoi de la page quand
   le formulaire n'atteint pas `finalize` (jeton, autre champ, `clean()`). La requête supprime
   ensuite le dossier `.staging-*` qu'elle a créé, donc aussi quand le handler ne valide pas. Un
   dossier encore tenu par un processus vivant reste, quelle que soit la durée de l'envoi. Le
   multipart suivant retire celui qu'un processus mort a laissé. Les tests couvrent le champ
   voisin, le jeton forcé invalide, un formulaire non soumis, et un envoi accepté que la fin de
   requête ne retire pas.
3. [ ] **Compteurs en mémoire plafonnés** — `RateLimiter` et `LoginGuard` bornent leur nombre de
   clés entre deux purges.
4. [ ] **Sessions d'un superutilisateur** — protégées comme son compte : un staff ne peut pas les
   supprimer.
5. [ ] **Tests d'open redirect** — passer les charges utiles de l'antisèche PortSwigger
   (*URL validation bypass*).
6. [x] **Suppression refusée par une clé étrangère** — l'admin affiche « Impossible de supprimer :
   d'autres enregistrements l'utilisent encore. » La phrase est la même quel que soit le droit de
   lecture : aucun nom de table, aucun nombre. Le détail du moteur reste dans le journal. Par
   défaut, un `belongs_to` sans action refuse (`NoAction`) : jamais de cascade implicite.

Sortie typique : une ou plusieurs versions **patch** `4.0.x` (ex. `4.0.1`) tant que la ligne 4.0
est supportée. Pas de feature mineure mélangée dans ces patchs.

---

## Après 4.0 — mineures sans rupture (4.x)

**Prérequis** : la file des correctifs sécu / entretien `4.0.x` ci-dessus est traitée (ou une
faille urgente a déjà forcé un patch). Ensuite seulement : une fonctionnalité à la fois.

Un projet déjà en 4.0 se met à jour sans toucher son code. Une fonctionnalité à la fois, dans
cet ordre : chacune sort en mineure (4.**x**.0), passe un contrôle de sécurité ciblé, puis la
suivante commence.

- [ ] **Relations many-to-many** — la table pivot est générée à partir du DSL (`through` garde
  une table pivot écrite à la main, avec ses colonnes en plus), migration comprise, et le modèle
  gagne ses méthodes d'accès. *(L'admin gère déjà du M2M via table de jonction déclarée ; il
  manque la génération DSL + migration.)*
- [ ] **makemigrations finalisé** — plus bloqué par la refonte admin (livrée). Trous constatés
  le 2026-10-10 en migrant un vrai projet (Campanile) vers la 3.0 :
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

### Construit sur le builder : parité avec l'admin Django

Chaque point deviendra une méthode typée du builder (mineures 4.x, sans nouvelle rupture si
l'API reste additive).

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
- **Visuel admin** — consolidation mobile-first / CSS (déjà amorcée ; à peaufiner)

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
