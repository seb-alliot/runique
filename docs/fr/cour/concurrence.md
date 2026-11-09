# Concurrence & état partagé
> Threads, verrous, atomiques, canaux et initialisation unique : partager des données entre plusieurs fils d'exécution sans course aux données, sans interblocage et sans fuite.

## Objectifs

- Lancer des threads, leur passer des données et récupérer leur résultat
- Partager une valeur avec `Arc` et la protéger avec `Mutex` ou `RwLock`
- Comprendre l'empoisonnement d'un verrou et savoir le traiter
- Utiliser les types atomiques pour les compteurs et les drapeaux
- Communiquer par canaux plutôt que par mémoire partagée
- Initialiser une valeur globale une seule fois avec `LazyLock` et `OnceLock`
- Éviter les interblocages et les stores qui grossissent sans fin
- Choisir le bon outil pour chaque situation

## Table des matières

1. Pourquoi la concurrence est difficile
2. Les threads
3. `Arc<T>` : partager
4. `Mutex<T>` : exclusion mutuelle
5. `RwLock<T>` : lecteurs multiples
6. Les atomiques
7. Canaux
8. `LazyLock<T>` et `OnceLock<T>`
9. Interblocages
10. Calcul parallèle sur des données
11. Choisir
12. Exemples dans Runique
13. Piège : un store partagé sans nettoyage
14. La même fonction écrite de plusieurs façons
15. Exercices pratiques
16. Aide-mémoire

---

## 1. Pourquoi la concurrence est difficile

Trois familles de bugs apparaissent quand plusieurs fils d'exécution touchent les mêmes données :

| Bug | Description | Ce que fait Rust |
|---|---|---|
| **course aux données** | deux threads accèdent à la même mémoire, l'un écrit, sans synchronisation | impossible à compiler (`Send`, `Sync`, cours `send-sync`) |
| **condition de course logique** | l'ordre des opérations change le résultat (lire puis écrire en deux étapes) | possible : c'est à toi de grouper les opérations sous un même verrou |
| **interblocage** (deadlock) | deux threads attendent chacun un verrou que l'autre tient | possible : discipline sur l'ordre des verrous (section 9) |

Rust supprime la première famille à la compilation. Les deux autres restent des erreurs de conception.

```rust
// Condition de course logique : compile, mais faux
use std::sync::Mutex;
let stock = Mutex::new(1);

// Deux threads font ceci en même temps :
let disponible = *stock.lock().unwrap() > 0;   // verrou pris puis relâché
if disponible {
    *stock.lock().unwrap() -= 1;               // l'autre thread a pu passer entre les deux : stock = -1
}

// Correct : lire et écrire sous le même verrou
let mut s = stock.lock().unwrap();
if *s > 0 {
    *s -= 1;
}
```

---

## 2. Les threads

### 2.1 Lancer et attendre

```rust
use std::thread;

let h = thread::spawn(|| {
    (1..=10).sum::<u32>()
});

let somme = h.join().unwrap();       // attendre la fin ; Err si le thread a paniqué
```

### 2.2 Passer des données : `move`

```rust
let v = vec![1, 2, 3];
let h = thread::spawn(move || v.iter().sum::<i32>());   // v appartient au thread
let s = h.join().unwrap();
```

### 2.3 Plusieurs threads

```rust
let handles: Vec<_> = (0..4)
    .map(|i| thread::spawn(move || i * i))
    .collect();

let resultats: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
```

### 2.4 Emprunter : `thread::scope`

```rust
let donnees = vec![1, 2, 3, 4, 5, 6];
let (gauche, droite) = donnees.split_at(3);

let (a, b) = thread::scope(|s| {
    let ha = s.spawn(|| gauche.iter().sum::<i32>());   // emprunte, sans move ni Arc
    let hb = s.spawn(|| droite.iter().sum::<i32>());
    (ha.join().unwrap(), hb.join().unwrap())
});
```

Un thread lancé dans un `scope` ne peut pas survivre au bloc : il peut donc **emprunter** les variables locales. C'est souvent la façon la plus simple de paralléliser un calcul.

### 2.5 Nommer un thread

