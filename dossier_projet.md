# Runique — Dossier projet personnel

## 1. Présentation

Je m'appelle Sébastien Alliot, cuisinier en reconversion vers le développement web. Actuellement en formation Graduate Développeur Front-End, je code depuis deux ans. Mon parcours a débuté avec Python et Django, un écosystème que j'ai rapidement apprécié pour sa structure et sa productivité. Suite à un échec d'examen en décembre 2025 — dû à l'absence de diaporama plutôt qu'à des lacunes techniques — j'ai traversé une période de démotivation qui m'a paradoxalement conduit vers Rust. En cherchant à reconstruire un projet existant dans ce nouveau langage, j'ai rapidement constaté que les patterns se répétaient. C'est ce constat qui est à l'origine de Runique : plutôt que de copier-coller des solutions, autant construire un framework.

---

## 2. Problème posé

L'écosystème Rust web existant — Axum, Actix — est puissant mais bas niveau. Chaque projet repart de zéro : gestion des formulaires, sessions, CSRF, panel d'administration, migrations. Django avait résolu ces problèmes il y a vingt ans avec une approche "batteries included" que Rust n'avait pas. L'objectif de Runique n'est pas de surpasser Django — ce serait prétentieux pour un projet d'un développeur autodidacte de deux ans d'expérience — mais de transposer cette philosophie dans l'écosystème Rust : fournir un environnement complet et structuré, sans sacrifier les garanties du langage. La complexité du borrow checker, les choix entre `Mutex` et `RwLock`, la gestion de la concurrence : ces contraintes ne sont pas des obstacles, elles sont ce qui rend le résultat fiable. Runique est né de la conviction qu'on peut avoir la productivité de Django et la sûreté de Rust dans le même outil.

---

## 3. Architecture technique

Runique repose sur une architecture que j'ai découverte progressivement plutôt que conçue à l'avance. En factorisant le code au fil du développement, j'ai identifié un pattern récurrent en quatre étapes : récupération de la donnée, traitement, validation, persistance. Ce cycle s'est révélé universel dans le framework — il structure le moteur de formulaires, la gestion des requêtes HTTP, le pipeline de sécurité. Plutôt que d'imposer une architecture top-down, Runique a émergé de la répétition et de la simplification. C'est ce que le compilateur Rust encourage naturellement : quand quelque chose compile et reste lisible, c'est souvent que la structure est juste.

---

## 3b. Rust et la programmation orientée objet

Rust n'est pas un langage orienté objet au sens classique — il n'y a pas d'héritage de classes. À la place, les `struct` portent les données et les `trait` définissent les comportements partagés. C'est une forme de composition plutôt que d'héritage, plus proche de ce que font les interfaces en Java ou les protocoles en Swift. Runique exploite ce modèle intensivement : `RuniqueUser`, `RuniqueForm`, `AdminResource` sont tous des traits que l'utilisateur du framework implémente pour brancher son code sur les mécanismes internes. Ce choix de conception n'est pas une limitation — c'est ce qui rend le framework extensible sans couplage fort.

---

## 4. Fonctionnalité clé — Le système de slots

Le système de slots est la fonctionnalité qui m'a le plus coûté, et dont je suis le plus fier. En Rust, les middlewares HTTP doivent être enregistrés dans un ordre précis — les sessions avant le CSRF, la compression avant la gestion d'erreurs. Django masque cette contrainte par son dynamisme ; Rust l'expose brutalement. Après plusieurs refactorisations douloureuses du builder, j'ai cherché une solution qui soit à la fois structurée et souple. L'idée des slots est venue naturellement : chaque middleware se voit attribuer une priorité numérique fixe (la session occupe toujours le slot 50, le CSRF le 60), et le builder trie et assemble automatiquement à la construction. L'utilisateur déclare ses middlewares dans n'importe quel ordre — le framework garantit que l'ordre d'exécution est toujours correct. Ce n'est pas une contrainte imposée de l'extérieur, c'est une orchestration intégrée au builder lui-même.

---

## 5. Mise en production

Runique.io est à la fois la documentation du framework, une collection de cours Rust, et la première application déployée avec Runique lui-même. C'est un projet qui se documente et se démontre en se faisant tourner. Le déploiement sur VPS a été étonnamment simple comparé à Django : le binaire Rust est autonome, il peut fonctionner seul avec ACME pour le HTTPS automatique ou derrière un proxy Nginx. C'est là que la promesse bas niveau de Rust se concrétise. En revanche, héberger un deuxième projet sur le même VPS a révélé des frictions réelles : conflits de ports pour la validation ACME, bugs de configuration que seul un vrai environnement multi-projets pouvait faire apparaître. La production n'enseigne pas ce que le développement local peut simuler — elle enseigne ce qu'on n'avait pas prévu de simuler.

