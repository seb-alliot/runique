# Async/Await et Tokio

**Programmation asynchrone en Rust** — futurs, runtime, tâches, attente simultanée, canaux, limites de concurrence, annulation et arrêt propre, avec les différentes façons d'écrire la même opération.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Comprendre ce qu'est un futur et pourquoi il ne fait rien tant qu'on ne l'attend pas
- Démarrer un runtime tokio et choisir sa configuration
- Lancer des tâches avec `spawn`, et du code bloquant avec `spawn_blocking`
- Attendre plusieurs opérations à la fois : `join!`, `try_join!`, `select!`, `JoinSet`, streams
- Mettre une limite de temps et une limite de concurrence
- Communiquer entre tâches avec les canaux `mpsc`, `oneshot`, `broadcast`, `watch`
- Partager un état et choisir entre verrous std et tokio
- Gérer l'annulation et arrêter un serveur proprement
- Éviter les pièges : code bloquant, verrou gardé pendant un `.await`, tâches oubliées

## Table des matières

1. Synchrone, asynchrone, parallèle
2. Les futurs
3. Le runtime tokio
4. Lancer des tâches
5. Code bloquant
6. Attendre plusieurs futurs
7. Temps : sleep, interval, timeout
8. Canaux
9. État partagé
10. Limiter la concurrence
11. Streams
12. Annulation
13. Arrêt propre
14. La même opération écrite de plusieurs façons
15. Exemples dans Runique
16. Pièges courants
17. Exercices
18. Aide-mémoire

## 1. Synchrone, asynchrone, parallèle

### 1.1 Le problème

Un serveur web passe la plupart de son temps à **attendre** : la base de données, le réseau, le disque. En code synchrone, un thread qui attend ne fait rien d'autre.

```rust
// Synchrone : chaque appel bloque le thread jusqu'à la réponse
fn traiter() {
    let u = lire_utilisateur();     // 20 ms d'attente
    let c = lire_commandes();       // 30 ms d'attente
    // total : 50 ms pendant lesquels le thread est inutilisable
}
```

### 1.2 Trois notions à ne pas confondre

| Notion | Définition | Outil |
|---|---|---|
| **Concurrence** | plusieurs tâches avancent en se relayant | `async`/`await`, tâches tokio |
| **Parallélisme** | plusieurs calculs s'exécutent au même instant sur plusieurs cœurs | threads, `rayon` |
| **Asynchrone** | une tâche qui attend rend la main au lieu de bloquer | `async`/`await` |

L'asynchrone sert à **attendre beaucoup de choses à la fois** avec peu de threads : des milliers de connexions sur quelques threads. Il ne rend pas un calcul plus rapide. Pour un calcul lourd (hachage, traitement d'image), ce sont les threads qui comptent (section 5).

### 1.3 Thread ou tâche async

| | Thread système | Tâche async |
|---|---|---|
| Coût de création | quelques dizaines de µs, une pile de plusieurs Mo | quelques centaines d'octets |
| Nombre raisonnable | quelques centaines | des centaines de milliers |
| Changement de tâche | décidé par le système, n'importe quand | seulement aux `.await` |
| Bon pour | calcul, code bloquant | entrées/sorties : réseau, base, fichiers |

## 2. Les futurs

### 2.1 `async fn` renvoie un futur

```rust
async fn saluer() -> String {
    "bonjour".to_string()
}

// est équivalent à :
fn saluer2() -> impl std::future::Future<Output = String> {
    async { "bonjour".to_string() }
}
```

Appeler `saluer()` **n'exécute rien** : cela crée un futur, une valeur qui représente un calcul pas encore fait.

### 2.2 Un futur ne fait rien sans `.await`

```rust
async fn ecrire_journal() {
    println!("écrit");
}

async fn exemple() {
    ecrire_journal();          // RIEN ne se passe. Avertissement : unused implementer of `Future`
    ecrire_journal().await;    // maintenant, la fonction s'exécute
}
```

Les futurs Rust sont **paresseux** : contrairement aux promesses JavaScript, ils ne démarrent que lorsqu'on les attend (ou qu'on les confie à `spawn`).