```rust
let h = thread::Builder::new()
    .name("nettoyage".into())
    .spawn(|| { /* ... */ })
    .unwrap();
```

Le nom apparaît dans les messages de panique et les outils de profilage.

### 2.6 Threads ou tâches async

Pour des milliers d'attentes réseau : tâches async (cours `async-tokio`). Pour quelques calculs lourds en parallèle : threads. Les primitives de ce cours (`Arc`, `Mutex`, atomiques, `LazyLock`) servent dans les deux cas.

---

## 3. `Arc<T>` : partager

```rust
use std::sync::Arc;
use std::thread;

let config = Arc::new(vec!["a", "b"]);
let mut handles = vec![];
for i in 0..3 {
    let c = Arc::clone(&config);         // copie du pointeur, compteur + 1
    handles.push(thread::spawn(move || println!("{i} : {:?}", c)));
}
for h in handles {
    h.join().unwrap();
}
println!("{}", Arc::strong_count(&config));   // 1 : les threads ont libéré leur copie
```

`Arc` donne **plusieurs propriétaires** en lecture seule. Pour modifier, il faut un verrou ou un atomique **à l'intérieur** : `Arc<Mutex<T>>`, `Arc<RwLock<T>>`, `Arc<AtomicU64>`.

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| Threads | un seul | plusieurs |
| Compteur | simple | atomique (un peu plus lent) |

---

## 4. `Mutex<T>` : exclusion mutuelle

### 4.1 Prendre le verrou

```rust
use std::sync::Mutex;

let m = Mutex::new(0);
{
    let mut garde = m.lock().unwrap();   // un seul thread à la fois ici
    *garde += 1;
}                                        // garde libérée : verrou relâché
```

`lock()` renvoie un `MutexGuard` : il se déréférence vers la valeur, et son `Drop` relâche le verrou. On ne peut **pas** oublier de déverrouiller.

### 4.2 Compteur partagé entre threads

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let compteur = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..10)
    .map(|_| {
        let c = Arc::clone(&compteur);
        thread::spawn(move || *c.lock().unwrap() += 1)
    })
    .collect();
for h in handles {
    h.join().unwrap();
}
assert_eq!(*compteur.lock().unwrap(), 10);
```

### 4.3 Garder le verrou le moins longtemps possible

```rust
// Mauvais : le verrou est tenu pendant tout le traitement
let liste = Mutex::new(vec![1, 2, 3]);
{
    let garde = liste.lock().unwrap();
    for x in garde.iter() {
        traitement_long(*x);             // les autres threads attendent
    }
}

// Mieux : copier, relâcher, traiter
let copie = liste.lock().unwrap().clone();   // verrou relâché à la fin de l'instruction
for x in copie {
    traitement_long(x);
}

fn traitement_long(_: i32) {}
```

### 4.4 L'empoisonnement

Si un thread **panique** pendant qu'il tient le verrou, le `Mutex` est marqué « empoisonné » : la donnée est peut-être dans un état incohérent. Les `lock()` suivants renvoient `Err(PoisonError)`.

```rust
use std::sync::Mutex;

let m = Mutex::new(vec![1]);

// 1. unwrap : propager la panique (choix par défaut le plus courant)
let g = m.lock().unwrap();
drop(g);

// 2. Récupérer quand même la donnée, si elle ne peut pas être incohérente
let g = match m.lock() {
    Ok(g) => g,
    Err(empoisonne) => empoisonne.into_inner(),
};
drop(g);

// 3. Version courte de 2
let g = m.lock().unwrap_or_else(|e| e.into_inner());
```

La forme 2 convient à un cache ou à une table de compteurs, où une entrée à moitié écrite n'est pas grave : continuer vaut mieux qu'arrêter le service. C'est ce que fait Runique dans ses limiteurs (section 12).

### 4.5 Essayer sans attendre

```rust
match m.try_lock() {
    Ok(mut g) => g.push(2),
    Err(_) => println!("occupé : on réessaiera plus tard"),
}
```

### 4.6 Accès sans verrou quand on est seul

```rust
let mut m = Mutex::new(5);
*m.get_mut().unwrap() += 1;        // &mut m prouve qu'aucun autre accès n'existe : pas de verrou
let valeur = m.into_inner().unwrap();   // récupérer la donnée et détruire le Mutex
```

---

## 5. `RwLock<T>` : lecteurs multiples

```rust
use std::sync::{Arc, RwLock};

