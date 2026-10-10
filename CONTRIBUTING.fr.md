🌍 **Langues** : [English](CONTRIBUTING.md) | [Français](CONTRIBUTING.fr.md)

# Contribuer à Runique

Merci de votre intérêt pour le projet ! Cette page liste ce qu'un changement doit remplir avant
d'être fusionné. Pour lancer les tests et le site de démonstration en local, voir
[TESTING.fr.md](TESTING.fr.md).

**Vous avez trouvé une faille de sécurité ?** N'ouvrez pas d'issue publique : suivez
[SECURITY.md](SECURITY.md).

---

## Organisation du dépôt

| Dossier | Contenu |
| --- | --- |
| `runique/` | Le framework (crate `runique`) |
| `runique/derive_form/` | Les macros `model!{}`, `extend!{}` et `#[form]` |
| `runique/runique_dsl/` | Le parseur du DSL des modèles, partagé par les macros et la CLI |
| `demo-app/` | Le site de démonstration (runique.io), Postgres uniquement |
| `docs/fr/`, `docs/en/` | La documentation servie par le site |

---

## Avant d'ouvrir une pull request

La CI lance ces vérifications ; lancez-les d'abord en local :

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --features orm,sqlite,postgres,mysql,mariadb,all-databases,acme -- -D warnings

cd runique
DB_ENGINE=postgres cargo clippy --all-targets --features big-pk -- -D warnings
DB_ENGINE=postgres cargo clippy --all-targets --features pk-uuid -- -D warnings
cargo test --features orm,sqlite,postgres,mysql,mariadb,all-databases,acme
cargo test --features big-pk,all-databases
cargo test --features pk-uuid,all-databases
```

Les avertissements sont des erreurs (`-D warnings`). Les trois passes de tests couvrent les
trois types de clé primaire (`i32`, `i64`, `Uuid`) : un test qui écrit des id à la main doit
utiliser les helpers `pk` de `runique/tests/helpers/pk.rs`, pas des entiers en dur.

---

## Code

- Rust edition 2024. Enchaînez les conditions avec des let-chains
  (`if let Some(x) = a && cond { … }`) plutôt que des `if` imbriqués.
- Les commentaires expliquent le **pourquoi**, quand il n'est pas évident (une contrainte
  cachée, un invariant). Pas de commentaire qui répète le code.
- Pas de refactoring sans gain concret (comportement, performance, sécurité).
- Une API publique supprimée ou modifiée est une rupture : elle va dans le CHANGELOG et dans le
  guide de migration (`MIGRATION-x.y.md`).

---

## Tests

- Chaque correctif de bug arrive avec un test qui échoue sans lui.
- Testez les deux côtés d'une condition : le cas accepté et le cas refusé, limite comprise.
- Les contrôles de sécurité se testent par une tentative qui doit échouer (mauvais jeton, id
  d'un autre utilisateur, corps trop gros…), pas seulement par le cas qui marche.

---

## Code généré

`src/admins/` (dans demo-app ou un projet) est généré depuis `admin!{}` par `runique start` :
ne le modifiez jamais à la main. Modifiez le générateur
(`runique/src/admin/daemon/generator.rs`), puis rafraîchissez son fichier de référence et
relisez le diff :

```bash
cd runique
RUNIQUE_UPDATE_GOLDEN=1 cargo test --lib admin::daemon::generator
```

---

## Documentation

- La documentation existe en français et en anglais : mettez à jour `docs/fr/` et `docs/en/`
  ensemble, ainsi que `CHANGELOG.md` et `CHANGELOG.fr.md`.
- Les liens internes sont vérifiés :

  ```bash
  cargo test -p demo-app internal_doc_links_resolve
  ```

---

## Versions et publication

Runique suit le versionnage sémantique, avec un cycle fixe :

1. **Contrôles de sécurité réguliers** sur l'existant. Les correctifs s'accumulent dans la
   section « Non publié » du CHANGELOG et sortent ensemble dans une **version corrective**
   (3.0.**x**).
2. **Une seule fonctionnalité à la fois**, développée jusqu'au « fini » (voir la
   [feuille de route](ROADMAP.fr.md)), publiée en **version mineure** (3.**x**.0).
3. **Un contrôle de sécurité ciblé sur cette fonctionnalité**, et ses correctifs en version
   corrective (3.x.**y**).
4. La fonctionnalité suivante ne commence qu'ensuite.
5. Une **rupture** d'API ne sort jamais dans une 3.x : elle attend la version majeure suivante
   (4.0), avec son guide de migration.

Une faille grave n'attend pas le regroupement : elle est corrigée et publiée tout de suite.

---

## Code sensible pour la sécurité

Pour tout changement touchant à l'authentification, aux permissions, aux sessions, aux jetons,
aux redirections ou aux entrées utilisateur, vérifiez explicitement :

- les attaques temporelles (comparer les secrets avec `ct_eq`, jamais `==`) ;
- les injections (SQL, HTML, templates) ;
- le contournement du contrôle d'accès (objet d'un autre utilisateur, bouton caché dont la
  route répond quand même) ;
- le CSRF ;
- l'open redirect ;
- l'exposition de données sensibles (logs, pages d'erreur, listes de l'admin).