### 2.3 `.await` : attendre sans bloquer

```rust
async fn traiter() {
    let u = lire_utilisateur().await;   // la tâche est suspendue ; le thread sert à d'autres tâches
    let c = lire_commandes().await;
}
```

Ce code est toujours **séquentiel** : `lire_commandes` ne commence qu'après `lire_utilisateur`. La différence avec le synchrone : pendant l'attente, le thread traite d'autres requêtes. Pour lancer les deux en même temps : section 6.

### 2.4 Blocs `async` et closures `async`

```rust
let futur = async {
    let a = 1;
    a + 1
};

let x = 5;
let avec_capture = async move { x * 2 };   // move : le bloc possède x

let f = async |n: u32| n + 1;              // closure async (Rust 1.85+)
```

### 2.5 Ce qu'est un futur, en bref

Un futur est une machine à états générée par le compilateur. Chaque `.await` est un point où la fonction peut s'arrêter puis reprendre. Le runtime appelle sa méthode `poll` : le futur répond « terminé, voici la valeur » ou « pas encore, réveille-moi quand ce sera prêt ». Tout ce qui vit à travers un `.await` est stocké dans cette machine, ce qui explique les règles sur `Send` (cours `send-sync`).

## 3. Le runtime tokio

Rust ne fournit pas de runtime asynchrone : il faut une bibliothèque qui exécute les futurs. tokio est la plus utilisée (axum, sea-orm, Runique reposent dessus).

### 3.1 Installation

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

`full` active tout ; une bibliothèque ou un binaire optimisé choisit seulement ce dont il a besoin :

| Feature | Contenu |
|---|---|
| `rt` / `rt-multi-thread` | runtime à un thread / à plusieurs threads |
| `macros` | `#[tokio::main]`, `#[tokio::test]`, `join!`, `select!` |
| `time` | `sleep`, `interval`, `timeout` |
| `sync` | canaux, `Mutex`, `RwLock`, `Semaphore` |
| `net` | TCP, UDP |
| `fs` | fichiers |
| `signal` | Ctrl+C, signaux Unix |
| `test-util` | contrôle du temps dans les tests |

### 3.2 `#[tokio::main]`

```rust
#[tokio::main]
async fn main() {
    println!("dans le runtime");
}

// est équivalent à :
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            println!("dans le runtime");
        });
}
```

### 3.3 Variantes

```rust
#[tokio::main(flavor = "current_thread")]        // un seul thread : plus léger, pour un outil CLI
async fn main() {}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]   // nombre de threads choisi
async fn main2() {}
```

Par défaut, le runtime multithread crée un thread par cœur.

### 3.4 Appeler du code async depuis du code synchrone

```rust
fn outil() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let r = rt.block_on(async { 42 });
}
```

`block_on` à l'intérieur d'un runtime déjà en marche panique (« Cannot start a runtime from within a runtime ») : on ne l'utilise qu'à la frontière, dans un `main` ou un outil synchrone.

## 4. Lancer des tâches

### 4.1 `tokio::spawn`

```rust
#[tokio::main]
async fn main() {
    let tache = tokio::spawn(async {
        // s'exécute en parallèle de main, éventuellement sur un autre thread
        21 * 2
    });

    let resultat = tache.await.unwrap();      // JoinHandle : attendre la fin et récupérer la valeur
    println!("{resultat}");
}
```

- `spawn` démarre la tâche **immédiatement**, sans attendre de `.await`.
- Il renvoie un `JoinHandle<T>` ; `.await` dessus donne `Result<T, JoinError>` (`Err` si la tâche a paniqué ou a été annulée).
- La tâche doit être `Send + 'static` : elle peut changer de thread et survivre à la fonction qui la lance (cours `send-sync`).

### 4.2 Tâche détachée

```rust
tokio::spawn(async {
    envoyer_email().await;     // personne n'attend le résultat
});
```

Si le `JoinHandle` est ignoré, la tâche continue quand même. Mais une panique ou une erreur dans la tâche passe **inaperçue** : journalise les erreurs à l'intérieur.

### 4.3 Passer des données à une tâche