let config = Arc::new(RwLock::new(String::from("v1")));

let lecture = config.read().unwrap();      // plusieurs lecteurs en même temps
println!("{}", *lecture);
drop(lecture);

let mut ecriture = config.write().unwrap(); // un seul écrivain, aucun lecteur
*ecriture = String::from("v2");
```

| | `Mutex<T>` | `RwLock<T>` |
|---|---|---|
| Lecteurs simultanés | non | oui |
| Coût | plus faible | plus élevé |
| Pour | lectures et écritures fréquentes, sections courtes | beaucoup de lectures, rares écritures (configuration, cache) |
| Exigence sur `T` pour partager | `Send` | `Send + Sync` |

Un `RwLock` n'est pas toujours plus rapide : pour des sections très courtes, un `Mutex` est souvent meilleur. Et un écrivain peut attendre longtemps si les lecteurs se succèdent sans interruption.

---

## 6. Les atomiques

### 6.1 Compteurs et drapeaux sans verrou

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

let compteur = Arc::new(AtomicU64::new(0));
let handles: Vec<_> = (0..10)
    .map(|_| {
        let c = Arc::clone(&compteur);
        thread::spawn(move || {
            c.fetch_add(1, Ordering::Relaxed);
        })
    })
    .collect();
for h in handles {
    h.join().unwrap();
}
assert_eq!(compteur.load(Ordering::Relaxed), 10);

static ARRET: AtomicBool = AtomicBool::new(false);
ARRET.store(true, Ordering::Release);
if ARRET.load(Ordering::Acquire) { /* arrêter */ }
```

### 6.2 Opérations

| Méthode | Effet |
|---|---|
| `load`, `store` | lire, écrire |
| `fetch_add`, `fetch_sub` | ajouter, retirer ; renvoie l'ancienne valeur |
| `fetch_max`, `fetch_min` | garder le maximum, le minimum |
| `swap` | remplacer, renvoie l'ancienne valeur |
| `compare_exchange(attendu, nouveau, ..)` | remplacer seulement si la valeur est celle attendue |

```rust
use std::sync::atomic::{AtomicU32, Ordering};

// Réserver une place seulement s'il en reste : lecture et écriture en une seule opération
fn reserver(places: &AtomicU32) -> bool {
    places
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |p| p.checked_sub(1))
        .is_ok()
}
```

### 6.3 `Ordering` en pratique

| Ordering | Usage |
|---|---|
| `Relaxed` | compteur, statistique : seul le nombre compte |
| `Release` (écriture) / `Acquire` (lecture) | un drapeau qui publie d'autres données (« prêt » après avoir rempli un tampon) |
| `AcqRel` | lecture-modification-écriture qui fait les deux |
| `SeqCst` | ordre global strict ; le plus sûr en cas de doute |

Un atomique ne protège **qu'une** valeur. Pour garder cohérentes deux valeurs liées (un total et un nombre), il faut un `Mutex`.

---

## 7. Canaux

### 7.1 `std::sync::mpsc`

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();
for i in 0..3 {
    let tx = tx.clone();
    thread::spawn(move || tx.send(i * 10).unwrap());
}
drop(tx);                                   // fermer : sinon rx attend pour toujours

let total: i32 = rx.iter().sum();           // reçoit jusqu'à la fermeture
```

### 7.2 Canal limité

```rust
let (tx, rx) = mpsc::sync_channel::<String>(10);   // au-delà de 10, send bloque
```

### 7.3 Mémoire partagée ou messages

```rust
// 1. Mémoire partagée : plusieurs threads modifient la même Vec sous un verrou
// 2. Messages : chaque thread envoie ses résultats ; un seul propriétaire assemble
```

Les messages évitent les verrous et les interblocages : une seule tâche possède la donnée. Pour l'asynchrone, tokio fournit ses propres canaux (`mpsc`, `oneshot`, `broadcast`, `watch`, voir `async-tokio`). La crate `crossbeam-channel` offre des canaux std plus riches (plusieurs récepteurs, `select!`).

---

## 8. `LazyLock<T>` et `OnceLock<T>`

### 8.1 `LazyLock` : valeur globale calculée au premier accès

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

static CODES: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    HashMap::from([(404, "introuvable"), (500, "erreur interne")])
});

fn message(code: u16) -> &'static str {
    CODES.get(&code).copied().unwrap_or("inconnu")
}
```

