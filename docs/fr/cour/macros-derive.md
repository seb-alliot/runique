# Macros procédurales — Derive
> Générer automatiquement des implémentations de trait avec #[derive(...)]

## Objectifs

- Comprendre la structure d'une crate proc-macro
- Parser le code source avec `syn`
- Générer du code avec `quote`
- Écrire une derive macro de A à Z

---

## Table des matières

1. [Qu'est-ce qu'une derive macro ?](#1-quest-ce-quune-derive-macro-)
2. [Structure d'une crate proc-macro](#2-structure-dune-crate-proc-macro)
3. [syn — parser le code](#3-syn--parser-le-code)
4. [quote — générer du code](#4-quote--générer-du-code)
5. [Exemple complet — Describe](#5-exemple-complet--describe)
6. [Accéder aux champs de la struct](#6-accéder-aux-champs-de-la-struct)
7. [Attributs helper sur les champs](#7-attributs-helper-sur-les-champs)

---

## 1. Qu'est-ce qu'une derive macro ?

Une derive macro s'utilise comme ça :

```rust
#[derive(Debug, Clone, MaMacro)]
struct Personne {
    nom: String,
    age: u32,
}
```

`Debug` et `Clone` sont des derives de la stdlib. `MaMacro` est une derive personnalisée. Elle reçoit la définition complète de `Personne` et génère du code Rust supplémentaire — généralement une implémentation de trait.

---

## 2. Structure d'une crate proc-macro

Les macros procédurales **doivent** être dans leur propre crate séparée.

```toml
# ma_lib_derive/Cargo.toml
[package]
name = "ma_lib_derive"
version = "0.1.0"

[lib]
proc-macro = true

[dependencies]
syn   = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```

```toml
# ma_lib/Cargo.toml
[dependencies]
ma_lib_derive = { path = "../ma_lib_derive" }
```

```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_derive(MaTrait)]
pub fn derive_ma_trait(input: TokenStream) -> TokenStream {
    // input = la struct/enum sur laquelle #[derive(MaTrait)] est appliqué
    // retour = code Rust à ajouter
    todo!()
}
```

---

## 3. syn — parser le code

`syn` transforme les tokens bruts en arbre syntaxique utilisable.

```rust
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MaTrait)]
pub fn derive_ma_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let nom_struct = &input.ident;       // Identifiant : Personne, Config, etc.
    let generics   = &input.generics;    // Les paramètres génériques <T, U>
    let data       = &input.data;        // Contenu : struct, enum, union

    todo!()
}
```

Les champs utiles de `DeriveInput` :

```rust
input.ident      // nom de la struct/enum
input.generics   // <T: Clone, U>
input.attrs      // attributs #[...] posés sur la struct
input.data       // Data::Struct, Data::Enum, Data::Union
```

---

## 4. quote — générer du code

`quote!` produit des tokens Rust à partir d'un template. Les variables sont interpolées avec `#`.

```rust
use quote::quote;

let nom = &input.ident;

let expanded = quote! {
    impl #nom {
        pub fn hello(&self) -> String {
            format!("Je suis une instance de {}", stringify!(#nom))
        }
    }
};

TokenStream::from(expanded)
```

Règles d'interpolation :

```rust
let name: &Ident = ...;
let ty: &Type = ...;
let items: &[...] = ...;

quote! {
    #name          // Interpolation simple
    #ty            // Un type
    #(#items),*   // Répétition (comme $(...),* en macro_rules!)
}
```

---

## 5. Exemple complet — Describe

Une derive macro qui génère une méthode `describe()` sur n'importe quelle struct.

```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn describe(&self) -> String {
                format!("Instance de {}", stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// utilisation
use ma_lib_derive::Describe;

#[derive(Describe)]
struct Config {
    host: String,
    port: u16,
}

fn main() {
    let c = Config { host: "localhost".to_string(), port: 8080 };
    println!("{}", c.describe()); // "Instance de Config"
}
```

---

## 6. Accéder aux champs de la struct

Pour itérer sur les champs et générer du code par champ :

```rust
use syn::{Data, Fields};

#[proc_macro_derive(ListeChamps)]
pub fn derive_liste_champs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extraire les champs nommés
    let champs = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ListeChamps ne supporte que les structs avec champs nommés"),
        },
        _ => panic!("ListeChamps ne supporte que les structs"),
    };

    // Noms des champs comme strings
    let noms_champs: Vec<_> = champs
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();

    let expanded = quote! {
        impl #name {
            pub fn champs() -> &'static [&'static str] {
                &[ #(#noms_champs),* ]
            }
        }
    };

    TokenStream::from(expanded)
}
```

```rust
#[derive(ListeChamps)]
struct Personne {
    nom: String,
    age: u32,
    email: String,
}

fn main() {
    println!("{:?}", Personne::champs()); // ["nom", "age", "email"]
}
```

---

## 7. Attributs helper sur les champs

On peut définir des attributs personnalisés à placer sur les champs :

```rust
#[proc_macro_derive(Validable, attributes(valider))]
pub fn derive_validable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let champs = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(f) => &f.named,
            _ => panic!(),
        },
        _ => panic!(),
    };

    // Regarde si le champ a #[valider(min_len = 3)]
    for champ in champs {
        for attr in &champ.attrs {
            if attr.path().is_ident("valider") {
                // Parser les arguments de l'attribut
            }
        }
    }

    // Génère la validation...
    todo!()
}
```

```rust
#[derive(Validable)]
struct RegisterForm {
    #[valider(min_len = 3, max_len = 50)]
    username: String,
    #[valider(email)]
    email: String,
    #[valider(min_len = 8)]
    password: String,
}
```

C'est exactement comme ça que `derive_form` fonctionne dans Runique pour générer la validation des formulaires.