```rust
use std::sync::Arc;

let config = Arc::new(String::from("prod"));
for i in 0..3 {
    let config = Arc::clone(&config);      // une copie du pointeur par tâche
    tokio::spawn(async move {
        println!("tâche {i} : {config}");
    });
}
```

### 4.4 `JoinSet` : un groupe de tâches

```rust
use tokio::task::JoinSet;

let mut groupe = JoinSet::new();
for id in 1..=5 {
    groupe.spawn(async move { id * 10 });
}

while let Some(r) = groupe.join_next().await {   // dans l'ordre de FIN, pas de lancement
    println!("{}", r.unwrap());
}
```

Un `JoinSet` garde les tâches ensemble : `join_next` rend les résultats au fur et à mesure, et **toutes** les tâches restantes sont annulées si le `JoinSet` est libéré. C'est la façon sûre de lancer un nombre variable de tâches.

## 5. Code bloquant

### 5.1 Le problème

```rust
async fn mauvais() {
    std::thread::sleep(std::time::Duration::from_secs(1));   // bloque le THREAD entier
    let h = argon2_hacher("mot de passe");                    // calcul de 100 ms : bloque aussi
}
```

Un thread du runtime qui bloque ne peut plus faire avancer **aucune** autre tâche. Avec 4 threads et 4 requêtes qui hachent un mot de passe, tout le serveur est figé.

### 5.2 Les corrections

```rust
async fn bon() {
    // 1. Attendre : la version async
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 2. Calcul lourd ou bibliothèque bloquante : un thread dédié
    let hash = tokio::task::spawn_blocking(|| argon2_hacher("mot de passe"))
        .await
        .unwrap();
}
```

| Code | Correction |
|---|---|
| `std::thread::sleep` | `tokio::time::sleep(..).await` |
| `std::fs::read_to_string` sur un gros fichier | `tokio::fs::read_to_string(..).await` |
| calcul de plus de quelques centaines de µs | `spawn_blocking` |
| bibliothèque synchrone (client HTTP bloquant, SQLite synchrone) | `spawn_blocking`, ou la version async de la bibliothèque |
| `std::sync::Mutex` gardé pendant une attente | relâcher avant le `.await` (section 9) |

`spawn_blocking` exécute la closure sur un ensemble de threads séparé, prévu pour bloquer.

## 6. Attendre plusieurs futurs

### 6.1 `join!` : tous, en même temps

```rust
async fn utilisateur() -> String { "ada".into() }
async fn commandes() -> Vec<u32> { vec![1, 2] }

async fn page() {
    // Séquentiel : 20 ms + 30 ms = 50 ms
    let u = utilisateur().await;
    let c = commandes().await;

    // Simultané : max(20, 30) = 30 ms
    let (u, c) = tokio::join!(utilisateur(), commandes());
}
```

`join!` fait avancer les deux futurs **dans la même tâche**, en alternance : pas de nouveau thread, pas de `Send` exigé.

### 6.2 `try_join!` : tous, ou la première erreur

```rust
async fn a() -> Result<u32, String> { Ok(1) }
async fn b() -> Result<u32, String> { Err("échec".into()) }

async fn combiner() -> Result<u32, String> {
    let (x, y) = tokio::try_join!(a(), b())?;   // s'arrête dès la première erreur
    Ok(x + y)
}
```

### 6.3 `select!` : le premier qui termine

```rust
use std::time::Duration;

async fn rapide() -> &'static str { "rapide" }
async fn lent() -> &'static str { tokio::time::sleep(Duration::from_secs(5)).await; "lent" }

async fn course() {
    tokio::select! {
        r = rapide() => println!("{r}"),
        r = lent() => println!("{r}"),       // jamais affiché : ce futur est ANNULÉ
    }
}
```

Dès qu'une branche termine, les autres futurs sont **libérés** : leur travail en cours est abandonné (section 12).

### 6.4 `select!` dans une boucle

```rust
use tokio::sync::mpsc;

async fn boucle(mut rx: mpsc::Receiver<String>, mut arret: tokio::sync::oneshot::Receiver<()>) {
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => println!("message : {msg}"),
            _ = &mut arret => {
                println!("arrêt demandé");
                break;
            }
        }
    }
}
```

