## Async/Await et Tokio

## Programmation Asynchrone en Rust 

Futures, Async Runtime et Concurrence 

## Objectifs du cours

À la fin de ce cours, tu sauras : 

- Comprendre async/await en Rust 

- Utiliser Tokio comme runtime 

- Gérer la concurrence avec async 

- Créer des applications web async 

- Maîtriser les patterns async 

## Table des matières

1. Concepts de base 

- 1.1 - Sync vs Async 

- 1.2 - Futures en Rust 

- 1.3 - async/await 

2. Tokio Runtime 

- 2.1 - Installation et setup 

2.2 - #[tokio::main] 

- 2.3 - Spawning tasks 

3. Opérations async courantes 

- 3.1 - tokio::time 

3.2 - tokio::fs 

3.3 - tokio::net 

4. Concurrence 

- 4.1 - join! et select! 

- 4.2 - Channels (mpsc) 

- 4.3 - Mutex et RwLock 

5. Patterns avancés 

- 5.1 - Stream trait 

- 5.2 - Backpressure 

6. Web avec Axum 

- 6.1 - Serveur HTTP basique 

- 6.2 - Routes et handlers 

7. Best practices 

8. Exercices 

## 1. Concepts de base

## 1.1 - Sync vs Async

```
// Code SYNCHRONE (bloquant)
fn fetch_data() -> String {
    std::thread::sleep(Duration::from_secs(2));
    "data".to_string()
}
fn main() {
    let data1 = fetch_data();  // Attend 2s
    let data2 = fetch_data();  // Attend encore 2s
    // Total : 4 secondes
}
// Code ASYNCHRONE (non-bloquant)
async fn fetch_data() -> String {
    tokio::time::sleep(Duration::from_secs(2)).await;
    "data".to_string()
}
#[tokio::main]
async fn main() {
    let future1 = fetch_data();
    let future2 = fetch_data();
    let (data1, data2) = tokio::join!(future1, future2);
    // Total : 2 secondes (parallèle !)
}
```

### Async** ≠ **Multi-threading :

Async permet de faire plusieurs choses **sans bloquer** , mais ne crée pas forcément plusieurs threads. 

**1.2 - Futures en Rust** 

```
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
// Une Future retourne Poll<Output>
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context)
        -> Poll<Self::Output>;
}
// Poll peut être :
enum Poll<T> {
    Ready(T),      // La valeur est prête
    Pending,       // Pas encore prêt, revenir plus tard
}
// Les futures sont LAZY (ne font rien tant qu'on ne les .await pas)
async fn operation() -> i32 {
    42
}
let future = operation();  // Ne fait RIEN
let result = future.await; // MAINTENANT ça s'exécute
```

## 1.3 - async/await

```
// Fonction async
async fn dire_bonjour() {
    println!("Bonjour !");
}
// Retourne une Future
async fn calculer() -> i32 {
    42
}
// Utiliser .await pour attendre
async fn utiliser() {
    dire_bonjour().await;
    let resultat = calculer().await;
    println!("{}", resultat);
}
// async fn est du sucre syntaxique pour :
fn calculer() -> impl Future<Output = i32> {
    async { 42 }
}
// Chaîner des opérations async
async fn traiter() -> Result<String, Error> {
    let data = fetch_data().await?;
    let processed = process(data).await?;
    let saved = save(processed).await?;
    Ok(saved)
}
```

## 2. Tokio Runtime

## 2.1 - Installation et setup