- La closure s'exécute **une seule fois**, au premier accès, même si plusieurs threads y accèdent en même temps.
- Disponible depuis Rust 1.80 : remplace `lazy_static!` et `once_cell::sync::Lazy`.

### 8.2 Avant Rust 1.80

```rust
// lazy_static (crate)
// lazy_static! { static ref CODES: HashMap<u16, &'static str> = { .. }; }

// once_cell (crate)
// static CODES: once_cell::sync::Lazy<HashMap<..>> = Lazy::new(|| ..);

// Aujourd'hui : std uniquement
// static CODES: std::sync::LazyLock<HashMap<..>> = LazyLock::new(|| ..);
```

### 8.3 `OnceLock` : valeur fixée plus tard, une seule fois

```rust
use std::sync::OnceLock;

static URL_BASE: OnceLock<String> = OnceLock::new();

fn initialiser(url: String) {
    if URL_BASE.set(url).is_err() {
        panic!("URL_BASE déjà initialisée");
    }
}

fn url_base() -> &'static str {
    URL_BASE.get().map(String::as_str).expect("initialiser() doit être appelée au démarrage")
}

// Ou : calculée à la première demande, depuis une fonction
fn cache() -> &'static Vec<u32> {
    static C: OnceLock<Vec<u32>> = OnceLock::new();
    C.get_or_init(|| (1..=10).collect())
}
```

### 8.4 `LazyLock` ou `OnceLock`

| | `LazyLock` | `OnceLock` |
|---|---|---|
| La valeur est connue | à la déclaration (une fonction) | plus tard (au démarrage, depuis la configuration) |
| Initialisation | automatique au premier accès | `set` explicite, ou `get_or_init` |
| Pour | tables constantes, regex, lecture de l'environnement | configuration reçue au démarrage, valeur passée par `main` |

### 8.5 Le piège de l'initialisation paresseuse

Une `LazyLock` qui lit une variable d'environnement la lit **au premier accès**, pas au démarrage du programme. Si le premier accès a lieu avant le chargement de `.env`, la valeur lue est celle d'avant. Et une fois calculée, elle ne change plus jusqu'à la fin du programme : modifier `.env` demande un redémarrage.

---

## 9. Interblocages

### 9.1 Deux verrous pris dans un ordre différent

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let a = Arc::new(Mutex::new(0));
let b = Arc::new(Mutex::new(0));

let (a1, b1) = (Arc::clone(&a), Arc::clone(&b));
let t1 = thread::spawn(move || {
    let _ga = a1.lock().unwrap();
    let _gb = b1.lock().unwrap();      // attend b
});

let (a2, b2) = (Arc::clone(&a), Arc::clone(&b));
let t2 = thread::spawn(move || {
    let _gb = b2.lock().unwrap();
    let _ga = a2.lock().unwrap();      // attend a : si t1 tient a et attend b, les deux attendent pour toujours
});
```

### 9.2 Les règles

1. **Toujours le même ordre** : si un code prend `a` puis `b`, tout le code prend `a` puis `b`.
2. **Un seul verrou à la fois** quand c'est possible : regrouper les données liées dans un seul `Mutex<Struct>`.
3. **Ne jamais reprendre un verrou qu'on tient déjà** : un `Mutex` std n'est pas réentrant, un second `lock()` dans le même thread bloque pour toujours.
4. **Ne rien appeler d'inconnu** (closure de l'utilisateur, fonction qui pourrait prendre un verrou) en tenant un verrou.

```rust
let m = std::sync::Mutex::new(0);
let _g = m.lock().unwrap();
// let _g2 = m.lock().unwrap();   // interblocage dans le même thread
```

---

## 10. Calcul parallèle sur des données

```rust
use rayon::prelude::*;