`&mut arret` : on emprunte le futur au lieu de le consommer, pour pouvoir le réutiliser à chaque tour.

### 6.5 Une liste de futurs : `join_all`

```rust
use futures::future::join_all;

async fn charger(id: u32) -> u32 { id }

async fn tout_charger() -> Vec<u32> {
    let futurs = (1..=10).map(charger);
    join_all(futurs).await                 // tous en même temps, résultats dans l'ordre
}
```

### 6.6 Choisir

| Besoin | Outil |
|---|---|
| quelques futurs connus, tous nécessaires | `join!` / `try_join!` |
| le premier qui finit, ou une attente avec arrêt possible | `select!` |
| liste de futurs dans la même tâche | `join_all`, `try_join_all` |
| beaucoup de travaux, sur plusieurs threads | `JoinSet` |
| beaucoup de travaux, avec une limite simultanée | `buffer_unordered` (section 11) ou `Semaphore` (section 10) |

## 7. Temps : sleep, interval, timeout

```rust
use std::time::Duration;
use tokio::time::{interval, sleep, timeout};

async fn temps() {
    sleep(Duration::from_millis(100)).await;

    let mut tic = interval(Duration::from_secs(60));
    for _ in 0..3 {
        tic.tick().await;           // le premier tick est immédiat
        println!("toutes les minutes");
    }

    match timeout(Duration::from_secs(2), operation_lente()).await {
        Ok(valeur) => println!("{valeur}"),
        Err(_) => println!("trop long : abandon"),
    }
}
```

`timeout` **annule** l'opération si le délai est dépassé. Toute requête réseau vers un service externe devrait avoir une limite de temps : sans elle, un service lent bloque les tâches qui l'attendent indéfiniment.

`interval` rattrape les ticks manqués par défaut ; `tic.set_missed_tick_behavior(MissedTickBehavior::Delay)` décale au lieu de rattraper.

## 8. Canaux

### 8.1 Les quatre canaux de tokio

| Canal | Émetteurs | Récepteurs | Valeurs | Usage |
|---|---|---|---|---|
| `mpsc` | plusieurs | un | file de messages | file de travaux |
| `oneshot` | un | un | une seule | renvoyer une réponse |
| `broadcast` | plusieurs | plusieurs, chacun reçoit tout | file | notifications à tous |
| `watch` | un | plusieurs, voient la dernière valeur | une valeur courante | configuration, état |

### 8.2 `mpsc` : une file de travaux

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(100);    // capacité 100 : au-delà, send attend

    for i in 0..3 {
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send(format!("travail {i}")).await.unwrap();
        });
    }
    drop(tx);                                            // sinon rx ne se terminerait jamais

    while let Some(msg) = rx.recv().await {              // None quand tous les émetteurs sont libérés
        println!("{msg}");
    }
}
```

La **capacité** limite la mémoire : quand la file est pleine, `send().await` attend que le consommateur rattrape. C'est la contre-pression (*backpressure*). `unbounded_channel` n'a pas de limite, et peut donc faire grossir la mémoire sans fin si le consommateur est plus lent.

### 8.3 `oneshot` : une réponse

```rust
use tokio::sync::{mpsc, oneshot};

enum Commande {
    Lire { cle: String, reponse: oneshot::Sender<Option<String>> },
}

async fn gestionnaire(mut rx: mpsc::Receiver<Commande>) {
    let mut donnees = std::collections::HashMap::new();
    donnees.insert("a".to_string(), "1".to_string());
    while let Some(Commande::Lire { cle, reponse }) = rx.recv().await {
        let _ = reponse.send(donnees.get(&cle).cloned());
    }
}

async fn client(tx: mpsc::Sender<Commande>) {
    let (rep_tx, rep_rx) = oneshot::channel();
    tx.send(Commande::Lire { cle: "a".into(), reponse: rep_tx }).await.unwrap();
    let valeur = rep_rx.await.unwrap();
}
```

C'est le pattern « acteur » : une seule tâche possède l'état, les autres lui envoient des commandes. Aucun verrou n'est nécessaire.

### 8.4 `broadcast` et `watch`

```rust
use tokio::sync::{broadcast, watch};

