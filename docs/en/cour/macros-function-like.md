# Procedural macros — Function-like
> `my_macro!(...)` written in Rust: invent a syntax, parse it with `syn`, validate it at compile time and generate code. With, as a real example, Runique's `model!{}` DSL.

## Objectives

- Know when a procedural function-like macro replaces `macro_rules!`
- Write the signature and the entry point
- Design a syntax and parse it with `Parse`: keywords, punctuation, groups, lists, choices
- Validate values at compile time (text, numbers, names)
- Generate structs, modules and constants
- Share the parser between a macro and another tool (CLI)
- Compare the three kinds of procedural macros

## Table of contents

1. `macro_rules!` or procedural macro
2. Signature
3. Designing a syntax
4. Parsing with `Parse`
5. Example: validating a query at compile time
6. Example: a configuration DSL
7. Parsing complex structures
8. Generating code
9. What a macro must not do
10. Example in Runique: `model!{}` and `extend!{}`
11. Comparing the three kinds of procedural macros
12. Exercises
13. Cheat sheet

---

## 1. `macro_rules!` or procedural macro

| Need | `macro_rules!` | Procedural function-like |
|---|---|---|
| Regular syntax (lists, pairs) | yes | yes |
| Build identifiers (`get_` + name) | no | yes |
| Compute at compile time (validate a text, convert a case) | no | yes |
| Precise, positioned error messages | limited (`compile_error!`) | yes |
| Read a very free syntax | possible but painful (tt muncher) | natural |
| Cost | none | `proc-macro` crate, `syn`, `quote` |

Runique's `urlpatterns!` is a `macro_rules!`: its syntax is regular. `model!{}` is procedural: it converts names (`cour_id` → `CourId`), validates types and constraints, and generates whole modules.

---

## 2. Signature

```rust
use proc_macro::TokenStream;

#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // input: everything between the delimiters of my_macro!( ... ) / [ ... ] / { ... }
    input
}
```

```rust
my_macro!(a + b);
my_macro! { config { port: 8080 } }
```

As with `macro_rules!`, the call can use `()`, `[]` or `{}`; the delimiters are not passed on.

---

## 3. Designing a syntax

Before writing the parser, write **examples** of use and check three rules:

1. **It must be a sequence of valid Rust tokens**: identifiers, literals, punctuation, well-balanced groups. `my_macro!(a @@ b)` is accepted; a lone apostrophe (`'`) or an unclosed quote is not.
2. **Look like Rust** when possible: `name: type [options]` reads better than a syntax invented from scratch.
3. **Stay unambiguous**: each element must be recognisable by looking one or two tokens ahead.

```rust
// Target syntax example
config! {
    server {
        port: 8080,
        host: "0.0.0.0",
    }
    database {
        url: "postgres://localhost/app",
        connections: 10,
    }
}
```

---

## 4. Parsing with `Parse`

### 4.1 The principle

```rust
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Token};

// port: 8080
struct Setting {
    name: Ident,
    value: LitInt,
}

impl Parse for Setting {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;       // consumes an identifier
        input.parse::<Token![:]>()?;            // consumes ':'
        let value: LitInt = input.parse()?;     // consumes an integer
        Ok(Setting { name, value })
    }
}
```

Each `input.parse()` **consumes** tokens and returns a positioned error if it is not what was expected.

### 4.2 The tools

| Tool | Role |
|---|---|
| `input.parse::<T>()` | consume a `T` (`Ident`, `LitStr`, `Type`, `Expr`, `Path`...) |
| `Token![:]`, `Token![=>]`, `Token![,]` | Rust punctuation and keywords (`Token![fn]`, `Token![pub]`) |
| `syn::custom_keyword!(table)` | declare a keyword of your own |
| `input.peek(T)` | look at the next token without consuming it |
| `input.lookahead1()` | look, and produce an "expected A, B or C" error |
| `braced!`, `bracketed!`, `parenthesized!` | enter a `{}`, `[]`, `()` group |
| `Punctuated::<T, Token![,]>::parse_terminated` | a separated list, trailing comma allowed |
| `input.is_empty()` | end of the input (or of the group) |
| `input.error("...")` | error at the current position |