```
# Dans Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```
# Features spécifiques (plus léger)
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net"] }
```

```
# Features utiles :
# - rt-multi-thread : Runtime multi-thread
# - rt : Runtime single-thread
# - macros : #[tokio::main] et autres
# - net : TCP/UDP
# - fs : File system async
# - time : Sleep et timers
# - sync : Primitives de sync (Mutex, etc.)
```

## 2.2 - #[tokio::main]

```
// Avec la macro (simple)
#[tokio::main]
async fn main() {
    println!("Hello async world!");
}
// Équivalent à :
fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            println!("Hello async world!");
        })
}
// Configuration custom du runtime
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // ...
}
```

```
// Single-thread runtime
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Utile pour les tests ou petites apps
}
```

## 2.3 - Spawning tasks

```
use tokio::task;
#[tokio::main]
async fn main() {
    // Spawner une task (comme un thread léger)
    let handle = tokio::spawn(async {
        // Cette tâche s'exécute en parallèle
        println!("Dans une task !");
        42
    });
    // Attendre le résultat
    let result = handle.await.unwrap();
    println!("Résultat : {}", result);
    // Spawner plusieurs tasks
    let mut handles = vec![];
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Task {} terminée", i);
            i
        });
        handles.push(handle);
    }
    // Attendre toutes les tasks
    for handle in handles {
        handle.await.unwrap();
    }
}
```

II **tokio::spawn requiert 'static :** Les tasks doivent posséder leurs données ou utiliser Arc pour partager. 

## 3. Opérations async courantes

## 3.1 - tokio::time

```
use tokio::time::{sleep, interval, timeout, Duration};
// Sleep (non-bloquant)
tokio::time::sleep(Duration::from_secs(1)).await;
// Interval (répétitif)
let mut interval = interval(Duration::from_secs(1));
for _ in 0..5 {
    interval.tick().await;
    println!("Tick !");
}
// Timeout
let result = timeout(
    Duration::from_secs(5),
    longue_operation()
).await;
match result {
    Ok(value) => println!("Terminé : {:?}", value),
    Err(_) => println!("Timeout !"),
}
```

```
// Deadline
use tokio::time::Instant;
let deadline = Instant::now() + Duration::from_secs(10);
tokio::time::sleep_until(deadline).await;
```

## 3.2 - tokio::fs

```
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
```

```
// Lire un fichier
let contenu = fs::read_to_string("fichier.txt").await?;
```

```
// Écrire un fichier
fs::write("sortie.txt", "Hello async!").await?;
```

```
// Lire avec buffer
let mut file = fs::File::open("data.txt").await?;
let mut buffer = Vec::new();
file.read_to_end(&mut buffer).await?;
```

```
// Écrire avec buffer
let mut file = fs::File::create("output.txt").await?;
file.write_all(b"Hello").await?;
file.flush().await?;
```

```
// Manipulations de fichiers
fs::rename("old.txt", "new.txt").await?;
fs::remove_file("temp.txt").await?;
fs::create_dir_all("path/to/dir").await?;
```

## 3.3 - tokio::net

```
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Serveur TCP simple
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Serveur démarré sur port 8080");
    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Connexion de : {}", addr);
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    return;
                }
                socket.write_all(&buffer[0..n]).await.unwrap();
            }
        });
    }
}
// Client TCP
async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    stream.write_all(b"Hello server!").await?;
```

```
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    println!("Reçu : {}", String::from_utf8_lossy(&buffer[..n]));
    Ok(())
}
```

## 4. Concurrence

## 4.1 - join! et select!

```
use tokio::{join, select};
```

```
// join! attend TOUTES les futures
async fn exemple_join() {
    let (res1, res2, res3) = join!(
        fetch_data1(),
        fetch_data2(),
        fetch_data3()
    );
    // Les 3 s'exécutent en parallèle
}
```

```
// select! prend la PREMIÈRE qui termine
async fn exemple_select() {
    select! {
        result = fetch_data() => {
            println!("Data: {:?}", result);
        }
```

```
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            println!("Timeout !");
        }
    }
}
// try_join! pour Result
use tokio::try_join;
```

```
async fn exemple_try_join() -> Result<(), Error> {
    let (res1, res2) = try_join!(
        async_operation1(),
        async_operation2()
    )?;
```

```
    Ok(())
}
```

## 4.2 - Channels (mpsc)

```
use tokio::sync::mpsc;
```

```
#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);  // Buffer de 32
    // Spawner le producteur
    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    // Consommateur
    while let Some(value) = rx.recv().await {
        println!("Reçu : {}", value);
    }
}
// unbounded channel (sans limite)
use tokio::sync::mpsc::unbounded_channel;
```

```
let (tx, mut rx) = unbounded_channel();
```

```
// oneshot (un seul message)
use tokio::sync::oneshot;
let (tx, rx) = oneshot::channel();
```

```
tokio::spawn(async move {
    tx.send(42).unwrap();
});
let result = rx.await.unwrap();
```

## 4.3 - Mutex et RwLock

```
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
// Mutex pour async
#[tokio::main]
async fn main() {
    let data = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let data = Arc::clone(&data);
        let handle = tokio::spawn(async move {
            let mut num = data.lock().await;
            *num += 1;
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }
    println!("Résultat : {}", *data.lock().await);
}
// RwLock (multiple readers, single writer)
let lock = Arc::new(RwLock::new(String::from("hello")));
// Lecture (multiple simultanés)
let read = lock.read().await;
println!("{}", *read);
// Écriture (exclusif)
let mut write = lock.write().await;
write.push_str(" world");
```

## 5. Patterns avancés

## 5.1 - Stream trait

```
use tokio_stream::{Stream, StreamExt};
```

```
// Stream = Iterator async
async fn exemple_stream() {
    let mut stream = tokio_stream::iter(vec![1, 2, 3, 4, 5]);
    while let Some(value) = stream.next().await {
        println!("{}", value);
    }
}
// Créer un stream custom
use async_stream::stream;
fn number_stream() -> impl Stream<Item = i32> {
    stream! {
        for i in 0..10 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            yield i;
        }
    }
}
// Transformer des streams
async fn traiter_stream() {
    let stream = number_stream()
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2);
    tokio::pin!(stream);
    while let Some(value) = stream.next().await {
        println!("{}", value);
    }
}
```

## 5.2 - Backpressure

```
use tokio::sync::Semaphore;
use std::sync::Arc;
// Limiter le nombre de requêtes simultanées
#[tokio::main]
async fn main() {
    let semaphore = Arc::new(Semaphore::new(5));  // Max 5 simultanés
    let mut handles = vec![];
    for i in 0..100 {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let handle = tokio::spawn(async move {
            // Faire le travail
            process(i).await;
            drop(permit);  // Libère le slot
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }
}
// Rate limiting
use tokio::time::{interval, Duration};
```

```
let mut interval = interval(Duration::from_millis(100));
```

```
for request in requests {
    interval.tick().await;  // Attend avant chaque requête
    process(request).await;
}
```

## 6. Web avec Axum

## 6.1 - Serveur HTTP basique

```
# Dans Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
# Code
use axum::{
    routing::get,
    Router,
};
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("Serveur sur http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
async fn handler() -> &'static str {
    "Hello, World!"
}
```

## 6.2 - Routes et handlers

```
use axum::{
    extract::{Path, Query, Json},
    response::IntoResponse,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1,
        username: payload.username,
    };
    (StatusCode::CREATED, Json(user))
}
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(User {
        id,
        username: "Alice".to_string(),
    })
}
```

```
#[tokio::main]
```

```
async fn main() {
    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user));