let (tx, mut rx1) = broadcast::channel::<String>(16);
let mut rx2 = tx.subscribe();
tx.send("à tous".into()).unwrap();          // rx1 et rx2 reçoivent chacun le message

let (cfg_tx, mut cfg_rx) = watch::channel("v1".to_string());
cfg_tx.send("v2".into()).unwrap();          // les lecteurs voient la dernière valeur
// cfg_rx.changed().await : attendre le prochain changement
let actuelle = cfg_rx.borrow().clone();
```

## 9. État partagé

### 9.1 `Arc<Mutex<T>>` avec un verrou std

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Etat { visites: Arc<Mutex<u64>> }

async fn handler(etat: Etat) {
    *etat.visites.lock().unwrap() += 1;      // verrou pris et relâché sans .await entre les deux
}
```

Un `std::sync::Mutex` est **le bon choix** tant que le verrou n'est jamais gardé pendant un `.await` : il est plus rapide qu'un verrou async.

### 9.2 Quand il faut un verrou async

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

async fn ecrire(fichier: Arc<Mutex<tokio::fs::File>>, ligne: &[u8]) {
    use tokio::io::AsyncWriteExt;
    let mut f = fichier.lock().await;        // attendre le verrou sans bloquer le thread
    f.write_all(ligne).await.unwrap();       // verrou gardé pendant l'écriture : nécessaire ici
}
```

| | `std::sync::Mutex` | `tokio::sync::Mutex` |
|---|---|---|
| Prendre le verrou | `lock()` bloque le thread (brièvement) | `lock().await` suspend la tâche |
| Garder pendant un `.await` | non (bloque, et le futur n'est plus `Send`) | oui |
| Vitesse | plus rapide | plus lent |
| Pour | sections courtes, sans attente | ressource utilisée pendant une attente (connexion, fichier) |

### 9.3 Lecteurs nombreux : `RwLock`

```rust
use std::sync::{Arc, RwLock};

let cache: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(vec![]));
let lecture = cache.read().unwrap().len();       // plusieurs lecteurs simultanés
cache.write().unwrap().push("x".into());         // un seul écrivain
```

### 9.4 Un nombre ou un drapeau : les atomiques

```rust
use std::sync::atomic::{AtomicU64, Ordering};
static REQUETES: AtomicU64 = AtomicU64::new(0);
REQUETES.fetch_add(1, Ordering::Relaxed);
```

Le détail des primitives de partage est dans le cours `concurrence`.

## 10. Limiter la concurrence

### 10.1 Le problème

```rust
// Lancer 10 000 requêtes d'un coup : le service distant refuse, la mémoire explose
// for url in urls { tokio::spawn(telecharger(url)); }
```

### 10.2 `Semaphore`

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

async fn telecharger(id: u32) -> u32 { id }

async fn tout(ids: Vec<u32>) -> Vec<u32> {
    let limite = Arc::new(Semaphore::new(10));    // au plus 10 en même temps
    let mut groupe = JoinSet::new();
    for id in ids {
        let permis = Arc::clone(&limite).acquire_owned().await.unwrap();   // attend une place libre
        groupe.spawn(async move {
            let r = telecharger(id).await;
            drop(permis);                          // libère la place
            r
        });
    }
    let mut resultats = Vec::new();
    while let Some(r) = groupe.join_next().await {
        resultats.push(r.unwrap());
    }
    resultats
}
```

Un `Semaphore` contient N permis ; chaque tâche en prend un et le rend en le libérant. La 11e attend qu'une place se libère.

## 11. Streams

Un **stream** est l'équivalent asynchrone d'un itérateur : une suite de valeurs qui arrivent dans le temps.

```rust
use futures::stream::{self, StreamExt};

async fn charger(id: u32) -> u32 { id * 2 }

async fn exemple() {
    // Comme un itérateur : map, filter, collect... mais avec .await
    let doubles: Vec<u32> = stream::iter(1..=5).then(charger).collect().await;

    // Au plus 4 chargements simultanés, résultats dans l'ordre d'arrivée
    let rapides: Vec<u32> = stream::iter(1..=100)
        .map(charger)
        .buffer_unordered(4)
        .collect()
        .await;

    // Parcourir un à un
    let mut s = stream::iter(vec!["a", "b"]);
    while let Some(x) = s.next().await {
        println!("{x}");
    }
}
```