### 4.3 Your own keywords

```rust
mod kw {
    syn::custom_keyword!(table);
    syn::custom_keyword!(fields);
}

// table: "users"
struct Table { name: syn::LitStr }

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::table>()?;           // the word `table`, and nothing else
        input.parse::<Token![:]>()?;
        Ok(Table { name: input.parse()? })
    }
}
```

### 4.4 Choosing between several forms

```rust
use syn::{LitBool, LitInt, LitStr};

enum Value {
    Text(LitStr),
    Integer(LitInt),
    Boolean(LitBool),
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let l = input.lookahead1();
        if l.peek(LitStr) {
            Ok(Value::Text(input.parse()?))
        } else if l.peek(LitInt) {
            Ok(Value::Integer(input.parse()?))
        } else if l.peek(LitBool) {
            Ok(Value::Boolean(input.parse()?))
        } else {
            Err(l.error())                     // "expected string literal, integer literal or boolean literal"
        }
    }
}
```

`lookahead1` remembers everything that was tested and builds the error message automatically.

### 4.5 Groups and lists

```rust
use syn::{braced, punctuated::Punctuated};

// server { port: 8080, host: "0.0.0.0", }
struct Section {
    name: Ident,
    settings: Vec<(Ident, Value)>,
}

impl Parse for Section {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);             // everything between { } becomes a new stream
        let list = Punctuated::<Pair, Token![,]>::parse_terminated(&content)?;
        Ok(Section { name, settings: list.into_iter().map(|p| (p.name, p.value)).collect() })
    }
}

struct Pair { name: Ident, value: Value }

impl Parse for Pair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(Pair { name, value: input.parse()? })
    }
}
```

### 4.6 Repeating until the end

```rust
struct Config { sections: Vec<Section> }

impl Parse for Config {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sections = Vec::new();
        while !input.is_empty() {
            sections.push(input.parse()?);
        }
        Ok(Config { sections })
    }
}
```

---

