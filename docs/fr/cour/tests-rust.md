# Tests en Rust
> Tests unitaires, d'intégration, de documentation — écrire des tests fiables avec Cargo

## Objectifs

- Écrire des tests unitaires avec `#[test]`
- Organiser les tests avec `#[cfg(test)]`
- Écrire des tests d'intégration dans `tests/`
- Utiliser les tests de documentation
- Maîtriser les macros d'assertion

---

## Table des matières

1. [Tests unitaires](#1-tests-unitaires)
   - 1.1 [Syntaxe de base](#11-syntaxe-de-base)
   - 1.2 [Macros d'assertion](#12-macros-dasssertion)
   - 1.3 [Tests qui doivent paniquer](#13-tests-qui-doivent-paniquer)
   - 1.4 [Tests ignorés](#14-tests-ignorés)
2. [Organisation avec cfg(test)](#2-organisation-avec-cfgtest)
3. [Tests d'intégration](#3-tests-dintégration)
4. [Tests de documentation](#4-tests-de-documentation)
5. [Commandes cargo test](#5-commandes-cargo-test)
6. [Bonnes pratiques](#6-bonnes-pratiques)

---

## 1. Tests unitaires

### 1.1 Syntaxe de base

```rust
pub fn ajouter(a: i32, b: i32) -> i32 {
    a + b
}

pub fn est_pair(n: i32) -> bool {
    n % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;  // importe tout du module parent

    #[test]
    fn test_ajouter() {
        assert_eq!(ajouter(2, 3), 5);
    }

    #[test]
    fn test_est_pair() {
        assert!(est_pair(4));
        assert!(!est_pair(3));
    }
}
```

### 1.2 Macros d'assertion

```rust
// assert! — vérifie qu'une condition est vraie
assert!(2 + 2 == 4);
assert!(vec![1, 2, 3].len() == 3, "longueur incorrecte");

// assert_eq! — vérifie l'égalité (affiche les deux valeurs si échec)
assert_eq!(2 + 2, 4);
assert_eq!(
    "bonjour".to_uppercase(),
    "BONJOUR",
    "conversion en majuscules échouée"
);

// assert_ne! — vérifie l'inégalité
assert_ne!(2 + 2, 5);

// Pour les flottants — comparer avec une tolérance
let resultat = 0.1 + 0.2;
assert!((resultat - 0.3).abs() < 1e-10);
```

### 1.3 Tests qui doivent paniquer

```rust
pub fn diviser(a: i32, b: i32) -> i32 {
    if b == 0 { panic!("division par zéro"); }
    a / b
}

#[test]
#[should_panic]
fn test_division_par_zero() {
    diviser(10, 0);
}

// Vérifier le message de panique
#[test]
#[should_panic(expected = "division par zéro")]
fn test_message_panique() {
    diviser(10, 0);
}
```

### 1.4 Tests ignorés

```rust
#[test]
#[ignore = "trop lent pour le CI"]
fn test_lent() {
    // prend plusieurs minutes...
}
```

```bash
cargo test -- --ignored          # exécute seulement les tests ignorés
cargo test -- --include-ignored  # exécute tous les tests y compris ignorés
```

---

## 2. Organisation avec cfg(test)

```rust
// src/lib.rs

pub struct Calculatrice {
    valeur: f64,
}

impl Calculatrice {
    pub fn new(v: f64) -> Self { Calculatrice { valeur: v } }
    pub fn ajouter(&mut self, n: f64) -> &mut Self { self.valeur += n; self }
    pub fn resultat(&self) -> f64 { self.valeur }

    // Méthode privée — testable uniquement dans cfg(test)
    fn valeur_interne(&self) -> f64 { self.valeur }
}

#[cfg(test)]  // ce module n'est compilé que lors des tests
mod tests {
    use super::*;

    #[test]
    fn test_addition_chainee() {
        let mut calc = Calculatrice::new(0.0);
        calc.ajouter(5.0).ajouter(3.0);
        assert_eq!(calc.resultat(), 8.0);
    }

    #[test]
    fn test_valeur_interne() {
        let calc = Calculatrice::new(42.0);
        assert_eq!(calc.valeur_interne(), 42.0); // accès au privé OK dans tests
    }
}
```

---

## 3. Tests d'intégration

Les tests d'intégration testent l'API publique comme un utilisateur externe.

```
mon_projet/
├── src/
│   └── lib.rs
└── tests/              ← dossier des tests d'intégration
    ├── api_test.rs
    └── helpers/
        └── mod.rs
```

```rust
// tests/api_test.rs
use mon_projet::Calculatrice;  // uniquement l'API publique

#[test]
fn test_integration_calcul() {
    let mut calc = Calculatrice::new(10.0);
    calc.ajouter(5.0);
    assert_eq!(calc.resultat(), 15.0);
}
```

```rust
// tests/helpers/mod.rs — helpers partagés entre tests
pub fn creer_calculatrice_initialisee() -> mon_projet::Calculatrice {
    mon_projet::Calculatrice::new(100.0)
}
```

```rust
// tests/autre_test.rs
mod helpers;

#[test]
fn test_avec_helper() {
    let calc = helpers::creer_calculatrice_initialisee();
    assert_eq!(calc.resultat(), 100.0);
}
```

---

## 4. Tests de documentation

Les exemples dans la documentation sont **exécutés comme des tests**.

```rust
/// Additionne deux nombres.
///
/// # Exemples
///
/// ```
/// let resultat = mon_projet::ajouter(2, 3);
/// assert_eq!(resultat, 5);
/// ```
///
/// ```
/// // Un exemple qui doit paniquer
/// # use mon_projet::diviser;
/// let r = diviser(10, 2);
/// assert_eq!(r, 5);
/// ```
pub fn ajouter(a: i32, b: i32) -> i32 {
    a + b
}
```

```bash
cargo test --doc  # exécute uniquement les tests de documentation
```

**Syntaxe spéciale dans les doc-tests :**

```rust
/// ```
/// # // Les lignes préfixées par # sont exécutées mais pas affichées
/// # let x = 5;
/// println!("{x}"); // affiché dans la doc
/// # assert_eq!(x, 5);
/// ```
```

---

## 5. Commandes cargo test

```bash
cargo test                        # tous les tests
cargo test nom_du_test            # filtre par nom (sous-chaîne)
cargo test tests::                # tests dans le module 'tests'

cargo test -- --nocapture         # affiche println! dans les tests
cargo test -- --test-threads=1    # séquentiel (utile si tests partagent état)
cargo test -- --ignored           # tests marqués #[ignore]

cargo test --lib                  # tests unitaires uniquement (src/)
cargo test --test api_test        # un fichier d'intégration spécifique
cargo test --doc                  # tests de documentation uniquement

cargo test -p ma_crate            # dans un workspace, une crate spécifique
```

---

## 6. Bonnes pratiques

**Nommer les tests clairement :**
```rust
// ❌ Trop vague
#[test] fn test1() { }

// ✅ Nom descriptif
#[test] fn ajouter_deux_positifs_retourne_leur_somme() { }
#[test] fn diviser_par_zero_panique() { }
```

**Arranger / Agir / Vérifier (AAA) :**
```rust
#[test]
fn test_insertion_element() {
    // Arranger
    let mut liste = Vec::new();

    // Agir
    liste.push(42);

    // Vérifier
    assert_eq!(liste.len(), 1);
    assert_eq!(liste[0], 42);
}
```

**Tests paramétrés avec une boucle :**
```rust
#[test]
fn test_est_pair_plusieurs_valeurs() {
    let cas = [(0, true), (1, false), (2, true), (99, false), (100, true)];
    for (entree, attendu) in cas {
        assert_eq!(est_pair(entree), attendu, "est_pair({entree}) devrait être {attendu}");
    }
}
```
