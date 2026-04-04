# Concurrence et État Partagé en Rust
> `Mutex`, `RwLock`, `Arc`, `LazyLock`, `OnceLock` — partager des données entre threads en toute sécurité

## Objectifs

- Comprendre pourquoi la concurrence est difficile et comment Rust y répond
- Utiliser `Mutex<T>` pour l'exclusion mutuelle
- Utiliser `RwLock<T>` pour la lecture/écriture concurrente
- Partager des données avec `Arc<T>`
- Connaître le pattern `Arc<Mutex<T>>`
- Initialiser des ressources globales avec `LazyLock` et `OnceLock`
- Choisir la bonne primitive selon le cas d'usage

---

## Table des matières

1. [Pourquoi la concurrence est difficile](#1-pourquoi-la-concurrence-est-difficile)
2. [Mutex<T> — exclusion mutuelle](#2-mutext--exclusion-mutuelle)
3. [RwLock<T> — lecture/écriture](#3-rwlockt--lectureécriture)
4. [Arc<T> — référence comptée thread-safe](#4-arct--référence-comptée-thread-safe)
5. [Arc<Mutex<T>> — pattern classique](#5-arcmutext--pattern-classique)
6. [LazyLock<T> — initialisation paresseuse](#6-lazylockt--initialisation-paresseuse)
7. [OnceLock<T> — valeur initialisée une seule fois](#7-oncelockt--valeur-initialisée-une-seule-fois)
8. [Comparaison et quand utiliser quoi](#8-comparaison-et-quand-utiliser-quoi)
9. [Exemples concrets avec Runique](#9-exemples-concrets-avec-runique)
10. [Piège — stores partagés sans cleanup](#10-piège--stores-partagés-sans-cleanup)
11. [Exercices pratiques](#11-exercices-pratiques)
12. [Aide-mémoire](#12-aide-mémoire)

---

## 1. Pourquoi la concurrence est difficile

En programmation concurrente, plusieurs threads accèdent aux mêmes données simultanément.
Cela génère deux catégories de bugs :

**Data race** — deux threads modifient la même mémoire sans synchronisation. Le résultat est
imprévisible et peut être différent à chaque exécution.

**Deadlock** — deux threads attendent chacun un verrou que l'autre détient. Ils se bloquent
mutuellement pour toujours.

```rust
// Ce code ne compile PAS — Rust empêche le data race à la compilation
use std::thread;

let mut compteur = 0;

thread::spawn(|| compteur += 1); // erreur : compteur emprunté depuis un autre thread
thread::spawn(|| compteur += 1); // erreur : idem
```

Rust résout ces problèmes grâce aux traits `Send` et `Sync` vérifiés à la compilation,
et aux primitives de synchronisation de la bibliothèque standard.

---

## 2. `Mutex<T>` — exclusion mutuelle

`Mutex<T>` (*Mutual Exclusion*) garantit qu'un seul thread à la fois peut accéder aux données.
Pour lire ou modifier la valeur, il faut d'abord acquérir le **verrou**.

```rust
use std::sync::Mutex;

let m = Mutex::new(5);

{
    // lock() bloque jusqu'à ce que le verrou soit disponible
    let mut val = m.lock().unwrap();
    *val += 1;
    println!("{val}"); // 6
} // le verrou est libéré ici automatiquement (drop du MutexGuard)

// On peut de nouveau accéder
println!("{:?}", m.lock().unwrap()); // 6
```

### Gestion des erreurs avec `lock()`

`lock()` retourne `Err` si un thread a paniqué en tenant le verrou (*poisoned mutex*).

```rust
use std::sync::Mutex;

let mutex = Mutex::new(vec![1, 2, 3]);

match mutex.lock() {
    Ok(mut guard) => {
        guard.push(4);
        println!("{:?}", *guard);
    }

    Err(poisoned) => {
        // Récupérer quand même les données
        let mut guard = poisoned.into_inner();
        guard.push(99);
        println!("Récupéré : {:?}", *guard);
    }
}
```

### `try_lock()` — tentative non bloquante

```rust
use std::sync::Mutex;

let mutex = Mutex::new(0);

match mutex.try_lock() {
    Ok(mut val) => *val += 1,
    Err(_) => println!("Verrou occupé, on continue"),
}
```

---

## 3. `RwLock<T>` — lecture/écriture

`RwLock<T>` (*Read-Write Lock*) permet **plusieurs lecteurs simultanés** ou **un seul écrivain**.
C'est plus efficace que `Mutex` quand les lectures sont fréquentes et les écritures rares.

```rust
use std::sync::RwLock;

let verrou = RwLock::new(vec![1, 2, 3]);

// Plusieurs lectures simultanées — OK
let lecture1 = verrou.read().unwrap();
let lecture2 = verrou.read().unwrap();
println!("{:?} {:?}", *lecture1, *lecture2);
drop(lecture1);
drop(lecture2);

// Écriture exclusive — bloque si des lecteurs sont actifs
{
    let mut ecriture = verrou.write().unwrap();
    ecriture.push(4);
} // verrou d'écriture libéré

println!("{:?}", verrou.read().unwrap()); // [1, 2, 3, 4]
```

### Différence avec `Mutex`

```rust
// Mutex : un seul accès à la fois, même pour la lecture
// RwLock : plusieurs lecteurs simultanés, un seul écrivain

// Choisir selon le ratio lecture/écriture :
// - Beaucoup de lectures, peu d'écritures → RwLock
// - Équilibré ou données petites → Mutex (moins de surcharge)
```

---

## 4. `Arc<T>` — référence comptée thread-safe

`Arc<T>` (*Atomically Reference Counted*) permet à **plusieurs threads de posséder** la même
valeur. Chaque clone incrémente un compteur atomique ; la valeur est libérée quand le compteur
atteint zéro.

```rust
use std::sync::Arc;
use std::thread;

let donnees = Arc::new(vec![1, 2, 3, 4, 5]);
let mut handles = vec![];

for i in 0..3 {
    let clone = Arc::clone(&donnees);

    let handle = thread::spawn(move || {
        println!("Thread {i} : {:?}", clone);
    });

    handles.push(handle);
}

for h in handles {
    h.join().unwrap();
}

// donnees est toujours accessible ici
println!("Total : {}", donnees.len());
```

> `Arc<T>` seul ne permet que la lecture. Pour modifier les données partagées entre threads,
> combinez avec `Mutex<T>` ou `RwLock<T>`.

```rust
// Rc<T> vs Arc<T>
use std::rc::Rc;
use std::sync::Arc;

let rc  = Rc::new(42);   // thread unique — compteur ordinaire, plus rapide
let arc = Arc::new(42);  // multi-thread — compteur atomique, légèrement plus lent

// Rc ne peut PAS être envoyé entre threads (erreur de compilation)
// Arc peut traverser les frontières de threads
```

---

## 5. `Arc<Mutex<T>>` — pattern classique

C'est la combinaison standard pour **partager et modifier** des données entre plusieurs threads.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let compteur = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let clone = Arc::clone(&compteur);

    let handle = thread::spawn(move || {
        let mut val = clone.lock().unwrap();
        *val += 1;
    });

    handles.push(handle);
}

for h in handles {
    h.join().unwrap();
}

println!("Résultat : {}", compteur.lock().unwrap()); // 10
```

### Pattern avec état applicatif

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Cache {
    donnees: Arc<Mutex<HashMap<String, String>>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            donnees: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn inserer(&self, cle: &str, valeur: &str) {
        let mut map = self.donnees.lock().unwrap();
        map.insert(cle.to_string(), valeur.to_string());
    }

    fn obtenir(&self, cle: &str) -> Option<String> {
        let map = self.donnees.lock().unwrap();
        map.get(cle).cloned()
    }
}

let cache = Cache::new();
let cache2 = cache.clone(); // partage le même Arc intérieur

cache.inserer("cle1", "valeur1");
println!("{:?}", cache2.obtenir("cle1")); // Some("valeur1")
```

---

## 6. `LazyLock<T>` — initialisation paresseuse

`LazyLock<T>` (stable depuis Rust 1.80) initialise une valeur **la première fois qu'on y accède**,
de façon thread-safe. Idéal pour les ressources globales coûteuses à initialiser.

```rust
use std::sync::LazyLock;
use std::collections::HashMap;

// Initialisé au premier accès, jamais avant
static CODES_PAYS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("FR", "France");
    m.insert("DE", "Allemagne");
    m.insert("JP", "Japon");
    m
});

fn main() {
    // La HashMap est créée ici, au premier accès
    println!("{:?}", CODES_PAYS.get("FR")); // Some("France")
    println!("{:?}", CODES_PAYS.get("JP")); // Some("Japon")
}
```

### Comparaison avec `once_cell` (avant Rust 1.80)

```rust
// Avant Rust 1.80, on utilisait la crate once_cell
// once_cell::sync::Lazy est identique à std::sync::LazyLock

// Depuis Rust 1.80, LazyLock est dans la stdlib — pas de dépendance externe nécessaire
use std::sync::LazyLock;

static CONFIG: LazyLock<String> = LazyLock::new(|| {
    std::env::var("APP_CONFIG").unwrap_or_else(|_| "defaut".to_string())
});
```

### `LazyLock` avec un type complexe

```rust
use std::sync::LazyLock;

struct Connexion {
    url: String,
}

impl Connexion {
    fn nouvelle(url: &str) -> Self {
        println!("Connexion établie vers {url}");
        Connexion { url: url.to_string() }
    }

    fn ping(&self) -> bool {
        println!("Ping vers {}", self.url);
        true
    }
}

static DB: LazyLock<Connexion> = LazyLock::new(|| {
    Connexion::nouvelle("postgres://localhost/mabase")
});

// La connexion n'est créée que lors du premier appel à DB
fn main() {
    println!("Démarrage...");
    DB.ping(); // connexion créée ici
    DB.ping(); // déjà initialisé, réutilisé directement
}
```

---

## 7. `OnceLock<T>` — valeur initialisée une seule fois

`OnceLock<T>` est similaire à `LazyLock` mais l'initialisation est **manuelle** — vous choisissez
quand et comment initialiser la valeur.

```rust
use std::sync::OnceLock;

static INSTANCE: OnceLock<String> = OnceLock::new();

fn obtenir_instance() -> &'static String {
    INSTANCE.get_or_init(|| {
        println!("Initialisation unique...");
        "valeur globale".to_string()
    })
}

fn main() {
    println!("{}", obtenir_instance()); // initialise
    println!("{}", obtenir_instance()); // réutilise, pas de réinitialisation
}
```

### Initialisation depuis une fonction externe

```rust
use std::sync::OnceLock;

static PORT: OnceLock<u16> = OnceLock::new();

fn configurer(port: u16) -> Result<(), u16> {
    PORT.set(port) // retourne Err(port) si déjà initialisé
}

fn port() -> u16 {
    *PORT.get().expect("port non configuré")
}

fn main() {
    configurer(8080).unwrap();

    match configurer(9090) {
        Ok(_)  => println!("configuré"),
        Err(p) => println!("déjà initialisé avec {p}"),
    }

    println!("Port actif : {}", port()); // 8080
}
```

### Différence `LazyLock` vs `OnceLock`

```rust
// LazyLock — initialisation automatique à la closure définie à la déclaration
static A: LazyLock<String> = LazyLock::new(|| "automatique".to_string());

// OnceLock — initialisation manuelle, peut être faite depuis n'importe où
static B: OnceLock<String> = OnceLock::new();

fn main() {
    let _ = &*A;              // A s'initialise ici
    B.set("manuel".to_string()).unwrap(); // B initialisé explicitement
}
```

---

## 8. Comparaison et quand utiliser quoi

| Type | Thread-safe | Propriétaires | Mutation | Cas d'usage |
|---|---|---|---|---|
| `Mutex<T>` | ✅ | 1 | oui (lock exclusif) | Compteur, état partagé |
| `RwLock<T>` | ✅ | 1 | oui (1 écrivain ou N lecteurs) | Cache lu souvent, écrit rarement |
| `Arc<T>` | ✅ | N | non (seul) | Partage en lecture seule |
| `Arc<Mutex<T>>` | ✅ | N | oui (lock) | État partagé entre threads |
| `Arc<RwLock<T>>` | ✅ | N | oui (lock) | Config partagée, lectures fréquentes |
| `LazyLock<T>` | ✅ | — | non (init une fois) | Ressource globale paresseuse |
| `OnceLock<T>` | ✅ | — | non (init une fois) | Valeur globale initialisée manuellement |

**Règles pratiques :**

- Vous partagez entre threads sans modifier → `Arc<T>`
- Vous partagez et modifiez → `Arc<Mutex<T>>`
- Lectures très fréquentes, écritures rares → `Arc<RwLock<T>>`
- Ressource globale à initialiser une seule fois → `LazyLock<T>` ou `OnceLock<T>`
- Thread unique avec mutation partagée → `Rc<RefCell<T>>`

---

## 9. Exemples concrets avec Runique

Dans Runique, plusieurs primitives de concurrence sont utilisées pour gérer l'état global
du framework (environnement, token CSS, configuration de session, nettoyage de tâches).

### `LazyLock` pour l'environnement global

```rust
use std::sync::LazyLock;

// Lecture du .env une seule fois au démarrage
static ENV: LazyLock<RuniqueEnv> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    match std::env::var("DEBUG").as_deref() {
        Ok("true") => RuniqueEnv::Development,
        _          => RuniqueEnv::Production,
    }
});

pub fn is_debug() -> bool {
    matches!(*ENV, RuniqueEnv::Development)
}

pub fn css_token() -> String {
    static TOKEN: LazyLock<String> = LazyLock::new(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_millis();
        format!("{:04}", ts % 10_000)
    });
    TOKEN.clone()
}
```

### `Arc<Mutex<T>>` pour le nettoyage de sessions

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone)]
struct SessionStore {
    donnees: Arc<Mutex<HashMap<String, (Vec<u8>, Instant)>>>,
}

impl SessionStore {
    fn new() -> Self {
        SessionStore {
            donnees: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn nettoyer_expirees(&self, duree_max: std::time::Duration) {
        let mut map = self.donnees.lock().unwrap();
        let maintenant = Instant::now();

        map.retain(|_cle, (_valeur, horodatage)| {
            maintenant.duration_since(*horodatage) < duree_max
        });
    }
}
```

### `Arc<RwLock<T>>` pour une configuration partagée

```rust
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct AppConfig {
    interne: Arc<RwLock<ConfigInterne>>,
}

struct ConfigInterne {
    page_size: usize,
    site_title: String,
}

impl AppConfig {
    fn page_size(&self) -> usize {
        // Lecture légère — plusieurs threads peuvent lire simultanément
        self.interne.read().unwrap().page_size
    }

    fn definir_page_size(&self, taille: usize) {
        // Écriture exclusive
        self.interne.write().unwrap().page_size = taille;
    }
}
```

---

## 10. Piège — stores partagés sans cleanup

Un store partagé qui grossit sans limite est un piège classique en production.

### Le problème

```rust
// ⚠️ Ce store grossit indéfiniment si personne ne nettoie
static STORE: LazyLock<Arc<Mutex<HashMap<String, Session>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

fn ajouter_session(id: String, session: Session) {
    STORE.lock().unwrap().insert(id, session);
}

// Les sessions expirées restent en mémoire pour toujours
// → OOM silencieux en prod après quelques jours/semaines
```

### Pourquoi c'est silencieux

- Pas d'erreur, pas de panic — juste une RAM qui monte lentement
- En développement, le process redémarre souvent → le problème ne se voit pas
- En prod avec peu de trafic, ça peut prendre des semaines avant de crasher

### La solution générique — thread de nettoyage

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;

struct Store {
    donnees: Arc<Mutex<HashMap<String, (Vec<u8>, Instant)>>>,
}

impl Store {
    fn new() -> Self {
        Self {
            donnees: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Lance un thread de nettoyage en arrière-plan
    fn spawn_cleanup(&self, ttl: Duration, intervalle: Duration) {
        let donnees = Arc::clone(&self.donnees);

        thread::spawn(move || loop {
            thread::sleep(intervalle);

            let maintenant = Instant::now();
            donnees
                .lock()
                .unwrap()
                .retain(|_, (_, horodatage)| {
                    maintenant.duration_since(*horodatage) < ttl
                });
        });
    }
}
```

### Ce que Runique fait

Runique intègre ce pattern directement dans ses builders. Le cleanup est automatiquement configuré et lancé au démarrage :

```rust
// Sessions : limite mémoire + nettoyage toutes les 5 minutes
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
            .with_session_cleanup_interval(5)
    })

// Rate limiter : nettoyage intégré au spawn_cleanup
RateLimiter::new()
    .max_requests(100)
    .retry_after(60)
    .spawn_cleanup(Duration::from_secs(60))

// Login guard : même pattern
LoginGuard::new()
    .max_attempts(5)
    .lockout_secs(300)
    .spawn_cleanup(Duration::from_secs(60))
```

> **Règle** : tout `Arc<Mutex<HashMap>>` qui reçoit des insertions doit avoir un thread de nettoyage. Si ce n'est pas le cas, c'est un bug latent.

---

## 11. Exercices pratiques

### Exercice 1 — Compteur concurrent

Implémentez un compteur thread-safe que plusieurs threads peuvent incrémenter simultanément.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn compteur_concurrent(nb_threads: usize, increments: usize) -> usize {
    let compteur = Arc::new(Mutex::new(0usize));
    let mut handles = vec![];

    for _ in 0..nb_threads {
        let c = Arc::clone(&compteur);

        handles.push(thread::spawn(move || {
            for _ in 0..increments {
                *c.lock().unwrap() += 1;
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    *compteur.lock().unwrap()
}

fn main() {
    let resultat = compteur_concurrent(8, 100);
    println!("Résultat : {resultat}"); // toujours 800
}
```

### Exercice 2 — Cache avec `RwLock`

Implémentez un cache thread-safe utilisant `RwLock` pour maximiser les lectures concurrentes.

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct CacheRW<K, V> {
    donnees: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> CacheRW<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn new() -> Self {
        CacheRW { donnees: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn inserer(&self, cle: K, valeur: V) {
        self.donnees.write().unwrap().insert(cle, valeur);
    }

    fn obtenir(&self, cle: &K) -> Option<V> {
        self.donnees.read().unwrap().get(cle).cloned()
    }

    fn taille(&self) -> usize {
        self.donnees.read().unwrap().len()
    }
}
```

### Exercice 3 — Singleton avec `OnceLock`

Implémentez un pattern singleton thread-safe pour une configuration d'application.

```rust
use std::sync::OnceLock;

struct AppSettings {
    port: u16,
    debug: bool,
    max_connexions: usize,
}

static SETTINGS: OnceLock<AppSettings> = OnceLock::new();

fn init_settings(port: u16, debug: bool, max_connexions: usize) {
    SETTINGS.set(AppSettings { port, debug, max_connexions })
        .expect("Settings déjà initialisés");
}

fn settings() -> &'static AppSettings {
    SETTINGS.get().expect("Settings non initialisés — appeler init_settings d'abord")
}

fn main() {
    init_settings(8080, true, 100);

    println!("Port : {}", settings().port);
    println!("Debug : {}", settings().debug);
}
```

---

## 11. Aide-mémoire

| Primitive | Import | Usage principal |
|---|---|---|
| `Mutex<T>` | `std::sync::Mutex` | Accès exclusif (lecture + écriture) |
| `RwLock<T>` | `std::sync::RwLock` | N lecteurs OU 1 écrivain |
| `Arc<T>` | `std::sync::Arc` | Propriété partagée entre threads |
| `LazyLock<T>` | `std::sync::LazyLock` | Global initialisé paresseusement |
| `OnceLock<T>` | `std::sync::OnceLock` | Global initialisé une seule fois |

**Patterns fréquents :**

```rust
// Partagé + mutable entre threads
let etat = Arc::new(Mutex::new(valeur));

// Partagé + mutable, lectures fréquentes
let config = Arc::new(RwLock::new(valeur));

// Global paresseux
static X: LazyLock<T> = LazyLock::new(|| { ... });

// Global initialisé manuellement
static Y: OnceLock<T> = OnceLock::new();
Y.set(valeur).unwrap();
```

**Points clés :**

- `Mutex` bloque tous les accès — simple, sûr, légèrement moins performant sous forte lecture
- `RwLock` autorise plusieurs lectures simultanées — gain réel si lectures >> écritures
- `Arc` ne permet pas la mutation seul — combinez avec `Mutex` ou `RwLock`
- `LazyLock` remplace `once_cell::sync::Lazy` depuis Rust 1.80
- `OnceLock` remplace `once_cell::sync::OnceCell` depuis Rust 1.70
- Un `Mutex` verrouillé dans un `await` peut bloquer des threads Tokio — préférez `tokio::sync::Mutex` en code async