## 5. Example: validating a query at compile time

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    let text = parse_macro_input!(input as LitStr);
    let sql = text.value();
    let uppercase = sql.trim_start().to_uppercase();

    if !uppercase.starts_with("SELECT") {
        return syn::Error::new(text.span(), "query! only accepts SELECT")
            .to_compile_error()
            .into();
    }
    if sql.contains(';') {
        return syn::Error::new(text.span(), "a single statement, without `;`")
            .to_compile_error()
            .into();
    }

    quote!(#text).into()                      // the text, now checked
}
```

```rust
let q = query!("SELECT id, email FROM users WHERE id = $1");
// let r = query!("DELETE FROM users");   // compile-time ERROR, underlined on the text
```

The check happens **only once**, at compile time: no runtime cost. This is the principle of `sqlx::query!`, which goes further by querying a real database during compilation to check the columns and types.

Warning: such a macro validates the **shape** of the query, not its safety. Values coming from the user always go through parameters (`$1`), never through an assembled string.

---

## 6. Example: a configuration DSL

With the parsers from section 4, the `config!` macro generates one struct per section and a constructor:

```rust
#[proc_macro]
pub fn config(input: TokenStream) -> TokenStream {
    let cfg = parse_macro_input!(input as Config);

    let structs = cfg.sections.iter().map(|s| {
        let struct_name = quote::format_ident!("{}", capitalize(&s.name.to_string()));
        let fields = s.settings.iter().map(|(n, v)| {
            let ty = match v {
                Value::Text(_) => quote!(&'static str),
                Value::Integer(_) => quote!(i64),
                Value::Boolean(_) => quote!(bool),
            };
            quote!(pub #n: #ty)
        });
        let values = s.settings.iter().map(|(n, v)| {
            let lit = match v {
                Value::Text(l) => quote!(#l),
                Value::Integer(l) => quote!(#l),
                Value::Boolean(l) => quote!(#l),
            };
            quote!(#n: #lit)
        });
        quote! {
            #[derive(Debug, Clone)]
            pub struct #struct_name { #(#fields),* }
            impl #struct_name {
                pub const fn default_values() -> Self { Self { #(#values),* } }
            }
        }
    });

    quote!(#(#structs)*).into()
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}
```

```rust
config! {
    server { port: 8080, host: "0.0.0.0" }
    database { connections: 10, ssl: true }
}

const SERVER: Server = Server::default_values();     // Server { port: 8080, host: "0.0.0.0" }
```

A struct name built from a section name (`server` → `Server`): impossible with `macro_rules!`.

---

## 7. Parsing complex structures

### 7.1 Optional elements

```rust
// name: type [option1, option2]     the brackets are optional
struct Field {
    name: Ident,
    ty: Ident,
    options: Vec<Ident>,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let options = if input.peek(syn::token::Bracket) {        // brackets present?
            let content;
            syn::bracketed!(content in input);
            Punctuated::<Ident, Token![,]>::parse_terminated(&content)?.into_iter().collect()
        } else {
            Vec::new()
        };
        Ok(Field { name, ty, options })
    }
}
```

### 7.2 Keys in any order, each one only once

```rust
struct Meta { table: Option<syn::LitStr>, order: Option<syn::LitStr> }

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut m = Meta { table: None, order: None };
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let value: syn::LitStr = input.parse()?;
            let slot = match key.to_string().as_str() {
                "table" => &mut m.table,
                "order" => &mut m.order,
                _ => return Err(syn::Error::new(key.span(), "unknown key: `table` or `order`")),
            };
            if slot.replace(value).is_some() {
                return Err(syn::Error::new(key.span(), format!("`{key}` is already defined")));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(m)
    }
}
```

`Option::replace` returns the old value: if there was one, the key is duplicated.

### 7.3 Reusing the `syn` parsers

There is no need to rewrite the parsing of a type or an expression: `input.parse::<syn::Type>()`, `input.parse::<syn::Expr>()` and `input.parse::<syn::Path>()` accept the whole corresponding Rust syntax, generics included.

### 7.4 Testing the parser without a macro

```rust
#[test]
fn rejects_a_duplicate_key() {
    let r: syn::Result<Meta> = syn::parse_str(r#"table: "a", table: "b""#);
    assert!(r.is_err());
}
```

`syn::parse_str` and `syn::parse2` let you test `Parse` like ordinary code, if the parser lives in a normal crate (section 10).

---

## 8. Generating code

### 8.1 Several items

A function-like macro can produce **anything** valid at the call site: an expression, statements, or a sequence of items (structs, `impl`, modules, constants).

```rust
quote! {
    pub mod user {
        pub struct Model { /* ... */ }
        pub enum Column { Id, Email }
        pub const TABLE: &str = "users";
        impl Model { /* ... */ }
    }
}
```

### 8.2 Expression or items

```rust
let x = my_macro!(...);       // must produce an expression
my_macro! { ... }             // in item position: can produce declarations
```

A macro designed for both uses only works in one of them: decide which, and document it.

### 8.3 Spans

By default, generated code takes the position of the call. For an error in a specific piece to underline the user's code, use `quote_spanned!` with the span of the relevant token (see `macros-attribut`, section 7).

---

## 9. What a macro must not do

### 9.1 Depend on files or the environment without saying so

A macro can read a file or an environment variable during compilation, but:

- Cargo does not know that the macro depends on this file: if it changes, the crate is **not recompiled**;
- the result depends on the machine that compiles (CI, developer machine, server);
- `cargo check` and rust-analyzer run the macro constantly: a network or database access on every keystroke slows everything down.

```rust
// What the macro receives is the only reliable, reproducible input
// let content = std::fs::read_to_string("schema.txt");   // avoid
```

If it is unavoidable, prefer generating `include_str!("path")` in the produced code: the compiler then reads the file, and it tracks its changes.

### 9.2 Panic

Always a positioned `syn::Error`, never `panic!` or `unwrap()` on user input.

### 9.3 Hide too much

A macro that generates hundreds of lines makes errors hard to understand. The generated code must stay **visible** (`cargo expand`) and **documented**: which types, which functions, which names appear.

---

## 10. Example in Runique: `model!{}` and `extend!{}`

### 10.1 The syntax

```rust
// demo-app/src/entities/chapitre.rs
model! {
    Chapitre,
    table: "chapitre",
    pk: id => Pk,
    {
        cour_id:    Pk [required],
        slug:       text [required],
        title:      text [required],
        lead:       text [nullable],
        sort_order: int [required],
    },
    relations: {
        belongs_to: Cour via cour_id [cascade],
    }
}
```

It uses the techniques of this course: a keyword followed by `:` (`table:`, `pk:`, `relations:`), a `{ ... }` group of fields, optional lists in brackets (`[required]`, `[cascade]`), and custom keywords (`via`, `belongs_to`).

### 10.2 The split into three crates

```
model! { ... }
   │  tokens
   ▼
runique_dsl      parses the DSL → tree (AST)          normal crate
   │
   ├──► derive_form   generates the SeaORM entity, the columns, the form   (proc-macro)
   │
   └──► runique (CLI) makemigrations: compares the tree to the schema and writes the migrations
```

The parser is in `runique_dsl`, an **ordinary** crate: a `proc-macro` crate cannot export anything other than macros, and the `runique makemigrations` command must read exactly the same declarations as the macro. The same `impl Parse` therefore serves at compile time (code generation) and in the CLI (SQL migration generation).

Another advantage: the parser can be tested like ordinary code (section 7.4). `runique_dsl` has property tests (`proptest`) that send it generated inputs: it must answer with a tree or an error, **never** panic.

### 10.3 `extend!{}`: validating at compile time

```rust
// runique/derive_form/src/lib.rs (excerpt)
#[proc_macro]
pub fn extend(input: TokenStream) -> TokenStream {
    const FRAMEWORK_TABLES: &[&str] = &["eihwaz_users", "eihwaz_groupes"];

    let dsl = match syn::parse::<extend_schema::ExtendDsl>(input) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error().into(),
    };

    if !FRAMEWORK_TABLES.contains(&dsl.table.as_str()) {
        let tables_list = FRAMEWORK_TABLES.join(", ");
        let msg = format!(
            "extend!{{}}: \"{}\" is not a known framework table. Allowed: {}",
            dsl.table, tables_list
        );
        return quote::quote! { compile_error!(#msg); }.into();
    }
    // ...
}
```

`extend!{}` adds columns to the framework tables. An unknown table is rejected **at compile time**, with the list of allowed tables: the typo cannot reach a migration.

### 10.4 Feature guards in the macro crate

```rust
// runique/derive_form/src/lib.rs (start)
#[cfg(all(not(doc), not(feature = "all-databases"), feature = "postgres", feature = "mysql"))]
compile_error!("features `postgres` and `mysql` are mutually exclusive");
```

The macro crate receives the same features as `runique` (forwarded by `runique/Cargo.toml`: `postgres = [..., "derive_form/postgres"]`). It therefore knows the database engine at compile time and applies the same exclusion rules (see `cargo-dependances`, section 9).

The old mechanism read the engine from `.env` (`DB_ENGINE`, `DATABASE_URL`) during compilation, with exactly the flaws of section 9.1: a modified `.env` did not trigger a recompilation, and a forgotten file could point to a different engine than the one actually compiled. It remains as a fallback, only for projects whose `Cargo.toml` does not forward the feature yet (`runique/derive_form/src/model/utils/db_engine.rs`); the feature itself cannot diverge from what is compiled.

---

## 11. Comparing the three kinds of procedural macros

| | Derive | Attribute | Function-like |
|---|---|---|---|
| Declaration | `#[proc_macro_derive(Name)]` | `#[proc_macro_attribute]` | `#[proc_macro]` |
| Signature | `fn(item) -> TokenStream` | `fn(attr, item) -> TokenStream` | `fn(input) -> TokenStream` |
| Use | `#[derive(Name)]` on a type | `#[name(...)]` on an item | `name!(...)` anywhere |
| Receives | a valid type (struct, enum, union) | a valid Rust item + arguments | any tokens |
| Produces | added code | the replacement of the item | anything valid at that position |
| Modifies existing code | no | yes | there is no existing code |
| Examples | `Serialize`, `Debug`, `Error` | `#[tokio::main]`, `#[form]`, `#[async_trait]` | `sqlx::query!`, `model!{}`, `html!` |

Choice:

- add behaviour to a type → **derive**;
- modify an existing Rust item (function, struct) → **attribute**;
- a syntax that is not Rust → **function-like**.

---

## 12. Exercises

### Exercise 1 — `color!`

Write `color!("#ff8800")` that produces `(255u8, 136u8, 0u8)` and rejects at compile time a text that is not a 6-digit hexadecimal colour.

```rust
// Solution:
#[proc_macro]
pub fn color(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as syn::LitStr);
    let v = lit.value();
    let hex = v.strip_prefix('#').unwrap_or(&v);
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return syn::Error::new(lit.span(), "expected: \"#rrggbb\"").to_compile_error().into();
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap();   // checked above
    let (r, g, b) = (channel(0), channel(2), channel(4));
    quote!((#r, #g, #b)).into()
}
```

The `unwrap` is safe here: the previous line checked the six hexadecimal digits.

### Exercise 2 — `text_enum!`

Write `text_enum! { Status: draft, published, archived }` that generates `enum Status { Draft, Published, Archived }` with `fn as_str(&self) -> &'static str` returning the name in lowercase.

```rust
// Solution:
struct TextEnum { name: syn::Ident, variants: Vec<syn::Ident> }

impl Parse for TextEnum {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let v = Punctuated::<syn::Ident, Token![,]>::parse_terminated(input)?;
        Ok(TextEnum { name, variants: v.into_iter().collect() })
    }
}

#[proc_macro]
pub fn text_enum(input: TokenStream) -> TokenStream {
    let e = parse_macro_input!(input as TextEnum);
    let name = &e.name;
    let variants: Vec<_> = e.variants.iter()
        .map(|v| quote::format_ident!("{}", capitalize(&v.to_string())))
        .collect();
    let texts: Vec<String> = e.variants.iter().map(|v| v.to_string()).collect();
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #name { #(#variants),* }
        impl #name {
            pub fn as_str(&self) -> &'static str {
                match self { #( Self::#variants => #texts ),* }
            }
        }
    }
    .into()
}
```

### Exercise 3 — duplicate key

Test, without a macro, that the parser from section 7.2 rejects `order: "a", order: "b"`.

```rust
// Solution:
#[test]
fn duplicate_order() {
    let r: syn::Result<Meta> = syn::parse_str(r#"order: "a", order: "b""#);
    let e = r.err().expect("must fail");
    assert!(e.to_string().contains("already defined"));
}
```

## 13. Cheat sheet

```rust
#[proc_macro]
pub fn name(input: TokenStream) -> TokenStream {
    let dsl = parse_macro_input!(input as MyDsl);   // impl Parse for MyDsl
    generate(&dsl).unwrap_or_else(|e| e.to_compile_error()).into()
}

input.parse::<Ident>()?        input.parse::<Token![:]>()?      input.parse::<LitStr>()?
syn::custom_keyword!(table);   input.parse::<kw::table>()?
input.peek(Token![,])          input.peek(syn::token::Bracket)  input.is_empty()
let l = input.lookahead1(); if l.peek(LitStr) { .. } else { Err(l.error()) }
let c; braced!(c in input);    bracketed!   parenthesized!
Punctuated::<T, Token![,]>::parse_terminated(&c)?
syn::Error::new(lit.span(), "..")   syn::parse_str::<T>("..")   format_ident!("{}", x)
```

### Key points

- A procedural function-like macro is for when you need to compute, build names or read a free syntax.
- Write examples of the syntax first, then one `impl Parse` per element.
- `lookahead1` produces "expected A or B" messages; `Option::replace` detects duplicates.
- Validating at compile time costs nothing at runtime, but does not replace query parameters.
- The macro must depend only on its tokens: no hidden file, network or environment.
- Put the parser in a normal crate to test it and reuse it (like `runique_dsl`).