let v: Vec<u64> = (1..=1_000_000).collect();

let somme: u64 = v.iter().sum();                      // un cœur
let somme_par: u64 = v.par_iter().sum();              // tous les cœurs

let carres: Vec<u64> = v.par_iter().map(|x| x * x).collect();
let mut a_trier = v.clone();
a_trier.par_sort_unstable();
```

La crate `rayon` remplace `iter()` par `par_iter()` : le travail est réparti sur un ensemble de threads, sans verrou ni canal à écrire. Le compilateur vérifie que la closure peut être exécutée en parallèle (`Send + Sync`). À réserver aux calculs suffisamment gros : pour quelques milliers d'éléments simples, le découpage coûte plus qu'il ne rapporte.

---

## 11. Choisir

| Besoin | Outil |
|---|---|
| lancer un calcul en parallèle et récupérer le résultat | `thread::spawn` + `join`, ou `thread::scope` |
| paralléliser une boucle sur une collection | `rayon` |
| partager une valeur en lecture seule | `Arc<T>` |
| partager et modifier, sections courtes | `Arc<Mutex<T>>` |
| beaucoup de lectures, rares écritures | `Arc<RwLock<T>>` |
| compteur, drapeau, identifiant à incrémenter | `AtomicU64`, `AtomicBool` |
| producteurs et consommateur | canal `mpsc` |
| valeur globale constante, calculée une fois | `LazyLock` |
| valeur globale fixée au démarrage | `OnceLock` |
| état partagé dans du code async avec attente | `tokio::sync::Mutex` (voir `async-tokio`) |

---

## 12. Exemples dans Runique

### 12.1 `LazyLock` pour l'environnement

```rust
// runique/src/utils/config/env.rs (extrait)
static DEBUG: LazyLock<bool> = LazyLock::new(|| debug_from(read_after_dotenv("DEBUG").as_deref()));

pub fn is_debug() -> bool {
    *DEBUG
}

// Jeton de version des fichiers statiques (?v=...), calculé une seule fois :
// une empreinte du contenu des .css/.js du projet et de Runique.
static CSS_TOKEN: LazyLock<String> = LazyLock::new(|| { /* ... */ });
```

`read_after_dotenv` charge `.env` avant de lire la variable : c'est la réponse au piège de la section 8.5. Et puisque la valeur n'est calculée qu'une fois, un fichier CSS modifié en production n'est pris en compte qu'au redémarrage du serveur.

### 12.2 `Arc<Mutex<HashMap>>` avec empoisonnement toléré

Le limiteur de requêtes (`RateLimiter`) et le garde de connexion (`LoginGuard`) gardent leurs compteurs dans un `Arc<Mutex<HashMap<...>>>`. Leur nettoyage :

```rust
// runique/src/middleware/security/rate_limit.rs (extrait)
pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
    let store = self.store.clone();
    let window = self.window;
    let period = period.max(tokio::time::Duration::from_secs(1));
    tokio::spawn(async move {
        let mut ticker = interval(period);
        loop {
            ticker.tick().await;
            let mut guard = match store.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),       // section 4.4 : un compteur à moitié écrit n'est pas grave
            };
            let now = Instant::now();
            guard.retain(|_, (_, start)| now.duration_since(*start) < window);
        }
    });
}
```

On y voit : `Arc` cloné avant le `move` (section 3), `std::sync::Mutex` pris et relâché **sans `.await` entre les deux** (le verrou est libéré à la fin du tour de boucle, avant le prochain `tick().await`), empoisonnement toléré, et `retain` pour supprimer les entrées expirées.

---

## 13. Piège : un store partagé sans nettoyage

### 13.1 Le problème

```rust
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static TENTATIVES: LazyLock<Mutex<HashMap<String, u32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn echec_de_connexion(ip: &str) {
    *TENTATIVES.lock().unwrap().entry(ip.to_string()).or_insert(0) += 1;
}
// Chaque adresse IP qui se trompe ajoute une entrée, jamais retirée.
```

### 13.2 Pourquoi c'est grave

- Aucune erreur, aucune panique : la mémoire monte lentement.
- En développement, le processus redémarre souvent : le problème ne se voit pas.
- Un attaquant qui varie les adresses ou les identifiants peut **remplir la mémoire volontairement** : c'est un déni de service.

### 13.3 Les protections

1. **Nettoyer périodiquement** les entrées expirées (section 12.2).
2. **Borner la taille** : refuser ou évincer au-delà d'un maximum (une crate comme `lru` ou `moka`).
3. **Expirer à la lecture** : une entrée trop vieille est ignorée et supprimée quand on la consulte.

### 13.4 Ce que Runique fait

Les extensions de routes et le builder de l'admin lancent eux-mêmes le nettoyage quand ils créent un limiteur :

```rust
// runique/src/macros/routeur/router_ext.rs (extrait)
let limiter = Arc::new(limiter);
limiter.spawn_cleanup(limiter.cleanup_interval());
```

Pour les sessions, le builder règle une limite de mémoire et une période de nettoyage :

```rust
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)   // seuils bas et haut, en octets
            .with_session_cleanup_interval(300)                           // en secondes
    })
