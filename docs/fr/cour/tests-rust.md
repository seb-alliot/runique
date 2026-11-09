# Tests en Rust
> Tests unitaires, d'intégration, de documentation et asynchrones : toutes les façons d'écrire une vérification, d'organiser les tests et de les lancer.

## Objectifs

- Écrire un test et choisir la bonne macro d'assertion
- Tester un échec : panique attendue, `Result`, motif d'erreur
- Placer les tests au bon endroit : module `tests`, dossier `tests/`, documentation
- Partager du code de préparation entre tests
- Tester du code asynchrone, du code qui touche une base, du code qui dépend du temps ou du réseau
- Lancer précisément les tests voulus avec `cargo test`
- Savoir si les tests vérifient vraiment quelque chose (couverture, mutants)

---

## Table des matières

1. Le premier test
2. Les macros d'assertion
3. Tester un échec
4. Tests qui renvoient un `Result`
5. Ignorer un test
6. Le module de tests unitaires
7. Tests d'intégration
8. Tests de documentation
9. Préparer et nettoyer
10. Tests paramétrés
11. Tests asynchrones
12. Isoler les dépendances : base, temps, réseau
13. Tests et features
14. Commandes `cargo test`
15. Vérifier la qualité des tests
16. Bonnes pratiques
17. Exercices
18. Aide-mémoire

---

## 1. Le premier test

```rust
pub fn ajouter(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ajoute_deux_positifs() {
        assert_eq!(ajouter(2, 3), 5);
    }
}
```

- `#[test]` marque une fonction comme test : `cargo test` la trouve et l'exécute.
- Un test **réussit** s'il se termine sans paniquer, **échoue** s'il panique.
- `#[cfg(test)]` : le module n'est compilé que pour `cargo test`, il n'alourdit pas le binaire final.
- `use super::*;` importe tout le module parent, y compris les éléments privés.

Chaque test s'exécute dans **son propre thread**, en parallèle des autres.

---

## 2. Les macros d'assertion

### 2.1 — Les trois macros de base

```rust
#[test]
fn assertions() {
    let x = 5;
    assert!(x > 0);                 // condition vraie
    assert_eq!(x, 5);               // égalité : affiche les deux valeurs en cas d'échec
    assert_ne!(x, 0);               // différence
}
```

`assert_eq!` et `assert_ne!` demandent que les valeurs implémentent `PartialEq` et `Debug`.

### 2.2 — Ajouter un message

```rust
#[test]
fn avec_message() {
    let age = 17;
    assert!(age >= 18, "âge refusé : {age}, il faut au moins 18");
    assert_eq!(age, 18, "l'âge lu depuis le formulaire est {age}");
}
```

Le message s'affiche **en plus** des valeurs comparées. Il sert à dire ce que le test vérifiait, surtout dans une boucle.

### 2.3 — Pourquoi préférer `assert_eq!` à `assert!(a == b)`

```rust
assert!(resultat == 42);       // échec : "assertion failed: resultat == 42"
assert_eq!(resultat, 42);      // échec : "left: 41, right: 42"
```

La seconde forme montre la valeur obtenue, ce qui évite de relancer le test avec des `println!`.

### 2.4 — Tester une forme : `matches!`

```rust
#[derive(Debug)]
enum Erreur { Introuvable, Interdit(String) }

fn charger(id: u32) -> Result<u32, Erreur> {
    if id == 0 { Err(Erreur::Interdit("anonyme".into())) } else { Ok(id) }
}

#[test]
fn refuse_l_anonyme() {
    // Écriture 1 : matches! — pas besoin de PartialEq sur Erreur
    assert!(matches!(charger(0), Err(Erreur::Interdit(_))));

    // Écriture 2 : match explicite, pour vérifier aussi le contenu
    match charger(0) {
        Err(Erreur::Interdit(raison)) => assert_eq!(raison, "anonyme"),
        autre => panic!("attendu Interdit, obtenu {autre:?}"),
    }

    // Écriture 3 : let else
    let Err(Erreur::Interdit(raison)) = charger(0) else {
        panic!("attendu Interdit");
    };
    assert_eq!(raison, "anonyme");
}
```

### 2.5 — Comparer des flottants

