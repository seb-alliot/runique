# Macros procédurales — Attribute macros
> Transformer n'importe quel item Rust avec #[mon_attribut]

## Objectifs

- Comprendre la différence entre derive et attribute macros
- Écrire une attribute macro qui transforme une fonction
- Gérer les arguments de l'attribut
- Cas d'usage concrets

---

## Table des matières

1. [Derive vs Attribute — la différence](#1-derive-vs-attribute--la-différence)
2. [Signature d'une attribute macro](#2-signature-dune-attribute-macro)
3. [Exemple — mesure du temps d'exécution](#3-exemple--mesure-du-temps-dexécution)
4. [Lire les arguments de l'attribut](#4-lire-les-arguments-de-lattribut)
5. [Transformer une struct](#5-transformer-une-struct)
6. [Cas d'usage réels](#6-cas-dusage-réels)

---

## 1. Derive vs Attribute — la différence

| | Derive macro | Attribute macro |
|---|---|---|
| Syntaxe | `#[derive(Trait)]` | `#[mon_attribut]` |
| Ce qu'elle reçoit | La struct/enum, inchangée | L'item entier |
| Ce qu'elle retourne | Du code **en plus** | Le remplacement complet |
| S'applique à | Struct, enum, union | Tout item (fn, struct, impl, mod...) |

Une derive macro **ajoute** du code. Une attribute macro **remplace** l'item par ce qu'elle retourne — elle peut modifier, décorer, ou entièrement transformer.

---

## 2. Signature d'une attribute macro

```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn mon_attribut(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr = les arguments entre parenthèses : #[mon_attribut(arg1, arg2)]
    // item = le code sur lequel l'attribut est posé (la fonction, struct, etc.)
    // retour = le code qui remplace l'item
    item // Retourner item sans modification = attribut no-op
}
```

---

## 3. Exemple — mesure du temps d'exécution

```rust
// ma_lib_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn timed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let name    = &input.sig.ident;
    let vis     = &input.vis;
    let sig     = &input.sig;
    let block   = &input.block;

    let expanded = quote! {
        #vis #sig {
            let __start = std::time::Instant::now();
            let __result = (|| #block)();
            println!(
                "[timed] '{}' : {:?}",
                stringify!(#name),
                __start.elapsed()
            );
            __result
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// utilisation
use ma_lib_derive::timed;

#[timed]
fn calcul(n: u64) -> u64 {
    (0..n).sum()
}

fn main() {
    let r = calcul(1_000_000);
    // Affiche : [timed] 'calcul' : 1.2ms
    println!("résultat: {}", r);
}
```

La fonction `calcul` est remplacée par une version identique mais entourée d'un chronomètre. L'appelant n'y voit rien.

---

## 4. Lire les arguments de l'attribut

```rust
#[timed(prefix = "MON_APP")]
fn calcul(n: u64) -> u64 { ... }
```

Pour parser les arguments, on utilise `syn::parse` :

```rust
use syn::{parse_macro_input, LitStr, Token};
use syn::parse::{Parse, ParseStream};

struct TimedArgs {
    prefix: Option<String>,
}

impl Parse for TimedArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(TimedArgs { prefix: None });
        }
        // Attend : prefix = "valeur"
        let _: syn::Ident = input.parse()?; // "prefix"
        let _: Token![=] = input.parse()?;
        let val: LitStr = input.parse()?;
        Ok(TimedArgs { prefix: Some(val.value()) })
    }
}

#[proc_macro_attribute]
pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TimedArgs);
    let prefix = args.prefix.unwrap_or_else(|| "timed".to_string());

    let input = parse_macro_input!(item as ItemFn);
    let name  = &input.sig.ident;
    let sig   = &input.sig;
    let vis   = &input.vis;
    let block = &input.block;

    let expanded = quote! {
        #vis #sig {
            let __start = std::time::Instant::now();
            let __result = (|| #block)();
            println!("[{}] '{}' : {:?}", #prefix, stringify!(#name), __start.elapsed());
            __result
        }
    };

    TokenStream::from(expanded)
}
```

---

## 5. Transformer une struct

Une attribute macro peut aussi s'appliquer à une struct et la modifier :

```rust
#[proc_macro_attribute]
pub fn avec_id(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::ItemStruct);

    // Ajoute un champ `id: u32` à la struct
    if let syn::Fields::Named(ref mut fields) = input.fields {
        let id_field: syn::Field = syn::parse_quote! {
            pub id: u32
        };
        fields.named.push(id_field);
    }

    quote! { #input }.into()
}
```

```rust
#[avec_id]
struct Article {
    titre: String,
    contenu: String,
}

// Après expansion, la struct a 3 champs : id, titre, contenu
fn main() {
    let a = Article { id: 1, titre: "Hello".to_string(), contenu: "...".to_string() };
}
```

---

## 6. Cas d'usage réels

Les attribute macros sont utilisées partout dans l'écosystème Rust :

**Axum — définir des handlers HTTP**
```rust
// Simplifié — Axum utilise des extracteurs, pas d'attribute macro
// Mais des frameworks comme Actix-web utilisent ce pattern
#[get("/users")]
async fn list_users() -> impl Responder { ... }
```

**Tokio — transformer une fn en runtime async**
```rust
#[tokio::main]
async fn main() {
    // Tokio injecte le runtime autour de ce bloc
}
```

Ce qui se génère (simplifié) :
```rust
fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            // ton code ici
        });
}
```

**Serde — contrôle fin de la sérialisation**
```rust
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(rename = "host_name")]
    host: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}
```

`serde` et `rename` sont des attributs helper enregistrés par la derive macro Serialize/Deserialize.