```

```
    // ...
}
```

## 7. Best practices

• **1. Ne pas bloquer le runtime** Évite les opérations CPU-intensives dans les tasks async. Utilise `tokio::task::spawn_blocking` . 

• **2. Utiliser des channels pour communiquer** Préfère les channels aux Mutex quand c'est possible. 

## • **3. Limiter la concurrence** 

Utilise Semaphore pour éviter de surcharger le système. 

## • **4. Gérer les timeouts** 

Toujours ajouter des timeouts aux opérations réseau. 

## • **5. Cancel safety** 

Assure-toi que tes futures peuvent être annulées proprement. 

## • **6. Arc pour le partage** 

Utilise `Arc` pour partager des données entre tasks. 

## • **7. Profiler avec tokio-console** 

Utilise tokio-console pour déboguer les performances. 

## 8. Exercices pratiques

### Exercice 1 : Paralléliser des requêtes

```
// Écris une fonction qui fetch plusieurs URLs en parallèle
```

```
use reqwest;
```

```
async fn fetch_all(urls: Vec<String>) -> Vec<Result<String, reqwest::Error>> {
    let mut handles = vec![];
```

```
    for url in urls {
        let handle = tokio::spawn(async move {
            reqwest::get(&url)
                .await?
                .text()
                .await
        });
        handles.push(handle);
    }
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```

### Exercice 2 : Rate limiter

```
// Crée un rate limiter qui limite à N requêtes/seconde
use tokio::time::{interval, Duration};
```

```
struct RateLimiter {
    interval: tokio::time::Interval,
}
impl RateLimiter {
    fn new(requests_per_second: u64) -> Self {
        let duration = Duration::from_secs(1) / requests_per_second as u32;
        Self {
            interval: interval(duration),
        }
    }
    async fn acquire(&mut self) {
        self.interval.tick().await;
    }
}
// Utilisation
#[tokio::main]
async fn main() {
    let mut limiter = RateLimiter::new(10);  // 10 req/s
    for i in 0..100 {
        limiter.acquire().await;
        println!("Requête {}", i);
    }
}
```

## Aide-mémoire

|**Concept**|**Usage**|
|---|---|
|**`async fn`**|Fonction asynchrone|
|**`.await`**|Attendre une future|
|**`tokio::spawn`**|Créer une task|
|**`tokio::join!`**|Attendre plusieurs futures|
|**`tokio::select!`**|Première future terminée|
|**`mpsc::channel`**|Communication entre tasks|
|**`Arc<Mutex<T>>`**|Partage mutable|

- **async/await** = programmation non-bloquante 

- **Tokio** = runtime pour exécuter les futures 

- **spawn** = créer des tâches concurrentes 

- **join!** = attendre plusieurs futures 

- **channels** = communication entre tasks 

- **Semaphore** = limiter la concurrence 

### INCROYABLE !

Tu as terminé TOUS les cours Rust ! Tu maîtrises maintenant : I Variables, fonctions, ownership I Structures, enums, pattern matching I Collections et itérateurs I Gestion d'erreurs avancée I Lifetimes I Modules et organisation I Traits avancés I Async/await et Tokio I **TU ES UN EXPERT RUST !** I