```rust
#[test]
fn flottants() {
    let r = 0.1 + 0.2;
    // assert_eq!(r, 0.3);                        // échoue : 0.30000000000000004
    assert!((r - 0.3_f64).abs() < 1e-10, "obtenu {r}");
}
```

### 2.6 — Comparer des collections

```rust
use std::collections::HashSet;

#[test]
fn collections() {
    let mut obtenu = vec![3, 1, 2];
    obtenu.sort();
    assert_eq!(obtenu, [1, 2, 3]);       // Vec comparé à un tableau : accepté

    let a: HashSet<_> = [1, 2].into();
    let b: HashSet<_> = [2, 1].into();
    assert_eq!(a, b);                    // l'ordre ne compte pas dans un ensemble
}
```

Si l'ordre d'un résultat n'est pas garanti (parcours d'une `HashMap`), trie-le ou compare des ensembles : sinon le test échouera au hasard.

---

## 3. Tester un échec

### 3.1 — `#[should_panic]`

```rust
pub fn diviser(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("division par zéro");
    }
    a / b
}

#[test]
#[should_panic]
fn panique_sur_zero() {
    diviser(1, 0);
}

#[test]
#[should_panic(expected = "division par zéro")]
fn panique_avec_le_bon_message() {
    diviser(1, 0);
}
```

Sans `expected`, le test réussit sur **n'importe quelle** panique, y compris un bug sans rapport (un index hors limites ailleurs). Mets toujours `expected` : c'est une sous-chaîne du message de panique.

### 3.2 — Tester un `Result` d'erreur

La plupart des fonctions renvoient une erreur plutôt que de paniquer. On vérifie alors la valeur :

```rust
fn parse_age(t: &str) -> Result<u8, String> {
    t.parse().map_err(|_| format!("âge invalide : {t}"))
}

#[test]
fn refuse_un_texte() {
    // 1. is_err
    assert!(parse_age("abc").is_err());

    // 2. unwrap_err : récupérer l'erreur pour la vérifier
    let e = parse_age("abc").unwrap_err();
    assert_eq!(e, "âge invalide : abc");

    // 3. comparer tout le Result
    assert_eq!(parse_age("12"), Ok(12));
}
```

`is_err` seul vérifie qu'il y a une erreur, pas que c'est **la bonne**. Vérifie le contenu quand plusieurs erreurs sont possibles.

### 3.3 — Capturer une panique dans le test

```rust
#[test]
fn panique_capturee() {
    let r = std::panic::catch_unwind(|| diviser(1, 0));
    assert!(r.is_err());
}
```

Utile pour vérifier plusieurs paniques dans un même test, ou continuer après.

---

## 4. Tests qui renvoient un `Result`

```rust
#[test]
fn lit_la_config() -> Result<(), Box<dyn std::error::Error>> {
    let n: i32 = "42".parse()?;          // ? au lieu de unwrap
    assert_eq!(n, 42);
    Ok(())
}
```

Le test échoue si la fonction renvoie `Err`. À quoi ça sert : enchaîner des étapes qui peuvent échouer sans écrire `.unwrap()` partout ; le message d'échec affiche l'erreur.

Un test qui renvoie `Result` ne peut pas utiliser `#[should_panic]`. Pour tester une erreur, vérifie `is_err()` (section 3.2).

| Écriture | Message en cas d'échec |
|---|---|
| `.unwrap()` | `called Result::unwrap() on an Err value: ...` + ligne |
| `.expect("lecture de la config")` | ton message + l'erreur |
| `?` dans un test `-> Result` | l'erreur (sans la ligne exacte) |

---

## 5. Ignorer un test

```rust
#[test]
#[ignore = "lent : demande une base Postgres"]
fn import_complet() {}
```

```bash
cargo test                       # import_complet est sauté
cargo test -- --ignored          # seulement les ignorés
cargo test -- --include-ignored  # tout
```

À quoi ça sert : des tests lents, ou qui demandent un service externe, qu'on ne lance pas à chaque modification. Pour sauter un test selon l'environnement, on vérifie dans le test :

```rust
#[test]
fn avec_postgres() {
    let Ok(url) = std::env::var("DATABASE_URL_PG") else {
        eprintln!("DATABASE_URL_PG absente : test sauté");
        return;
    };
    // ...
}
```

