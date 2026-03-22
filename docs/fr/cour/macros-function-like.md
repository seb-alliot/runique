# Macros procédurales — Function-like
> Des macros avec la puissance des proc-macros mais la syntaxe de macro_rules!

## Objectifs

- Comprendre quand utiliser une function-like proc macro plutôt que `macro_rules!`
- Écrire une function-like macro avec parsing custom
- Créer un DSL avec validation à la compilation

---

## Table des matières

1. [macro_rules! vs function-like proc macro](#1-macro_rules-vs-function-like-proc-macro)
2. [Signature d'une function-like macro](#2-signature-dune-function-like-macro)
3. [Exemple — SQL avec validation à la compilation](#3-exemple--sql-avec-validation-à-la-compilation)
4. [Exemple — DSL de configuration](#4-exemple--dsl-de-configuration)
5. [Parser des structures complexes](#5-parser-des-structures-complexes)
6. [Comparaison des 3 types de proc macros](#6-comparaison-des-3-types-de-proc-macros)

---

## 1. macro_rules! vs function-like proc macro

Les deux s'appellent avec `ma_macro!(...)`. La différence est dans ce qu'on peut faire à l'intérieur.

| | `macro_rules!` | Function-like proc macro |
|---|---|---|
| Parsing | Pattern matching limité | Parsing Rust arbitraire via `syn` |
| Erreurs | Messages basiques | Messages précis avec `span` |
| Logique | Patterns déclaratifs | Code Rust complet |
| Validation | Limitée | Validation complète à la compilation |
| Complexité | Simple | Nécessite crate proc-macro |

Utilise `macro_rules!` tant que tu peux. Passe aux function-like proc macros quand le parsing devient trop complexe ou quand tu veux des erreurs de compilation précises.

---

## 2. Signature d'une function-like macro

```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro]
pub fn ma_macro(input: TokenStream) -> TokenStream {
    // input = tout ce qui est entre les parenthèses de ma_macro!(...)
    // retour = le code qui remplace l'appel entier
    input
}
```

---

## 3. Exemple — SQL avec validation à la compilation

Valider qu'une requête SQL commence par SELECT/INSERT/UPDATE **au moment de la compilation**, pas à l'exécution.

```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let sql_str = parse_macro_input!(input as LitStr);
    let sql = sql_str.value();

    let upper = sql.trim().to_uppercase();
    let valide = upper.starts_with("SELECT")
        || upper.starts_with("INSERT")
        || upper.starts_with("UPDATE")
        || upper.starts_with("DELETE");

    if !valide {
        // Erreur de compilation avec position précise
        return syn::Error::new(
            sql_str.span(),
            format!("Requête SQL invalide : doit commencer par SELECT/INSERT/UPDATE/DELETE")
        )
        .to_compile_error()
        .into();
    }

    quote! { #sql_str }.into()
}
```

```rust
// utilisation
use ma_lib_derive::sql;

fn main() {
    let q1 = sql!("SELECT * FROM users WHERE age > 18"); // ✅ OK

    let q2 = sql!("DROP TABLE users"); // ❌ Erreur de compilation
    // error: Requête SQL invalide : doit commencer par SELECT/INSERT/UPDATE/DELETE
}
```

Le `DROP TABLE` est refusé **avant même que le programme compile**. C'est impossible à faire avec `macro_rules!`.

---

## 4. Exemple — DSL de configuration

```rust
use std::collections::HashMap;

// Avec macro_rules! — fonctionne mais limité
macro_rules! config {
    ($($section:ident { $($key:ident = $value:expr),* $(,)? })*) => {{
        let mut cfg: HashMap<&str, HashMap<&str, String>> = HashMap::new();
        $(
            let mut section = HashMap::new();
            $(
                section.insert(stringify!($key), $value.to_string());
            )*
            cfg.insert(stringify!($section), section);
        )*
        cfg
    }};
}

fn main() {
    let cfg = config! {
        database {
            host = "localhost",
            port = 5432,
        }
        server {
            host = "0.0.0.0",
            port = 8080,
        }
    };

    println!("{:?}", cfg["database"]["host"]); // "localhost"
}
```

Avec une function-like proc macro, on pourrait ajouter : validation des types, erreurs précises si une clé est manquante, génération d'une struct typée plutôt qu'un HashMap.

---

## 5. Parser des structures complexes

Pour un DSL non-standard, on implémente le trait `Parse` de `syn` :

```rust
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, LitStr, braced};

// Représente : section_name { key = "value", ... }
struct Section {
    name: Ident,
    entries: Vec<(Ident, LitStr)>,
}

impl Parse for Section {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        let content;
        braced!(content in input); // Consomme les { }

        let mut entries = vec![];
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            let _: Token![=] = content.parse()?;
            let val: LitStr = content.parse()?;
            let _: Option<Token![,]> = content.parse()?; // virgule optionnelle
            entries.push((key, val));
        }

        Ok(Section { name, entries })
    }
}

#[proc_macro]
pub fn config_typed(input: TokenStream) -> TokenStream {
    // Parser toutes les sections
    let sections = syn::parse::Parser::parse(
        |s: ParseStream| {
            let mut sections = vec![];
            while !s.is_empty() {
                sections.push(s.parse::<Section>()?);
            }
            Ok(sections)
        },
        input,
    ).unwrap();

    // Générer une struct par section
    let structs = sections.iter().map(|s| {
        let name = &s.name;
        let fields = s.entries.iter().map(|(k, v)| {
            quote::quote! { pub #k: &'static str }
        });
        let values = s.entries.iter().map(|(k, v)| {
            quote::quote! { #k: #v }
        });
        quote::quote! {
            pub struct #name { #(#fields,)* }
            pub static CONFIG_#name: #name = #name { #(#values,)* };
        }
    });

    quote::quote! { #(#structs)* }.into()
}
```

---

## 6. Comparaison des 3 types de proc macros

```
#[derive(MaTrait)]    → Derive macro
struct Foo { ... }      → Ajoute du code, ne modifie pas la struct

#[mon_attribut]       → Attribute macro
fn ma_fn() { ... }      → Remplace l'item par ce que la macro retourne

ma_macro!(...)        → Function-like proc macro
                        → Remplace l'appel par ce que la macro retourne
```

En pratique dans l'écosystème :

| Cas d'usage | Type |
|---|---|
| Implémenter un trait automatiquement | Derive |
| Décorer une fonction (log, auth, retry) | Attribute |
| DSL avec parsing custom | Function-like |
| Validation à la compilation | Function-like |
| Génération de code depuis données externes | Function-like |

Dans Runique, `derive_form` est une **derive macro** — elle génère les méthodes de formulaire depuis la définition de struct. `admin!`, `model!`, `view!` sont des **`macro_rules!`** classiques — assez puissantes pour leur usage sans nécessiter une proc-macro.
