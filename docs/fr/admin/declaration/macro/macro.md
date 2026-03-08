# La macro `admin!`

## Syntaxe complète

```rust
admin! {
    clé: chemin::Model => TypeFormulaire {
        title: "Titre affiché",
        permissions: ["rôle1", "rôle2"]
    },
    autre_clé: autre::Model => AutreForm {
        title: "Autre ressource",
        permissions: ["admin"]
    }
}
```

La macro est parsée par le daemon (`runique start`) qui génère la fonction `admin_register()` dans `src/admins/admin_panel.rs`. Cette fonction construit le `HashMap<String, ResourceEntry>` chargé au boot.

---

## Champs

### Obligatoires

| Champ | Type | Description |
| --- | --- | --- |
| `key` (position) | identifiant | Utilisé dans les routes `/admin/{key}/…` |
| `model` (position) | chemin de type | Ex: `users::Model` |
| `form` (position) | chemin de type | Type du formulaire Runique associé |
| `title` | `&str` | Titre affiché dans l'interface |
| `permissions` | `[&str; N]` | Rôles déclarés pour cette ressource (⚠️ non appliqués — voir [Permissions](../../permission/permissions.md)) |

### Optionnels — surcharge de templates

Ces champs permettent de remplacer un template CRUD par un template personnalisé (voir [Surcharge des templates](../../template/surcharge/surcharge.md)).

| Champ | Valeur par défaut | Description |
| --- | --- | --- |
| `template_list` | `admin/list.html` | Template de liste |
| `template_create` | `admin/create.html` | Formulaire de création |
| `template_edit` | `admin/edit.html` | Formulaire d'édition |
| `template_detail` | `admin/detail.html` | Page de détail |
| `template_delete` | `admin/delete.html` | Page de confirmation de suppression |

---

## Exemple avec plusieurs ressources

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["admin", "editor"]
    },
    comments: comments::Model => CommentForm {
        title: "Commentaires",
        permissions: ["moderator", "admin"]
    }
}
```

---

## Ce qui est déclarable

La macro `admin!` couvre uniquement les **métadonnées de registre** :

- l'identifiant de route de la ressource
- le modèle SeaORM utilisé
- le formulaire Runique pour la création/modification
- le titre d'affichage
- les rôles autorisés (uniformément sur toutes les opérations CRUD)

## Ce qui n'est pas déclarable

| Fonctionnalité | Raison de l'exclusion |
| --- | --- |
| Permissions par opération CRUD | Non supporté — les rôles s'appliquent globalement |
| Règles conditionnelles | Relève de la logique métier, à écrire dans le code généré |
| Rendu HTML / templates | Séparation des responsabilités |
| Filtres ou relations complexes | Trop spécifique pour être déclaratif |
| Logique d'authentification | Gérée par les middlewares admin |

---

## Erreurs compile-time

La macro inclut une vérification statique : si un type référencé (`Model` ou formulaire) n'existe pas dans le scope, **la compilation échoue** avec un message d'erreur explicite.

```text
error[E0412]: cannot find type `RegisterForm` in module `users`
```

Les champs obligatoires manquants produisent également des erreurs à la compilation, pas au runtime.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CLI](../cli.md) | Fonctionnement de `runique start`
| [Daemon & génération](../daemon/generation.md) | Fichiers générés

## Revenir au sommaire

- [Sommaire Admin](../../11-Admin.md)