Attention : un test sauté ainsi apparaît comme **réussi**. Garde une trace (message) et assure-toi que la CI fournit la variable.

---

## 6. Le module de tests unitaires

### 6.1 — Dans le même fichier

```rust
// src/calcul.rs
pub fn moyenne(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    Some(somme(v) / v.len() as f64)
}

fn somme(v: &[f64]) -> f64 {
    v.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moyenne_de_rien_est_none() {
        assert_eq!(moyenne(&[]), None);
    }

    #[test]
    fn somme_privee() {
        assert_eq!(somme(&[1.0, 2.0]), 3.0);   // les tests unitaires voient le privé
    }
}
```

### 6.2 — Dans un fichier séparé

```rust
// src/calcul.rs
#[cfg(test)]
mod tests;              // contenu dans src/calcul/tests.rs
```

```rust
// src/calcul/tests.rs
use super::*;

#[test]
fn moyenne_simple() {
    assert_eq!(moyenne(&[2.0, 4.0]), Some(3.0));
}
```

À quoi ça sert : garder le fichier de code lisible quand les tests deviennent longs, tout en conservant l'accès au privé.

### 6.3 — Fonctions d'aide dans le module de tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn donnees() -> Vec<f64> {      // pas de #[test] : simple fonction d'aide
        vec![1.0, 2.0, 3.0]
    }

    #[test]
    fn utilise_l_aide() {
        assert_eq!(moyenne(&donnees()), Some(2.0));
    }
}
```

---

## 7. Tests d'intégration

### 7.1 — Le dossier `tests/`

```
mon_projet/
├── src/
│   └── lib.rs
└── tests/
    ├── api.rs          ← un binaire de test
    ├── formulaires.rs  ← un autre binaire de test
    └── common/
        └── mod.rs      ← code partagé (pas un test)
```

```rust
// tests/api.rs
use mon_projet::Calculatrice;      // seulement l'API publique, comme un utilisateur

#[test]
fn calcul_complet() {
    let mut c = Calculatrice::new(10.0);
    c.ajouter(5.0);
    assert_eq!(c.resultat(), 15.0);
}
```

- Chaque fichier de `tests/` est compilé comme une **crate séparée** qui dépend de ta bibliothèque.
- Pas besoin de `#[cfg(test)]` : tout le dossier n'est compilé que pour les tests.
- Ils ne voient que le `pub` : ils vérifient l'API telle que les utilisateurs la verront.
- Un package qui n'a qu'un `main.rs` (pas de `lib.rs`) ne peut pas avoir de tests d'intégration sur sa logique : une raison de plus de mettre la logique dans `lib.rs`.

### 7.2 — Code partagé entre fichiers de test

```rust
// tests/common/mod.rs
pub fn calculatrice_initialisee() -> mon_projet::Calculatrice {
    mon_projet::Calculatrice::new(100.0)
}
```

```rust
// tests/formulaires.rs
mod common;

#[test]
fn avec_aide() {
    let c = common::calculatrice_initialisee();
    assert_eq!(c.resultat(), 100.0);
}
```

Pourquoi `common/mod.rs` et pas `common.rs` : un fichier `tests/common.rs` serait pris pour un binaire de test (affiché « running 0 tests »). Un sous-dossier n'est pas traité comme un test.

### 7.3 — Un seul binaire pour tous les tests

```
tests/
├── main.rs        ← mod api; mod formulaires; mod common;
├── api.rs
└── formulaires.rs
```

Chaque fichier de `tests/` est lié séparément : avec beaucoup de fichiers, la compilation et l'édition de liens prennent du temps. Regrouper en un binaire (un `main.rs` qui déclare les autres comme modules) accélère nettement les gros projets.

### 7.4 — Unitaire ou intégration

| | Unitaire (`mod tests`) | Intégration (`tests/`) |
|---|---|---|
| Voit le privé | oui | non |
| Teste | une fonction, un détail | un parcours complet par l'API publique |
| Casse quand | le détail interne change | l'API publique change |
| Compilation | avec la crate | binaire séparé par fichier |

---

## 8. Tests de documentation

### 8.1 — Les exemples sont des tests