| Méthode | Effet |
|---|---|
| `then(f)` | applique une fonction async, un élément à la fois |
| `map(f).buffer_unordered(n)` | jusqu'à n futurs en même temps, résultats dans l'ordre d'arrivée |
| `map(f).buffered(n)` | idem, résultats dans l'ordre d'origine |
| `next().await` | élément suivant |
| `collect().await` | tout rassembler |

Les streams viennent de la crate `futures` (`StreamExt`) ou de `tokio-stream`. Une réponse HTTP en morceaux, les lignes d'un fichier ou les messages d'un WebSocket sont des streams.

## 12. Annulation

### 12.1 Libérer un futur l'annule

En Rust, annuler un futur, c'est **arrêter de l'attendre** : il est libéré à son dernier point de suspension, et le code après ce `.await` ne s'exécute jamais.

```rust
async fn transfert() {
    debiter().await;
    // si le futur est annulé ici (timeout, select! perdu, client déconnecté)...
    crediter().await;          // ...cette ligne ne s'exécute jamais
}
```

Causes d'annulation : `timeout` dépassé, branche perdante d'un `select!`, `JoinHandle::abort()`, `JoinSet` libéré, et, dans un serveur web, **le client qui ferme la connexion** (axum libère le futur du handler).

### 12.2 Se protéger

```rust
// 1. Transaction : rien n'est validé tant que commit n'est pas atteint
async fn transfert_sur(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    use sea_orm::TransactionTrait;
    let txn = db.begin().await?;
    // debiter(&txn).await?; crediter(&txn).await?;
    txn.commit().await          // annulé avant : la transaction est annulée à la libération
}

// 2. Détacher le travail qui doit aller au bout
async fn handler() {
    tokio::spawn(async {
        // continue même si le client se déconnecte
    });
}
```

### 12.3 Sûreté à l'annulation dans `select!`