---

## 5b. Déploiement & environnement

Runique supporte deux modes de déploiement : autonome avec ACME (activé en feature Cargo à la compilation) pour la gestion automatique des certificats HTTPS, ou derrière un reverse proxy Nginx qui termine le TLS. Le binaire compilé n'a aucune dépendance runtime — il se déploie par simple copie sur le serveur.

La configuration est portée par des variables d'environnement (`.env`). La variable `DEBUG` conditionne la validation des hôtes autorisés. La redirection HTTPS est gérée indépendamment par `ENFORCE_HTTPS=true` — active uniquement derrière un reverse proxy qui termine le TLS. Une seule base de code couvre tous les environnements.

---

## 6. Fonctionnalités clés

### derive_form! — modèle, migration et entité en un bloc

Un seul DSL génère à la fois l'entité SeaORM, la migration SQL et le formulaire de validation. Aucune duplication entre le modèle et la base de données.

```rust
derive_form! {
    Article {
        fields: {
            titre: text [max_length: 200, required]
            contenu: text [required]
            publie: bool [default: false]
            auteur: fk(User) [required]
            image: file [max_size: 2MB, allowed: image]
        }
        meta: { table_name: "articles", ordering: ["-created_at"] }
    }
}
```

### admin!{} — panel CRUD généré

Une déclaration suffit à obtenir un panel d'administration complet : liste paginée, recherche, filtres, actions groupées, création, édition, suppression.

```rust
admin! {
    article: article::Model => ArticleForm {
        title: "Articles",
        list_display: [
            ["titre", "Titre"],
            ["auteur", "Auteur"],
            ["publie", "Publié"],
        ],
        list_filter: [
            ["publie", "Publié", 5],
            ["auteur", "Auteur", 5],
        ],
    }
}
```

### Sécurité intégrée — CSRF, CSP, sessions

La sécurité n'est pas une option à configurer manuellement — elle est activée par défaut et ordonnée par le système de slots. L'audit OWASP ZAP sur runique.io ne retourne aucune alerte critique.

```rust
.middleware(|m| m
    .with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
    .with_csp(|c| c
        .policy(SecurityPolicy::strict())
        .with_header_security(true)
        .with_upgrade_insecure(!is_debug())
    )
)
```

> La validation des hôtes est conditionnée à `!is_debug()`. La redirection HTTP→HTTPS est contrôlée séparément par `ENFORCE_HTTPS=true` dans le `.env` — elle nécessite un reverse proxy (Nginx, Caddy) qui transmet le header `X-Forwarded-Proto`. Si le proxy gère lui-même la redirection HTTPS, `ENFORCE_HTTPS` doit rester à `false` pour éviter une boucle de redirection.

---

## 7. Fiche technique

| | |
|---|---|
| **Langage** | Rust — edition 2024 |
| **Version** | 2.1.12 |
| **Runtime** | Tokio (async) |
| **HTTP** | Axum 0.8 |
| **ORM** | SeaORM 2.0.0-rc.38 |
| **Templates** | Tera 1.20.1 |
| **Bases de données** | PostgreSQL, MariaDB, SQLite |
| **Hébergement** | VPS — runique.io |
| **Sécurité** | Audit OWASP ZAP — 0 alerte critique |
| **Documentation** | Bilingue FR/EN — 88 pages |
| **Cours** | 31 chapitres Rust publiés |
| **Dépôt** | github.com/seb-alliot/runique |
| **Crate** | crates.io/crates/runique |

---

## 8. Conclusion

Construire Runique m'a appris trois choses sur moi-même en tant que développeur : la patience, la persistance, et la capacité à penser sur le long terme. Un framework ne se construit pas en un sprint — il se construit par couches successives, par refactorisations, par des bugs qui n'apparaissent qu'en production. Ce qui m'a le plus surpris, c'est que la motivation ne vient pas de la ligne d'arrivée mais du chemin : voir le projet grandir, voir une fonctionnalité prendre forme après des semaines de conception, c'est une satisfaction que je n'avais pas anticipée. Runique n'est pas terminé — il ne le sera probablement jamais complètement. Mais c'est précisément ce qui le rend vivant.