```rust
/// Additionne deux nombres.
///
/// # Exemples
///
/// ```
/// let r = mon_projet::ajouter(2, 3);
/// assert_eq!(r, 5);
/// ```
pub fn ajouter(a: i32, b: i32) -> i32 {
    a + b
}
```

`cargo test` compile et exécute chaque bloc de code de la documentation. À quoi ça sert : la documentation ne peut pas devenir fausse sans que les tests échouent.

### 8.2 — Lignes cachées

Dans un exemple de documentation, une ligne commençant par `# ` est exécutée mais n'est pas affichée dans la doc :

```rust
/// ```
/// # use mon_projet::ajouter;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let n: i32 = "2".parse()?;
/// assert_eq!(ajouter(n, 1), 3);
/// # Ok(())
/// # }
/// ```
```

Elles servent à cacher les imports et l'enveloppe nécessaire à `?`.

### 8.3 — Annotations de bloc

| Annotation | Effet |
|---|---|
| ` ``` ` | compilé et exécuté |
| ` ```no_run ` | compilé, pas exécuté (accès réseau, boucle serveur) |
| ` ```should_panic ` | doit paniquer |
| ` ```compile_fail ` | doit **refuser** de compiler |
| ` ```ignore ` | ni compilé ni exécuté (à éviter : l'exemple peut se périmer) |
| ` ```text ` | pas du Rust : simplement affiché |

`compile_fail` sert à prouver qu'une API empêche une erreur : par exemple qu'on ne peut pas construire une preuve d'autorisation hors du module prévu.

```rust
/// ```compile_fail
/// let a = mon_projet::Authorized { };   // constructeur privé : ne doit pas compiler
/// ```
pub struct Authorized { _prive: () }
```

---

## 9. Préparer et nettoyer

Rust n'a pas de `setUp`/`tearDown` intégrés. Trois façons d'obtenir le même effet :

### 9.1 — Une fonction d'aide

```rust
fn panier_avec_deux_articles() -> Vec<(String, u32)> {
    vec![("pomme".into(), 2), ("poire".into(), 1)]
}

#[test]
fn total() {
    let p = panier_avec_deux_articles();
    assert_eq!(p.iter().map(|(_, n)| n).sum::<u32>(), 3);
}
```

### 9.2 — Nettoyage automatique avec `Drop`

```rust
struct DossierTemporaire(std::path::PathBuf);

impl DossierTemporaire {
    fn new(nom: &str) -> Self {
        let p = std::env::temp_dir().join(nom);
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}

impl Drop for DossierTemporaire {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);   // exécuté même si le test panique
    }
}

#[test]
fn ecrit_un_fichier() {
    let dossier = DossierTemporaire::new("test_ecriture");
    std::fs::write(dossier.0.join("a.txt"), "x").unwrap();
    assert!(dossier.0.join("a.txt").exists());
}   // dossier supprimé ici
```

`Drop` s'exécute aussi quand le test échoue : le nettoyage est garanti, contrairement à un appel en fin de fonction qui serait sauté par la panique. La crate `tempfile` fournit ce type tout fait.

### 9.3 — Préparation partagée et coûteuse

```rust
use std::sync::LazyLock;

static DONNEES: LazyLock<Vec<u32>> = LazyLock::new(|| {
    (0..1_000_000).collect()     // calculé une fois pour tous les tests
});

#[test]
fn utilise_les_donnees() {
    assert_eq!(DONNEES.len(), 1_000_000);
}
```

Réserve-la aux données **en lecture seule** : les tests tournent en parallèle et ne doivent pas se modifier l'un l'autre.

---

## 10. Tests paramétrés

### 10.1 — Une boucle sur des cas

```rust
fn est_pair(n: i32) -> bool { n % 2 == 0 }

#[test]
fn est_pair_cas_multiples() {
    let cas = [(0, true), (1, false), (2, true), (-3, false), (i32::MAX, false)];
    for (entree, attendu) in cas {
        assert_eq!(est_pair(entree), attendu, "est_pair({entree})");
    }
}
```

Simple, mais le test s'arrête au premier échec et n'apparaît que comme **un** test.

### 10.2 — Une macro qui génère un test par cas

```rust
macro_rules! cas_pair {
    ($($nom:ident: $entree:expr => $attendu:expr,)*) => {
        $(
            #[test]
            fn $nom() {
                assert_eq!(est_pair($entree), $attendu);
            }
        )*
    };
}