Une branche de `select!` qui perd est libérée. Si elle avait déjà **consommé** des données (lu une partie d'un message), elles sont perdues. La documentation de tokio indique pour chaque méthode si elle est *cancel safe* : `mpsc::Receiver::recv` l'est (le message reste dans le canal), `AsyncReadExt::read_exact` ne l'est pas.

## 13. Arrêt propre

```rust
use tokio::signal;

#[tokio::main]
async fn main() {
    let (arret_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    let mut arret_rx = arret_tx.subscribe();
    let travailleur = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                    println!("travail");
                }
                _ = arret_rx.recv() => {
                    println!("fin propre du travailleur");
                    break;
                }
            }
        }
    });

    signal::ctrl_c().await.unwrap();          // attendre Ctrl+C (ou SIGTERM avec signal::unix)
    let _ = arret_tx.send(());                // prévenir toutes les tâches
    travailleur.await.unwrap();               // attendre qu'elles aient fini
}
```

Avec axum : `axum::serve(listener, app).with_graceful_shutdown(signal)` cesse d'accepter des connexions et laisse finir les requêtes en cours. `tokio_util::sync::CancellationToken` est l'outil dédié pour diffuser un arrêt à de nombreuses tâches.

## 14. La même opération écrite de plusieurs façons

Charger 20 profils depuis un service distant.

```rust
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

async fn profil(id: u32) -> String { format!("profil {id}") }

// 1. Séquentiel : simple, lent (20 × la latence)
async fn v1() -> Vec<String> {
    let mut r = Vec::new();
    for id in 1..=20 {
        r.push(profil(id).await);
    }
    r
}

// 2. join_all : tout en même temps dans la même tâche, ordre conservé, aucune limite
async fn v2() -> Vec<String> {
    futures::future::join_all((1..=20).map(profil)).await
}

// 3. JoinSet : tâches séparées, sur plusieurs threads, ordre d'arrivée
async fn v3() -> Vec<String> {
    let mut g = JoinSet::new();
    for id in 1..=20 {
        g.spawn(profil(id));
    }
    let mut r = Vec::new();
    while let Some(p) = g.join_next().await {
        r.push(p.unwrap());
    }
    r
}

// 4. Stream avec limite : au plus 5 à la fois, ordre conservé
async fn v4() -> Vec<String> {
    stream::iter(1..=20).map(profil).buffered(5).collect().await
}

// 5. JoinSet + Semaphore : tâches séparées et limite
async fn v5() -> Vec<String> {
    let limite = Arc::new(Semaphore::new(5));
    let mut g = JoinSet::new();
    for id in 1..=20 {
        let l = Arc::clone(&limite);
        g.spawn(async move {
            let _p = l.acquire().await.unwrap();
            profil(id).await
        });
    }
    let mut r = Vec::new();
    while let Some(p) = g.join_next().await {
        r.push(p.unwrap());
    }
    r
}
```

| Version | Simultané | Limite | Ordre | Plusieurs threads |
|---|---|---|---|---|
| 1 séquentiel | non | — | oui | non |
| 2 `join_all` | tous | aucune | oui | non |
| 3 `JoinSet` | tous | aucune | arrivée | oui |
| 4 `buffered` | n | oui | oui | non |
| 5 `JoinSet` + `Semaphore` | n | oui | arrivée | oui |

Pour des appels réseau, la version 4 est souvent la meilleure : simple, limitée, ordonnée. Plusieurs threads (3, 5) ne servent que si chaque tâche fait aussi du calcul.

## 15. Exemples dans Runique

### 15.1 Une tâche de fond périodique

```rust
// runique/src/middleware/session/session_db.rs (extrait)
pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
    let db = self.db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        loop {
            interval.tick().await;
            let now = chrono::Utc::now().naive_utc();
            // supprime les sessions expirées ; l'erreur éventuelle est journalisée
            // Entity::delete_many().filter(Column::ExpiresAt.lt(now)).exec(&*db).await
        }
    });
}
```

`loop` + `interval` dans une tâche détachée : le nettoyage tourne pendant toute la vie du serveur. La connexion est clonée avant le `move` (section 4.3), et les erreurs sont journalisées dans la tâche, puisque personne n'attend son résultat (section 4.2).

### 15.2 Une lecture de fichier bloquante

```rust
// runique/src/forms/fields/media.rs (extrait)
let path = filename.to_string();
let read = tokio::task::spawn_blocking(move || read_image_dimensions(&path)).await;
```

Lire les dimensions d'une image envoyée est une opération de disque et de décodage synchrone : elle passe par `spawn_blocking` pour ne pas bloquer le thread qui traite les autres requêtes (section 5).

### 15.3 Une tâche détachée pour ne rien révéler

```rust
// runique/src/auth/password.rs (extrait)
// Everything that depends on the account runs in the background, its
// lookup and the token's write included: the page answers the same, in
// the same time, for every address.
tokio::spawn(async move {
    let Some(user) = BuiltinUserEntity::find_by_email(&engine.db, &email).await else {
        return;
    };
    // ... création du jeton, envoi de l'e-mail ...
});
```

Sur la page « mot de passe oublié », si la réponse attendait la recherche du compte et l'envoi de l'e-mail, sa **durée** révélerait si l'adresse existe (une adresse inconnue répondrait plus vite). En confiant tout ce travail à une tâche détachée, la page répond immédiatement et de la même façon pour toutes les adresses. C'est une protection contre les attaques par mesure du temps (*timing attacks*), obtenue avec un simple `spawn`.

## 16. Pièges courants

| Piège | Symptôme | Correction |
|---|---|---|
| futur jamais attendu | rien ne se passe, avertissement `unused` | `.await` ou `spawn` |
| `std::thread::sleep`, calcul lourd dans une tâche | tout le serveur ralentit | `tokio::time::sleep`, `spawn_blocking` |
| `std::sync::MutexGuard` gardé à travers un `.await` | « future is not `Send` », blocages | relâcher avant, ou `tokio::sync::Mutex` |
| émetteur `mpsc` jamais libéré | `while let Some(..) = rx.recv()` ne finit jamais | `drop(tx)` après les clones |
| canal sans limite | mémoire qui grandit | `mpsc::channel(n)` |
| appel réseau sans limite de temps | tâches suspendues indéfiniment | `timeout` |
| lancer des milliers de tâches d'un coup | service distant saturé | `Semaphore`, `buffer_unordered` |
| erreur dans une tâche détachée | perdue sans trace | journaliser dans la tâche |
| `block_on` dans un runtime | panique | `.await` directement |
| suite d'opérations interrompue par l'annulation | état incohérent | transaction, ou `spawn` |

## 17. Exercices

### Exercice 1 : paralléliser

`page()` appelle trois fonctions indépendantes l'une après l'autre. Réécris-la pour qu'elles s'exécutent en même temps, en renvoyant la première erreur.

```rust
async fn profil() -> Result<String, String> { Ok("p".into()) }
async fn amis() -> Result<Vec<String>, String> { Ok(vec![]) }
async fn notifications() -> Result<u32, String> { Ok(0) }

// Solution :
async fn page() -> Result<(String, Vec<String>, u32), String> {
    tokio::try_join!(profil(), amis(), notifications())
}
```

### Exercice 2 : limite de temps et nouvelles tentatives

Appelle `appel()` avec une limite de 2 secondes, jusqu'à 3 tentatives, en attendant 100 ms, 200 ms puis 400 ms entre elles.

```rust
use std::time::Duration;

async fn appel() -> Result<u32, String> { Ok(1) }

// Solution :
async fn avec_tentatives() -> Result<u32, String> {
    let mut attente = Duration::from_millis(100);
    for essai in 1..=3 {
        match tokio::time::timeout(Duration::from_secs(2), appel()).await {
            Ok(Ok(v)) => return Ok(v),
            Ok(Err(e)) if essai == 3 => return Err(e),
            Err(_) if essai == 3 => return Err("délai dépassé".into()),
            _ => {
                tokio::time::sleep(attente).await;
                attente *= 2;
            }
        }
    }
    unreachable!("la boucle renvoie toujours au troisième essai")
}
```

### Exercice 3 : limiteur

Écris une fonction qui traite une liste d'identifiants avec au plus 3 traitements simultanés, en renvoyant les résultats dans l'ordre d'origine.

```rust
use futures::stream::{self, StreamExt};

async fn traiter(id: u32) -> u32 { id }

// Solution :
async fn tout(ids: Vec<u32>) -> Vec<u32> {
    stream::iter(ids).map(traiter).buffered(3).collect().await
}
```

## 18. Aide-mémoire

```rust
#[tokio::main] async fn main() {}                       #[tokio::test] async fn t() {}
let h = tokio::spawn(async move { .. }); h.await?;      tokio::task::spawn_blocking(|| ..).await?
tokio::join!(a(), b())     tokio::try_join!(a(), b())?   tokio::select! { x = a() => .., _ = b() => .. }
let mut g = JoinSet::new(); g.spawn(f); g.join_next().await
tokio::time::sleep(d).await     interval(d).tick().await  timeout(d, f).await
mpsc::channel(n)   oneshot::channel()   broadcast::channel(n)   watch::channel(v)
Semaphore::new(n).acquire().await     stream::iter(v).map(f).buffered(n).collect().await
signal::ctrl_c().await   axum::serve(..).with_graceful_shutdown(..)
```

### Points clés

- Un futur ne fait rien tant qu'il n'est pas attendu ou lancé avec `spawn`.
- `.await` suspend la tâche sans bloquer le thread ; deux `.await` successifs restent séquentiels.
- `join!` pour attendre ensemble, `select!` pour le premier, `JoinSet` pour un groupe de tâches.
- Jamais de code bloquant dans une tâche : `spawn_blocking`.
- `std::sync::Mutex` tant qu'il n'est pas gardé à travers un `.await`.
- Toute attente externe a une limite de temps, toute rafale a une limite de concurrence.
- Annuler un futur arrête son exécution au `.await` en cours : protège les suites d'opérations par une transaction.
- Une tâche détachée peut servir à rendre une réponse indépendante d'un secret (temps constant).
