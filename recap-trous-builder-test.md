# Trous du builder de test (`runique_test`)

> **État au 2026-09-29, fin de journée** : T1, T2, T4, T5, T6, T7, T9, T12 et T13 (message) sont corrigés et testés (20 tests dans `runique/tests/test_builder/`). Les 11 mutations ont toutes été détectées. T10 existait déjà (avertissement dans `runique_app.rs`). **Restent** : T8 (en discussion), T3 et T11 (doc), et, dans T13, `nextest`, le nom du test et le type d'erreur du handler (doc).

> Inventaire du 2026-09-29, fait en lisant le code du builder et les sources de SeaORM 2.0.4, sqlx et tower-sessions. Rien n'est corrigé.
> Chaque trou a une reproduction (pour le casser), un niveau de gravité et des pistes de correction. **Les décisions sont à toi.**

Gravité :
- 🔴 **critique** : le test passe au vert alors que des données restent en base, ou toute la suite reste bloquée.
- 🟠 **élevé** : fuite entre tests, mauvaise cible, ou valeurs sensibles affichées.
- 🟡 **moyen** : effet de bord ou angle mort à documenter.
- ⚪ **ergonomie**

---

## 🔴 T1. Un `COMMIT` caché rend le rollback inopérant, et le test reste vert

Trois façons de mettre fin à la transaction du test sans que le builder le sache :
- un `COMMIT` ou un `BEGIN` en SQL brut : `db.execute_unprepared("COMMIT")` ;
- **sous MariaDB/MySQL, n'importe quel DDL** (`CREATE`, `ALTER`, `DROP`, `TRUNCATE`, `RENAME`, `LOCK TABLES`…). MariaDB valide aussitôt, de façon implicite, tout ce que la transaction avait écrit avant ;
- `SET autocommit = 1`.

Ensuite, la connexion passe en mode « chaque requête est validée tout de suite ». Notre `ROLLBACK` final ne trouve plus de transaction :

| Moteur | Réaction du `ROLLBACK` final | Le test voit-il le problème ? |
|---|---|---|
| Postgres | simple WARNING, puis succès | ❌ non, test **vert** |
| MariaDB/MySQL | aucun effet, succès | ❌ non, test **vert** |
| SQLite | erreur « no transaction is active » | ✅ oui (à confirmer) |

Pire encore : **`execute_unprepared` n'apparaît pas dans la trace**. SeaORM n'appelle son callback de mesure sur aucun moteur pour cet appel (vérifié dans ses pilotes et dans `transaction.rs`). Le `COMMIT` est donc invisible.

**Pour le casser :**
```rust
runique_test::<ADb>(ENV, async |db| {
    new_user("t1_leak").insert(db).await?;
    db.execute_unprepared("COMMIT").await?;   // or, on MariaDB: "CREATE TABLE t1 (x INT)"
    Ok(())
}).await
// → green, and the user t1_leak is still in the database
```

**Pistes :**
- **Sonde « la transaction est-elle toujours vivante ? »**, juste avant le rollback. Nouvelle méthode du trait, avec une implémentation par défaut qui renvoie `Ok`.
  - Postgres : relever `txid_current()` après `begin_test`, puis comparer juste avant le rollback. Si le numéro a changé, la transaction a été terminée.
  - MariaDB/MySQL : `SELECT @@in_transaction` (disponibilité à vérifier selon la version de MariaDB).
  - SQLite : l'erreur du rollback suffit peut-être déjà (à confirmer par un test).
- **Tracer nous-mêmes `execute_unprepared`.** `ConnectionTrait` pour `ADb` est **notre** implémentation (`db/adb.rs`), on peut y enregistrer la requête dans la trace. Voir T12.
- Sous MariaDB, repérer les mots-clés DDL dans la trace et faire échouer le test franchement.

---

## 🔴 T2. Les erreurs avalées donnent des faux positifs