cas_pair! {
    zero: 0 => true,
    un: 1 => false,
    negatif_impair: -3 => false,
    max: i32::MAX => false,
}
```

Chaque cas est un test séparé, avec son nom dans le rapport. Les crates `rstest` et `test-case` proposent la même chose avec un attribut (`#[case(0, true)]`).

### 10.3 — Tester les deux côtés d'une condition

Pour chaque condition du code, il faut au moins un cas où elle est vraie, un où elle est fausse, et la **limite** :

```rust
fn peut_voter(age: u32) -> bool { age >= 18 }

#[test]
fn vote_aux_limites() {
    assert!(!peut_voter(17));   // juste en dessous
    assert!(peut_voter(18));    // la limite exacte
    assert!(peut_voter(19));    // au-dessus
}
```

Un test qui ne vérifie que `peut_voter(30)` passerait aussi avec `age > 18` ou `age >= 0` : il ne protège pas la limite.

---

## 11. Tests asynchrones

### 11.1 — `#[tokio::test]`

```rust
async fn recuperer(id: u32) -> Option<String> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    (id > 0).then(|| format!("élément {id}"))
}

#[tokio::test]
async fn recupere_un_element() {
    assert_eq!(recuperer(1).await, Some("élément 1".into()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn avec_plusieurs_threads() {}
```

`#[test]` ne sait pas exécuter une fonction `async` : `#[tokio::test]` crée un runtime pour le test. Par défaut, il est mono-thread.

### 11.2 — Contrôler le temps

```rust
#[tokio::test(start_paused = true)]
async fn expiration() {
    let debut = tokio::time::Instant::now();
    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;   // instantané
    assert!(debut.elapsed() >= std::time::Duration::from_secs(3600));
}
```

Avec `start_paused` (feature `test-util` de tokio), le temps avance tout seul quand toutes les tâches attendent : un test d'expiration d'une heure dure quelques microsecondes.

---

## 12. Isoler les dépendances : base, temps, réseau

### 12.1 — Passer la dépendance en paramètre

```rust
// Difficile à tester : l'heure est lue à l'intérieur
fn est_expire_v1(expire_a: u64) -> bool {
    let maintenant = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    maintenant > expire_a
}

// Facile à tester : l'heure est un paramètre
fn est_expire(expire_a: u64, maintenant: u64) -> bool {
    maintenant > expire_a
}

#[test]
fn expiration_limite() {
    assert!(!est_expire(100, 100));
    assert!(est_expire(100, 101));
}
```

### 12.2 — Un trait pour remplacer un service

```rust
trait Envoi {
    fn envoyer(&self, a: &str, texte: &str) -> Result<(), String>;
}

struct Inscription<E: Envoi> { mail: E }

impl<E: Envoi> Inscription<E> {
    fn inscrire(&self, email: &str) -> Result<(), String> {
        self.mail.envoyer(email, "Bienvenue")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FauxEnvoi { envoyes: RefCell<Vec<String>> }

    impl Envoi for FauxEnvoi {
        fn envoyer(&self, a: &str, _texte: &str) -> Result<(), String> {
            self.envoyes.borrow_mut().push(a.to_string());
            Ok(())
        }
    }

    #[test]
    fn envoie_le_mail_de_bienvenue() {
        let i = Inscription { mail: FauxEnvoi { envoyes: RefCell::new(vec![]) } };
        i.inscrire("ada@ex.com").unwrap();
        assert_eq!(*i.mail.envoyes.borrow(), ["ada@ex.com"]);
    }
}
```

Le code de production utilise un vrai envoi SMTP ; le test, un faux qui enregistre les appels. La crate `mockall` génère ces faux automatiquement.

### 12.3 — Base de données : une transaction par test

Contre une vraie base, chaque test doit partir d'un état connu. La technique la plus sûre : ouvrir une transaction au début du test et l'**annuler** à la fin, quoi qu'il arrive. Aucun test ne voit les écritures d'un autre, même en parallèle.

