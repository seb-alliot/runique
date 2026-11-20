# CLI Runique

La CLI crée les projets, génère et applique les migrations, crée le premier superutilisateur et lance les tests métier. Elle se lance depuis la **racine du projet** (là où se trouve `.env`).

## Toutes les commandes

```bash
runique new <nom>                 # Créer un nouveau projet
runique makemigrations            # Écrire les migrations des modèles qui ont changé
runique makemigrations --force    # Pareil, en autorisant les changements destructifs
runique migration up              # Appliquer toutes les migrations en attente
runique migration down -n 1       # Annuler la dernière migration (son down() s'exécute)
runique migration status          # Lister les migrations et leur état
runique create-superuser          # Créer un superutilisateur
runique test [fichier] [test]     # Lancer src/runique_test/ (voir Tests)
runique --version                 # Version de la commande installée (aussi : -V)
```

`runique --version` affiche la version de Runique avec laquelle la commande a été compilée : c'est celle que `runique new` écrit dans le `Cargo.toml` d'un nouveau projet. Si elle diffère de la version de `runique` de votre projet, réinstallez la commande.

Pas de `runique start` : l'application se lance avec `cargo run`. L'admin se déclare en Rust (`ModelAdmin` + `.with_admin(...)`), sans daemon ni génération de code.

---

## `runique new`

Crée le projet complet : application, crate `migration/`, templates, CSS, `.env` avec un `SECRET_KEY` aléatoire. Voir [Prérequis & premier projet](/docs/fr/installation/prerequis).

Le projet est écrit en un seul lot : en entier, ou pas du tout. Après une coupure en plein milieu, relancez la même commande avec le même nom : elle retire le projet inachevé, puis le crée. Voir [Écriture atomique des fichiers](/docs/fr/architecture/atomic).

Noms refusés : vide, caractères autres que lettres / chiffres / `_` / `-`, commençant par un chiffre ou `-`, mots-clés Rust, crates intégrés (`std`, `core`, `alloc`, `proc_macro`, `test`), dossiers de cargo (`build`, `deps`, `examples`, `incremental`), noms de périphériques Windows (`con`, `nul`, `com1`…), et `migration` / `runique`. La comparaison ignore la casse et traite `-` comme `_`.

---

## `runique makemigrations`

```bash
runique makemigrations [--entities src/entities] [--migrations migration/src] [--force]
```

Lit les `model!{}` et `extend!{}` de `src/entities/`, les compare à leurs snapshots, et écrit les migrations. Si le crate `migration/` n'existe pas encore (projet non créé par `runique new`), il est créé, avec le moteur de `DB_ENGINE` (SQLite si la variable est absente). Détails : [Migrations](/docs/fr/installation/migrations).

La commande se connecte d'abord à la base du `.env` (`DATABASE_URL`, sinon les clés `DB_*`) : si elle ne répond pas, elle s'arrête avec une erreur et n'écrit rien. Elle ne modifie jamais la base ; c'est `runique migration up` qui applique.

Elle lit ensuite les migrations déjà appliquées (table `seaql_migrations`) et refuse de continuer si l'une d'elles n'est pas dans `migration/src/lib.rs` : la base appartient à un autre projet, ou un fichier de migration déjà appliqué a été supprimé. Les migrations du framework (comptes, sessions, historique) sont toujours reconnues. Le message liste les migrations en cause, et rien n'est écrit.

Avant tout calcul, chaque fichier `.rs` de `migration/src/snapshots/` et de `snapshots/runique/` est relu : un snapshot illisible, ou dont le nom de fichier n'est pas celui de la table qu'il décrit, arrête la commande. Les autres fichiers du dossier sont ignorés.

Les fichiers sont écrits en un seul lot. Un lancement interrompu par une coupure est réparé au lancement suivant, avant tout calcul.

---

## `runique migration up | down | status`

```bash
runique migration up [--migrations migration/src]
runique migration down [-n 1] [--migrations migration/src]
runique migration status [--migrations migration/src]
```

Lance le crate `migration` du projet (`cargo run` sur `migration/Cargo.toml`), qui lit `DATABASE_URL` dans `.env`. `sea-orm-cli` n'est pas nécessaire : la version de SeaORM qui applique les migrations est celle de `migration/Cargo.toml`, alignée sur Runique.

Ces trois commandes refusent de partir tant qu'un `makemigrations` interrompu n'a pas été réparé : relancez `runique makemigrations` d'abord.

---

## `runique create-superuser`

Crée un compte actif, superutilisateur, activé immédiatement. La commande lit la base comme l'application : `DATABASE_URL`, ou `DB_ENGINE` + `DB_*`, dans `.env`. Les migrations doivent être appliquées avant.

L'assistant enchaîne cinq étapes, avec des menus au clavier (flèches + Entrée) :

1. **Algorithme de hachage** : Argon2 (recommandé), Bcrypt, Scrypt, ou un fournisseur personnalisé
2. **Nom d'utilisateur** : refusé s'il est vide ou déjà pris
3. **Email** : refusé s'il est invalide ou déjà pris
4. **Mot de passe**, puis sa confirmation
5. **Récapitulatif** : confirmer et créer, changer d'algorithme, ou modifier le mot de passe

Le mot de passe doit faire **au moins 12 caractères** et contenir une minuscule, une majuscule, un chiffre et un caractère spécial. `Ctrl+C` annule à tout moment.

> La CLI s'exécute sans l'application : elle n'a pas accès à la `PasswordConfig` de `main.rs`. L'algorithme se choisit donc à chaque exécution. Choisissez le même que l'application ; à défaut, le hash est converti vers l'algorithme configuré à la première connexion réussie (mode `Auto`).
>
> **Fournisseur personnalisé** : indiquez le chemin d'un exécutable (pas un lien symbolique) qui lit le mot de passe sur **stdin** et écrit le hash sur **stdout**. Il est lancé sans shell et doit répondre en moins de 10 secondes.

---

## `runique test`

Lance les tests de `src/runique_test/`, chacun dans une transaction annulée à la fin. La commande refuse de tourner si la feature `test-utils` manque, ou si elle est déclarée dans `[dependencies]` au lieu de `[dev-dependencies]` (elle finirait alors dans le binaire de production). Voir [Tests](/docs/fr/architecture/testing).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Migrations](/docs/fr/installation/migrations) | Workflow de migration |
| [Troubleshooting](/docs/fr/installation/troubleshooting) | Résoudre les problèmes courants |

## Retour au sommaire

- [Installation](/docs/fr/installation)