```

Au seuil bas, les sessions anonymes expirées sont nettoyées sans bloquer ; au seuil haut, un nettoyage immédiat a lieu, et si la limite est encore dépassée, la requête est refusée (503) plutôt que de laisser la mémoire grossir.

Un `RateLimiter` ou un `LoginGuard` créé à la main (hors de ces extensions) doit appeler `spawn_cleanup(..)` une fois au démarrage : c'est ce que précise sa documentation.

> **Règle** : toute table partagée qui reçoit des insertions venant de l'extérieur doit avoir un nettoyage ou une taille maximale.

---

## 14. La même fonction écrite de plusieurs façons

Compter les mots de plusieurs textes en parallèle.

```rust
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn compter(t: &str) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    for mot in t.split_whitespace() {
        *m.entry(mot.to_lowercase()).or_insert(0) += 1;
    }
    m
}

fn fusionner(total: &mut HashMap<String, usize>, partiel: HashMap<String, usize>) {
    for (k, v) in partiel {
        *total.entry(k).or_insert(0) += v;
    }
}

// 1. Mémoire partagée : chaque thread écrit dans une table commune sous verrou
fn v1(textes: Vec<String>) -> HashMap<String, usize> {
    let total = Arc::new(Mutex::new(HashMap::new()));
    let hs: Vec<_> = textes.into_iter().map(|t| {
        let total = Arc::clone(&total);
        thread::spawn(move || {
            let partiel = compter(&t);                       // calcul hors verrou
            fusionner(&mut total.lock().unwrap(), partiel);  // verrou seulement pour fusionner
        })
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    Arc::try_unwrap(total).unwrap().into_inner().unwrap()
}

// 2. Canal : chaque thread envoie son résultat, le thread principal fusionne
fn v2(textes: Vec<String>) -> HashMap<String, usize> {
    let (tx, rx) = mpsc::channel();
    for t in textes {
        let tx = tx.clone();
        thread::spawn(move || tx.send(compter(&t)).unwrap());
    }
    drop(tx);
    let mut total = HashMap::new();
    for partiel in rx {
        fusionner(&mut total, partiel);
    }
    total
}

// 3. scope + join : les threads empruntent les textes, renvoient leur résultat
fn v3(textes: &[String]) -> HashMap<String, usize> {
    thread::scope(|s| {
        let hs: Vec<_> = textes.iter().map(|t| s.spawn(move || compter(t))).collect();
        let mut total = HashMap::new();
        for h in hs {
            fusionner(&mut total, h.join().unwrap());
        }
        total
    })
}

// 4. rayon : map en parallèle, puis réduction
fn v4(textes: &[String]) -> HashMap<String, usize> {
    use rayon::prelude::*;
    textes
        .par_iter()
        .map(|t| compter(t))
        .reduce(HashMap::new, |mut a, b| { fusionner(&mut a, b); a })
}
```

| Version | Verrou | Copie des textes | Simplicité |
|---|---|---|---|
| 1 | oui | déplacés dans les threads | moyenne |
| 2 | non | déplacés | bonne |
| 3 | non | empruntés | très bonne |
| 4 | non | empruntés, nombre de threads adapté aux cœurs | excellente |

---

## 15. Exercices pratiques

### Exercice 1 — compteur concurrent

Incrémente un compteur depuis 8 threads, 1 000 fois chacun, d'abord avec `Arc<Mutex<u64>>`, puis avec un atomique. Vérifie que le total vaut 8 000.

```rust
// Solution :
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn avec_mutex() -> u64 {
    let c = Arc::new(Mutex::new(0u64));
    let hs: Vec<_> = (0..8).map(|_| {
        let c = Arc::clone(&c);
        thread::spawn(move || for _ in 0..1000 { *c.lock().unwrap() += 1; })
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    let total = *c.lock().unwrap();
    total
}

fn avec_atomique() -> u64 {
    let c = Arc::new(AtomicU64::new(0));
    thread::scope(|s| {
        for _ in 0..8 {
            s.spawn(|| for _ in 0..1000 { c.fetch_add(1, Ordering::Relaxed); });
        }
    });
    c.load(Ordering::Relaxed)
}
```

### Exercice 2 — cache avec `RwLock`

Écris un cache `get_or_compute(cle, f)` : lecture partagée si la clé existe, sinon calcul puis écriture.

```rust
// Solution :
use std::collections::HashMap;
use std::sync::RwLock;

struct Cache { m: RwLock<HashMap<String, String>> }

impl Cache {
    fn get_or_compute(&self, cle: &str, f: impl FnOnce() -> String) -> String {
        if let Some(v) = self.m.read().unwrap().get(cle) {
            return v.clone();                         // verrou de lecture relâché ici
        }
        let mut w = self.m.write().unwrap();
        w.entry(cle.to_string()).or_insert_with(f).clone()   // revérifie : un autre thread a pu calculer
    }
}
```

`entry` sous le verrou d'écriture évite de calculer deux fois si deux threads ont raté la lecture en même temps.

### Exercice 3 — configuration unique

Fais en sorte qu'une configuration ne puisse être fixée qu'une fois au démarrage, puis lue partout.

```rust
// Solution :
use std::sync::OnceLock;

#[derive(Debug)]
struct Config { port: u16 }

static CONFIG: OnceLock<Config> = OnceLock::new();

fn initialiser(c: Config) -> Result<(), Config> {
    CONFIG.set(c)                                     // Err(c) si déjà fixée
}

fn config() -> &'static Config {
    CONFIG.get().expect("initialiser() doit être appelée dans main")
}
```

---

## 16. Aide-mémoire

```rust
thread::spawn(move || ..).join()         thread::scope(|s| { s.spawn(|| ..); })
Arc::new(v)   Arc::clone(&a)             Arc::try_unwrap(a)
Mutex::new(v)   m.lock().unwrap()        m.lock().unwrap_or_else(|e| e.into_inner())
m.try_lock()   m.get_mut()   m.into_inner()
RwLock::new(v)   r.read()   r.write()
AtomicU64::new(0)   a.fetch_add(1, Ordering::Relaxed)   a.load(..)   a.fetch_update(..)
mpsc::channel()   mpsc::sync_channel(n)   drop(tx)   rx.iter()
static X: LazyLock<T> = LazyLock::new(|| ..);
static Y: OnceLock<T> = OnceLock::new();   Y.set(v)   Y.get()   Y.get_or_init(|| ..)
v.par_iter().map(f).sum()                // rayon
```

### Points clés

- Rust empêche les courses aux données ; les conditions de course logiques et les interblocages restent à éviter.
- Lire puis écrire sous le **même** verrou.
- Garder un verrou le moins longtemps possible ; jamais d'appel lent ni d'`.await` en le tenant.
- Un verrou empoisonné peut être récupéré avec `into_inner` quand la donnée ne peut pas être incohérente.
- Atomiques pour une valeur isolée, `Mutex` pour des valeurs liées.
- Toujours prendre plusieurs verrous dans le même ordre.
- Les canaux et `thread::scope` évitent souvent tout verrou.
- `LazyLock` lit au premier accès et ne change plus ; toute table alimentée de l'extérieur doit être nettoyée ou bornée.