Dans Runique, c'est ce que fournit la feature `test-utils` (à déclarer en `[dev-dependencies]`) avec `runique_test` : chaque test reçoit une connexion dans une transaction annulée à la fin.

Alternatives :

- base SQLite en mémoire, recréée par test (rapide, mais pas le même moteur qu'en production) ;
- une base par test (isolation totale, plus lent) ;
- tests en série (`serial_test`, ou `--test-threads=1`) si l'isolation est impossible.

### 12.4 — Tests qui partagent un état global

Variables d'environnement, fichiers fixes, port réseau : les tests parallèles se marchent dessus.

```rust
#[test]
#[serial_test::serial]          // ces tests ne tournent jamais en même temps
fn modifie_l_environnement() {
    unsafe { std::env::set_var("MODE", "test"); }
}
```

Depuis l'édition 2024, `set_var` est `unsafe` : modifier l'environnement pendant que d'autres threads le lisent est un comportement indéfini. Préfère passer la configuration en paramètre.

---

## 13. Tests et features

```rust
#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "admin")]          // compilé seulement avec la feature
    fn admin_accessible() {}

    #[test]
    #[cfg(not(feature = "admin"))]
    fn admin_absent() {}
}
```

```bash
cargo test --features admin
cargo test --no-default-features
```

Un test sous `#[cfg(feature = "x")]` disparaît **silencieusement** sans la feature : `cargo test` affiche « 0 tests » pour ce module, sans erreur. Avec des features exclusives, il faut une commande par combinaison (voir `cargo-dependances`).

---

## 14. Commandes `cargo test`

```bash
cargo test                          # tous les tests : unitaires, intégration, doc
cargo test moyenne                  # tests dont le chemin contient "moyenne"
cargo test calcul::tests::          # un module
cargo test moyenne -- --exact       # nom exact (chemin complet)

cargo test --lib                    # seulement les tests unitaires de la bibliothèque
cargo test --test api               # seulement tests/api.rs
cargo test --doc                    # seulement les exemples de documentation
cargo test --bins                   # seulement les tests des binaires

cargo test -- --nocapture           # afficher les println! (cachés quand le test réussit)
cargo test -- --test-threads=1      # un test à la fois
cargo test -- --ignored             # seulement les #[ignore]
cargo test -- --include-ignored
cargo test -- --list                # lister sans exécuter
cargo test --no-fail-fast           # continuer après le premier fichier en échec

cargo test -p runique               # un membre du workspace
cargo test --features admin,test-utils nom_du_test
cargo test --release                # tests avec optimisations (dépassements d'entier non détectés)
```

Les options **avant** `--` sont pour Cargo, celles **après** pour le programme de test.

---

## 15. Vérifier la qualité des tests

### 15.1 — Couverture

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html             # rapport ligne par ligne
cargo llvm-cov --features admin
```

La couverture dit quelles lignes ont été **exécutées**, pas si elles ont été **vérifiées** : un test sans assertion couvre du code sans rien prouver.

### 15.2 — Tests de mutation

```bash
cargo install cargo-mutants
cargo mutants                     # modifie le code (>= devient >, true devient false...) et relance les tests
cargo mutants -f src/auth.rs      # un seul fichier
```

Chaque mutation qui **survit** (les tests passent toujours) montre une ligne dont le comportement n'est vérifié par aucun test. C'est le meilleur moyen de trouver les limites oubliées (section 10.3).

### 15.3 — Tests de propriétés

```rust
// avec la crate proptest
proptest::proptest! {
    #[test]
    fn inverser_deux_fois_redonne_l_original(v: Vec<i32>) {
        let mut w = v.clone();
        w.reverse();
        w.reverse();
        proptest::prop_assert_eq!(v, w);
    }
}
```

Au lieu d'écrire des cas à la main, on écrit une propriété vraie pour **toutes** les entrées ; la crate génère des centaines d'entrées et réduit tout contre-exemple au plus petit.

---

## 16. Bonnes pratiques

**Nommer le comportement :**

```rust
#[test] fn test1() {}                                    // vague
#[test] fn diviser_par_zero_renvoie_une_erreur() {}      // dit ce qui est garanti
```

**Arranger, agir, vérifier :**

```rust
#[test]
fn ajout_au_panier() {
    // Arranger
    let mut panier = Vec::new();
    // Agir
    panier.push(("pomme", 2));
    // Vérifier
    assert_eq!(panier.len(), 1);
}
```

**Un test, un comportement** : si le nom contient « et », c'est souvent deux tests.

**Tester ce qui doit être refusé** autant que ce qui doit marcher, surtout pour la sécurité : mot de passe faux, utilisateur sans droits, jeton expiré, entrée trop longue.

**Des tests déterministes** : pas d'heure réelle, pas d'aléatoire non fixé, pas d'ordre de `HashMap`, pas de dépendance à l'ordre d'exécution des tests.

---

## 17. Exercices

### Exercice 1 — validateur de mot de passe

Teste `valide(mdp: &str) -> bool` (au moins 8 caractères et un chiffre) des deux côtés de chaque condition.

```rust
// Solution :
fn valide(mdp: &str) -> bool {
    mdp.chars().count() >= 8 && mdp.chars().any(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longueur_limite() {
        assert!(!valide("abcdef1"));      // 7 caractères
        assert!(valide("abcdefg1"));      // 8 caractères
    }

    #[test]
    fn chiffre_obligatoire() {
        assert!(!valide("abcdefgh"));
        assert!(valide("abcdefg9"));
    }

    #[test]
    fn longueur_en_caracteres_pas_en_octets() {
        assert!(!valide("ééééé1"));       // 6 caractères mais 11 octets
    }
}
```

### Exercice 2 — erreur précise

`parse_port(t: &str) -> Result<u16, String>` refuse le texte non numérique et le port 0. Vérifie le message de chaque erreur.

```rust
// Solution :
fn parse_port(t: &str) -> Result<u16, String> {
    let p: u16 = t.parse().map_err(|_| "port non numérique".to_string())?;
    if p == 0 {
        return Err("port 0 interdit".into());
    }
    Ok(p)
}

#[test]
fn erreurs_de_port() {
    assert_eq!(parse_port("abc").unwrap_err(), "port non numérique");
    assert_eq!(parse_port("0").unwrap_err(), "port 0 interdit");
    assert_eq!(parse_port("70000").unwrap_err(), "port non numérique");   // dépasse u16
    assert_eq!(parse_port("1"), Ok(1));
}
```

### Exercice 3 — horloge injectée

Rends testable une fonction qui dit si une session de 30 minutes est expirée.

```rust
// Solution :
fn session_expiree(creee_a: u64, maintenant: u64) -> bool {
    maintenant.saturating_sub(creee_a) > 30 * 60
}

#[test]
fn expiration_session() {
    assert!(!session_expiree(0, 1800));
    assert!(session_expiree(0, 1801));
    assert!(!session_expiree(100, 50));    // horloge en arrière : pas de dépassement
}
```

---

## 18. Aide-mémoire

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn a() { assert!(c); assert_eq!(x, y); assert_ne!(x, z); }
    #[test] fn b() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    #[test] #[should_panic(expected = "message")] fn c() {}
    #[test] #[ignore = "raison"] fn d() {}
    #[tokio::test] async fn e() {}
}

assert!(matches!(r, Err(Erreur::X)));
assert_eq!(f().unwrap_err(), "message");
```

```bash
cargo test filtre                 cargo test -- --nocapture
cargo test --lib / --test nom / --doc
cargo test -- --test-threads=1    cargo test -- --ignored
cargo test --features a,b         cargo llvm-cov    cargo mutants
```

### Points clés

- Un test échoue en paniquant ; `assert_eq!` montre les valeurs.
- `#[should_panic]` toujours avec `expected`.
- Vérifie le contenu d'une erreur, pas seulement `is_err()`.
- Unitaires dans `mod tests` (voient le privé), intégration dans `tests/` (API publique).
- Les exemples de documentation sont exécutés : `no_run`, `compile_fail`, `should_panic`.
- `Drop` garantit le nettoyage, même quand le test échoue.
- Teste chaque condition des deux côtés, limite comprise.
- Injecte l'heure, le réseau et la base pour rendre le code testable.
- Un test sous `#[cfg(feature)]` disparaît sans bruit sans la feature.
- La couverture mesure l'exécution, les mutants mesurent la vérification.
