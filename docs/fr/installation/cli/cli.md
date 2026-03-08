# CLI Runique

## Créer un superutilisateur

```bash
runique create-superuser
```

La commande est entièrement interactive :

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

**Navigation :** `ESC` revient à l'étape précédente à tout moment.

> Le CLI s'exécute sans runtime applicatif — il n'a pas accès à la `PasswordConfig` configurée dans `main.rs`. L'algorithme est choisi explicitement à chaque exécution.
>
> Pour le cas `Custom`, fournissez un binaire ou script qui lit le mot de passe sur **stdin** et retourne le hash sur **stdout**.

---

## Toutes les commandes

```bash
runique new <nom>                                                    # Créer un nouveau projet
runique start [--main src/main.rs] [--admin src/admin.rs]           # Lancer avec daemon admin
runique makemigrations --entities src/entities --migrations migration/src  # Générer les migrations
runique migration up|down|status --migrations migration/src         # Gérer les migrations
runique create-superuser                                            # Créer un superutilisateur
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Migrations](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/migrations/migrations.md) | Workflow de migration |
| [Troubleshooting](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/troubleshooting/troubleshooting.md) | Résoudre les problèmes courants |

## Retour au sommaire

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)