Le code métier avale souvent les erreurs de la base : `.unwrap_or(None)`, `.unwrap_or_default()`, `.ok()`. Une requête qui échoue devient alors « aucun résultat ». Le handler ne voit rien, et le test passe.

**Exemple réel dans nos propres tests :** `user::unknown_username_is_not_found` appelle `find_user_by_username`, qui fait `result.unwrap_or(None)`. Si la requête échoue (table absente, colonne renommée, base cassée), elle renvoie `None`, et **le test passe quand même**.

Aggravant sous Postgres : après une requête échouée hors savepoint, toute la transaction est bloquée. Toutes les requêtes suivantes échouent, et, avalées, elles passent pour des résultats vides.

**Pour le casser :** renommer la table `eihwaz_users` dans le handler (ou pointer vers une base sans schéma), puis lancer `unknown_username_is_not_found`. Il reste vert.

**Pistes :**
- La trace sait déjà quelles requêtes ont échoué (`failed: true`). Règle possible : si **au moins une requête a échoué alors que le handler renvoie `Ok`**, le verdict devient suspect.
  - échec du test par défaut ? ou simple avertissement `⚠ 1 requête a échoué mais le handler a réussi` ?
  - Il faut une porte de sortie pour les échecs voulus, comme `insert_expecting_rejection` : par exemple ignorer les échecs survenus dans un savepoint annulé, ou une API explicite.
- C'est exactement le sujet du chantier « zéro erreur avalée », vu depuis les tests.

**Décision :** échec strict, avertissement, ou option de la CLI (`runique test --strict`) ?

---

## 🔴 T3. Le code qui passe par une autre connexion échappe à la transaction

Tout ce qui n'utilise pas le `db` reçu par le handler écrit **pour de vrai** :
- `engine.db` ;
- une connexion stockée dans un `static` ou un `OnceLock` ;
- un `RuniqueSessionStore` construit avec une autre connexion ;
- une tâche de fond avec sa propre connexion.

Ces écritures n'apparaissent pas non plus dans la trace.

**Pour le casser :** un handler qui appelle une fonction métier écrivant via `engine.db`. La ligne reste en base.

**Pistes :** impossible à intercepter en général. Il faut une règle documentée : **le code métier reçoit toujours `db: &ADb` en paramètre**. On pourrait détecter certains cas sous Postgres (une autre connexion active avec le même `application_name`), mais c'est du bricolage.

---

## 🔴 T4. Blocage infini : toute la suite reste pendue

Deux façons d'y arriver :
1. **Verrou croisé.** La transaction du test verrouille une ligne. Le code métier la relit ou la modifie via **une autre connexion** (T3). Cette connexion attend la fin de notre transaction, qui attend la fin du handler. Postgres attend indéfiniment par défaut.
2. **`runique_test` appelé depuis un handler.** Le verrou `SERIAL` n'est pas réentrant, donc blocage immédiat.

Il n'y a **aucun délai maximal**. Le test ne finit jamais, et `SERIAL` bloque aussi tous les tests suivants.

**Pour le casser :**
```rust
runique_test::<ADb>(ENV, async |_db| {
    runique_test::<ADb>(ENV, async |_| Ok(())).await.ok();  // never returns
    Ok(())
}).await
```

**Pistes :**
- Un **délai maximal** sur le handler (`tokio::time::timeout`, 30 s par défaut, configurable dans le `.env` de test). Une fois dépassé : rollback, trace affichée, et `✗ handler timed out after 30s`.
- Détecter l'appel imbriqué avec un marqueur propre à la tâche (`tokio::task_local!`), et renvoyer `Err` tout de suite.
- En plus, sous Postgres : `SET LOCAL lock_timeout` sur la transaction du test.

---

## 🟠 T5. Un clone de la connexion qui survit repousse le rollback, et déborde sur les tests suivants

