# CLI Runique

## Créer un superutilisateur

```bash
runique create-superuser
```

Interface en ligne de commande pour créer un projet, un superutilisateur, générer/appliquer les migrations et lancer les tests métier.

```
=== Créer un superutilisateur ===  [Ctrl+C pour quitter]

[1/5] Algorithme de hachage :
  1) Argon2  (recommandé)
  2) Bcrypt
  3) Scrypt
  4) Custom provider
Choix [1-4] (défaut: 1) :

[2/5] Username :
[3/5] Email :
[4/5] Mot de passe :
[5/5] Confirmer le mot de passe :

──────────────────────────────────
  Algorithme : Argon2
  Username   : admin
  Email      : admin@example.com
  Mot de passe : ••••••••
──────────────────────────────────
[Entrée] Confirmer  [A] Changer l'algo  [Ctrl+C] Annuler
```

**Navigation :** `ESC` ne fonctionne que sur l'étape de sélection de l'algorithme. Les étapes username/email/password (saisie texte) et l'écran de récapitulatif final n'ont pas de retour en arrière — `Ctrl+C` annule à tout moment.

> Le CLI s'exécute sans runtime applicatif — il n'a pas accès à la `PasswordConfig` configurée dans `main.rs`. L'algorithme est choisi explicitement à chaque exécution.
>
> Pour le cas `Custom`, fournissez un binaire ou script qui lit le mot de passe sur **stdin** et retourne le hash sur **stdout**.

---

## Toutes les commandes

```bash
runique new <nom>                                                    # Créer un nouveau projet
runique makemigrations --entities src/entities --migrations migration/src  # Générer les migrations
runique migration up --migrations migration/src                     # Appliquer les migrations (sea-orm-cli migrate up)
runique create-superuser                                            # Créer un superutilisateur
runique test [fichier] [test]                                       # Lancer src/runique_test/ (voir Tests)
```

Pas de `runique start` : l'app se lance avec `cargo run`. L'admin se déclare en Rust plain (`ModelAdmin` + `.with_admin(...)`), sans daemon ni génération.

Annuler ou lister les migrations passe par SeaORM : `sea-orm-cli migrate down -n N`, `sea-orm-cli migrate status`. `runique test` est décrit dans [Tests](/docs/fr/architecture/testing).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Migrations](/docs/fr/installation/migrations) | Workflow de migration |
| [Troubleshooting](/docs/fr/installation/troubleshooting) | Résoudre les problèmes courants |

## Retour au sommaire

- [Installation](/docs/fr/installation)
