
---
* architecture
* workflow daemon
* permissions basées sur `users`
* **section claire sur ce qui est déclarable dans la macro**
* limites explicites et assumées

Tu peux la **copier-coller telle quelle**.

---

##  Vue d’administration (bêta)

La vue d’administration de Runique repose sur une **macro déclarative (`admin!`)** combinée à un **daemon de génération**.

L’objectif est de proposer une approche **transparente, auditable et type-safe** :
le code admin généré est du Rust “normal”, lisible, inspectable et modifiable si nécessaire.

---

## 1) Déclaration des ressources via `admin!`

Le développeur déclare ses ressources administrables dans le fichier `src/admin.rs`.

Chaque ressource est définie par :

* une **clé** (`users`, `blog`, …)
* un **modèle** (chemin de type Rust)
* un **formulaire**
* un **titre**
* une liste de **rôles** autorisés

Exemple :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin", "staff"]
    }
}
```

La macro génère une fonction `admin_config()` qui construit un `AdminRegistry` et y enregistre chaque ressource via `AdminResource`.

 **Type-safe**
La macro inclut une vérification *compile-time* : si un modèle ou un formulaire référencé n’existe pas, la compilation échoue avec une erreur explicite.

---

## 2) Ce qui est déclarable dans la macro `admin!`

La macro `admin!` permet de déclarer **uniquement les métadonnées essentielles** d’une ressource admin.
Elle ne décrit **ni la logique métier**, ni l’authentification, ni le rendu HTML.

### Champs supportés

Pour chaque ressource, les champs suivants sont **obligatoires** :

| Champ         | Description                                                            |
| ------------- | ---------------------------------------------------------------------- |
| `key`         | Identifiant de la ressource (utilisé dans les routes `/admin/{key}/…`) |
| `model`       | Chemin du modèle Rust (ex: `users::Model`)                             |
| `form`        | Type du formulaire Runique                                             |
| `title`       | Titre affiché dans l’interface admin                                   |
| `permissions` | Liste de rôles autorisés                                               |

### Permissions

```rust
permissions: ["admin", "staff"]
```

* Les permissions sont exprimées sous forme de **rôles**
* La liste s’applique **uniformément à toutes les opérations CRUD** dans la version actuelle

###  Ce qui n’est pas déclarable (volontairement)

La macro `admin!` ne permet pas de déclarer :

* des permissions différentes par opération CRUD
* des règles conditionnelles
* du rendu HTML ou des templates
* de la logique métier
* des filtres ou relations complexes

Ces limites sont **assumées** : la macro reste simple et lisible, la logique réside dans le code Rust généré.

---

## 3) Parsing : lecture de `src/admin.rs`

Lors de l’exécution de `runique start`, le daemon :

* lit `src/admin.rs`
* parse la macro `admin! { ... }` via `syn`
* extrait les ressources sous forme de `ResourceDef`

Chaque ressource contient notamment :

* `key`
* `model_type`
* `form_type`
* `title`
* `permissions`

Le parser valide la présence des champs obligatoires et remonte des erreurs explicites en cas de syntaxe invalide.

---

## 4) Génération : création de `src/admins/`

À partir des ressources parsées, Runique génère automatiquement le dossier suivant :

```
src/admins/
  ├─ README.md
  ├─ mod.rs
  ├─ router.rs
  └─ handlers.rs
```

* **`router.rs`** : routes CRUD (`list`, `create`, `detail`, `edit`, `delete`)
* **`handlers.rs`** : handlers SeaORM + formulaires (GET/POST, validation, rendu)
* **`mod.rs`** : point d’entrée du module admin
* **`README.md`** : avertissement indiquant que le dossier est auto-généré

---

## 5) Daemon / watcher : régénération automatique

La commande `runique start` lance un watcher (basé sur `notify`) qui surveille `src/admin.rs`.

À chaque modification détectée :

1. le fichier est relu
2. la macro est parsée
3. le dossier `src/admins/` est régénéré
4. un retour simple est affiché (✅ ou ❌)

Un mécanisme de *debounce* empêche les régénérations multiples lors d’un même enregistrement.

---

##  Compromis assumé : écrasement du dossier `src/admins/`

Ce workflow implique un compromis volontaire :

* `runique start` **supprime et régénère entièrement** le dossier `src/admins/`
* toute modification manuelle dans ce dossier sera **écrasée**

Si des modifications manuelles sont nécessaires, il faut **ne pas utiliser `runique start`** et basculer sur un workflow `cargo run` afin d’éviter toute régénération automatique.

---

##  Permissions et rôles (basés sur la table `users`)

Le système de permissions repose sur l’utilisateur authentifié et les données stockées dans la **table `users`**.

Les contrôles d’accès s’appuient notamment sur :

* **`is_active`** : l’utilisateur doit être actif
* **`is_staff`** : autorisation d’accès à l’admin
* **`is_superuser`** : accès total
* **`roles`** *(optionnel)* : rôles personnalisés (ex: `"admin"`, `"editor"`)

Les permissions déclarées dans la macro :

```rust
permissions: ["admin", "staff"]
```

sont comparées aux rôles et attributs de l’utilisateur courant.
Un utilisateur est autorisé s’il possède **au moins un rôle compatible** ou un statut équivalent (ex: superuser).

>  Le champ `roles` permet une gestion souple sans imposer de schéma rigide.

---

##  Remarques importantes

* La macro `admin!` définit des **règles déclaratives**, pas la logique d’authentification
* Les vérifications sont effectuées **au runtime** via les middlewares admin
* La table `users` reste la **source de vérité** pour l’autorisation

---

##  État actuel (bêta)

* Génération automatique des routes et handlers CRUD
* Registre central de ressources (`AdminRegistry`)
* Permissions globales par ressource
* Feedback principalement **structurel** (daemon absent, fichier manquant, déclaration invalide)

L’amélioration du retour d’erreur, de la granularité des permissions et de la sécurité du workflow fait partie des axes d’évolution.

---

###  Conclusion

Cette architecture privilégie :

* la **lisibilité**
* la **sécurité par le typage**
* le **contrôle du développeur sur le code généré**

La vue admin est volontairement simple, explicite et évolutive.

---