`db.clone()` déplacé dans un `tokio::spawn` : `rollback_test` échoue avec `clone_outlived` (c'est déjà géré), mais :
- la transaction reste ouverte tant que la tâche tourne. Elle continue d'écrire, et ne sera annulée qu'à la fin de la tâche ;
- `SERIAL` est pourtant libéré, et le test suivant démarre pendant ce temps :
  - sous Postgres, il attend sur les verrous encore tenus ;
  - sous SQLite (fichier), il reçoit une erreur « database is locked ».
- une panique dans la tâche lancée n'est pas rattrapée : elle s'affiche, et le test peut passer.

Aucun test ne déclenche encore `clone_outlived`.

**Pour le casser :**
```rust
runique_test::<ADb>(ENV, async |db| {
    let db = db.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        new_user("t5_late").insert(&db).await.ok();
    });
    Ok(())
}).await
```

**Pistes :**
- Au lieu de `Arc::into_inner` immédiat : **attendre**, avec un délai maximal, que le clone soit relâché (vérifier `Arc::strong_count` à intervalles), puis faire le rollback, en gardant `SERIAL` jusque-là.
- Le test `clone_outlived`, prévu dans l'étape 2.

---

## 🟠 T6. Les caches en mémoire gardent des données annulées

Le rollback annule la base, **pas la mémoire du processus**. Un test qui passe par `login()` remplit `PERMISSIONS_CACHE` (`auth/guard.rs`) avec des groupes qui n'existent plus après le rollback. Le test suivant, dans le même binaire, les lit.

Même risque pour tout cache en `static`/`LazyLock` de Runique ou de l'application (le cache `groupes_droits`, etc.).

**Pour le casser :** test A : `login()` d'un utilisateur créé dans le test, avec des groupes. Test B : `get_permissions(même id)` → `Some(...)`.

**Pistes :**
- Le builder vide les caches **de Runique** après chaque test : une fonction `reset_runique_caches()`, qui fait la liste de tous les caches à vider.
- Pour les caches de l'application : un point d'accroche que le développeur branche lui-même.

---

## 🟠 T7. On ne teste pas toujours la base qu'on croit

1. **Repli sur l'environnement du processus.** Une clé absente du `.env` est lue dans le shell. Si le `.env` n'a pas `DATABASE_URL` et que ton shell en exporte un (vers la prod, par exemple), c'est lui qui est utilisé. On peut même obtenir une cible hybride : `DB_ENGINE` venant du fichier, `DB_HOST` du shell.
2. **La cible n'est affichée qu'une fois** (`TARGET_SHOWN`). Si deux tests utilisent deux fichiers `.env` différents, seul le premier est annoncé.
3. **Le chemin relatif dépend du dossier courant.** `cargo test` se place bien dans le dossier du package, mais un autre lanceur peut lire le `.env` d'un autre dossier, par exemple celui de la racine du workspace, sans rien dire.

**Pistes :**
- Pas de repli sur le shell pour les clés de connexion à la base : le fichier doit se suffire à lui-même. Ou alors, afficher la provenance de chaque clé.
- Réafficher la cible **chaque fois qu'elle change**.
- Résoudre le chemin par rapport à `CARGO_MANIFEST_DIR`, lu au moment de l'exécution, et afficher le chemin absolu dans la ligne de la cible.

---

## 🟠 T8. Rien n'empêche de lancer les tests sur la prod

Le rollback protège les écritures, **sauf** dans les cas T1, T3, T5 et T11. Et les verrous pris pendant un test bloquent le trafic réel.

**Pistes (décision en attente depuis la revue) :**
- refuser si `DEBUG` n'est pas vrai dans le `.env` de test ;
- ou demander un feu vert explicite dans le `.env`, par exemple `RUNIQUE_TEST_TARGET=dev` ;
- **pas** de règle sur le nom de la base : ça casserait demo-app, dont la base s'appelle `runique`.

---

## 🟠 T9. Des valeurs sensibles peuvent s'afficher dans la console

La promesse « que des placeholders, aucune valeur » a des trous :
- **SQL brut construit avec `format!`** : les valeurs sont dans le texte de la requête, donc dans la trace.
- **Les messages d'erreur de MariaDB contiennent les valeurs** : `Duplicate entry 'alice@exemple.com' for key 'email'`. Postgres et SQLite ne les incluent pas (vu en réel pour Postgres hier).
- **`mask_password` ne masque que la partie `utilisateur:motdepasse@`.** Une URL comme `postgres://hote/base?user=x&password=secret` s'affiche en clair dans la ligne `runique test → …`.

**Pistes :**
- `describe()` : analyser l'URL avec la crate `url`, masquer le mot de passe et les paramètres `password`/`pass`/`pwd`.
- Documenter le cas des messages de MariaDB et du SQL brut. Nettoyer automatiquement les messages d'erreur n'est pas fiable.

---

## 🟡 T10. `test-utils` livré en release en passant à côté de la CLI

La CLI refuse `test-utils` dans `[dependencies]`, mais un simple `cargo build` ou `cargo run` ne passe pas par elle.

**Piste simple et fiable :** avec le resolver 2 (par défaut depuis l'édition 2021), les features d'une dev-dependency **ne sont jamais activées** dans un `cargo build` ou `cargo run` normal. Donc, si `RuniqueAppBuilder::build()` tourne **avec** la feature `test-utils`, c'est forcément qu'elle a été livrée. On peut alors :
- afficher un avertissement bien visible ;
- ou refuser de démarrer (`#[cfg(feature = "test-utils")]` dans `build()`).

---

## 🟡 T11. Les effets de bord hors base ne sont pas annulés

- **Fichiers envoyés** : un `FileField` écrit dans `MEDIA_ROOT`, et le fichier reste.
- **Mails** (`dispatch_email`), **appels HTTP**, **hooks `ActiveModelBehavior`** qui déclenchent des actions externes.
- **Séquences et auto-increment** : les identifiants consommés ne sont pas rendus. Sans gravité, mais on voit des trous dans la numérotation de la base de dev.

**Pistes :** doc (déjà sur la liste) ; un `MEDIA_ROOT` temporaire pendant les tests ; plus tard, un mailer de test qui garde les mails en mémoire, comme le backend `locmem` de Django.

---

## 🟡 T12. Des requêtes invisibles dans la trace

- `execute_unprepared` n'est pas tracé par SeaORM (voir T1).
- Les requêtes faites sur d'autres connexions (voir T3).

**Piste :** tracer `execute_unprepared` nous-mêmes, dans notre `impl ConnectionTrait for ADb`. Il faut pour cela que la transaction de test connaisse la trace, par exemple en ajoutant un champ facultatif à la variante `RuniqueDb::Txn`.

---

## ⚪ T13. Ergonomie et cas limites

- **`TestFailure` ne garde que le nom du test** : `expected_failures.rs` ne peut pas vérifier *pourquoi* un test a échoué. On pourrait ajouter le message (demandé aussi par la revue).
- **Le handler doit renvoyer `C::Error` (`DbErr`)** : du code métier qui renvoie `AppError` doit tout convertir en `DbErr::Custom`. On pourrait accepter n'importe quel `E: Display`.
- **Nom du test** : il vient du nom du thread. Si `runique_test` est appelé depuis un `tokio::spawn`, il devient `tokio-runtime-worker`.
- **cargo nextest** lance chaque test dans son propre processus, en parallèle : `SERIAL` ne sert plus à rien, et les tests se marchent dessus. À documenter.
- **`sqlite::memory:`** : chaque test démarre sur une base vide, sans tables, ce qui donne des échecs déroutants. Le détecter et le signaler.
- **Aucun moyen d'agir sur la trace depuis le handler** : c'est l'`assert_queries` à concevoir.

---

## Ordre proposé pour la réflexion

1. **T1, T2, T4** : là où le builder ment ou bloque tout. C'est le cœur de sa fiabilité.
2. **T10** : quelques lignes, gain de sécurité net.
3. **T5, T6, T7, T9** : fuites et mauvaises cibles.
4. **T8** : ta décision sur le garde-fou.
5. **T3, T11, T12, T13** : doc et ergonomie.
