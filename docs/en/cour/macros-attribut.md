# Procedural macros — Attribute
> An attribute macro receives an item (function, struct, module...) and **replaces** it with the code it produces. Reading its arguments, transforming a function or a struct, and adding compile-time checks.

## Objectives

- Understand the difference between a derive macro and an attribute macro
- Write the signature of an attribute macro and parse the item it receives
- Read the attribute arguments in several ways (empty, list, key = value, free syntax)
- Transform a function: wrap its body, keep its signature, handle `async`
- Transform a struct: add fields, generate `impl` blocks
- Have the compiler check names with `quote_spanned!`
- Know the common attribute macros of the ecosystem

## Table of contents

1. Derive or attribute
2. Signature
3. Example: measuring execution time
4. Reading the attribute arguments
5. Transforming a function
6. Transforming a struct
7. Compile-time checks and error positions
8. Attribute order
9. Attribute macros in the ecosystem
10. Example in Runique: `#[form]`
11. Attribute, derive or `macro_rules!`
12. Exercises
13. Cheat sheet

---

## 1. Derive or attribute

| | `#[derive(X)]` | `#[x(...)]` |
|---|---|---|
| Applies to | struct, enum, union | any item: function, struct, enum, impl, module, trait |
| Receives | the item | the arguments **and** the item |
| Produces | code **added** after the item | the code that **replaces** the item |
| Can modify the item | no | yes (or copy it back unchanged) |
| Arguments | via helper attributes | directly: `#[x(a = 1, b)]` |

An attribute macro is more powerful, and therefore more surprising: the code the user reads is not necessarily the code that is compiled. Keep it for cases where you need to **change** the item.

---

## 2. Signature

```rust
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn my_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr: what is between the parentheses of #[my_attribute(...)]
    // item: the annotated item, complete
    item                // returning item unchanged = the attribute changes nothing
}
```

```rust
#[my_attribute(level = "debug", repeats = 3)]
fn compute() {}
// attr = level = "debug", repeats = 3
// item = fn compute() {}
```

The crate is a `proc-macro` crate, with `syn` and `quote`, as for a derive (see `macros-derive`, section 3). To transform functions and arbitrary code, `syn` needs the `full` feature.

---

## 3. Example: measuring execution time

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn timer(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);

    let attrs = &function.attrs;          // #[inline], ///doc... : to keep
    let vis = &function.vis;
    let sig = &function.sig;              // name, parameters, return type, async, generics
    let body = &function.block;
    let name = sig.ident.to_string();

    quote! {
        #(#attrs)*
        #vis #sig {
            let __start = ::std::time::Instant::now();
            let __result = (|| #body)();
            ::std::eprintln!("{}: {:?}", #name, __start.elapsed());
            __result
        }
    }
    .into()
}
```

```rust
#[timer]
fn compute(n: u64) -> u64 {
    (0..n).sum()
}

fn main() {
    compute(1_000_000);        // compute: 1.2ms
}
```

Important points:

- the attributes, visibility and signature are **copied back**, otherwise the function would lose its documentation or become private;
- the body is placed in a closure: a `return` in the body leaves the closure, and the duration is still printed;
- the added variables have an unlikely name (`__start`): a procedural macro is **not** hygienic, so a `start` could conflict with a user variable.

Limit of this version: a closure cannot contain `.await`, and a `?` in the body only works if the error type can be inferred. Section 5.3 covers `async` functions.

---

## 4. Reading the attribute arguments

### 4.1 No argument expected

```rust
#[proc_macro_attribute]
pub fn no_argument(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(proc_macro2::Span::call_site(), "#[no_argument] takes no argument")
            .to_compile_error()
            .into();
    }
    item
}
```

### 4.2 A list of `key = value` pairs and flags

```rust
use syn::{parse::Parser, punctuated::Punctuated, Expr, Lit, Meta, Token};

struct Options { level: String, repeats: u32, quiet: bool }

fn read_options(attr: TokenStream) -> syn::Result<Options> {
    let mut o = Options { level: "info".into(), repeats: 1, quiet: false };
    let list = Punctuated::<Meta, Token![,]>::parse_terminated.parse(attr)?;
    for meta in list {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("level") => {
                let Expr::Lit(e) = &nv.value else {
                    return Err(syn::Error::new_spanned(&nv.value, "string expected"));
                };
                let Lit::Str(s) = &e.lit else {
                    return Err(syn::Error::new_spanned(&e.lit, "string expected"));
                };
                o.level = s.value();
            }
            Meta::NameValue(nv) if nv.path.is_ident("repeats") => {
                let Expr::Lit(syn::ExprLit { lit: Lit::Int(n), .. }) = &nv.value else {
                    return Err(syn::Error::new_spanned(&nv.value, "integer expected"));
                };
                o.repeats = n.base10_parse()?;
            }
            Meta::Path(p) if p.is_ident("quiet") => o.quiet = true,
            other => return Err(syn::Error::new_spanned(other, "unknown option")),
        }
    }
    Ok(o)
}
```

`Meta` covers the three forms: `path` (flag), `path = value`, `path(...)` (nested list).

### 4.3 The same reading with `parse_nested_meta`

```rust
use syn::meta;

fn read_options_v2(attr: TokenStream) -> syn::Result<Options> {
    let mut o = Options { level: "info".into(), repeats: 1, quiet: false };
    let reader = meta::parser(|m| {
        if m.path.is_ident("level") {
            o.level = m.value()?.parse::<syn::LitStr>()?.value();
        } else if m.path.is_ident("repeats") {
            o.repeats = m.value()?.parse::<syn::LitInt>()?.base10_parse()?;
        } else if m.path.is_ident("quiet") {
            o.quiet = true;
        } else {
            return Err(m.error("unknown option"));
        }
        Ok(())
    });
    syn::parse::Parser::parse(reader, attr)?;
    Ok(o)
}
```

Shorter, and with well-placed error messages. This is the form recommended by `syn` 2.

### 4.4 Free syntax: implementing `Parse`

For arguments that are not `Meta` (a path, a list in brackets, a custom keyword), you write your own parser:

```rust
use syn::{bracketed, parse::{Parse, ParseStream}, punctuated::Punctuated, Ident, Path, Token};

// #[table(name = users, columns = [id, email])]
struct TableArgs { name: Ident, columns: Vec<Ident> }

impl Parse for TableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut columns = Vec::new();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => name = Some(input.parse::<Ident>()?),
                "columns" => {
                    let content;
                    bracketed!(content in input);                 // [ ... ]
                    let list: Punctuated<Ident, Token![,]> =
                        content.parse_terminated(Ident::parse, Token![,])?;
                    columns = list.into_iter().collect();
                }
                _ => return Err(syn::Error::new(key.span(), "unknown key: `name` or `columns`")),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;                      // optional comma between keys
            }
        }
        let name = name.ok_or_else(|| input.error("`name` is required"))?;
        Ok(TableArgs { name, columns })
    }
}

#[proc_macro_attribute]
pub fn table(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as TableArgs);
    let _ = args;
    item
}
```

| Form | For |
|---|---|
| `Punctuated<Meta, Token![,]>` | simple options, fixed structure |
| `meta::parser` | simple options, better messages |
| `impl Parse` | custom syntax (paths, lists, keywords) |
| `darling::FromMeta` | many typed options, default values |

---

## 5. Transforming a function

### 5.1 What an `ItemFn` contains

```rust
pub struct ItemFn {
    pub attrs: Vec<Attribute>,     // remaining attributes (including docs)
    pub vis: Visibility,
    pub sig: Signature,            // const, async, unsafe, fn, name, generics, parameters, return type
    pub block: Box<Block>,         // the body
}
```

```rust
let sig = &function.sig;
sig.ident;                 // name
sig.asyncness.is_some();   // async?
sig.inputs;                // parameters: FnArg::Receiver (self) or FnArg::Typed (name: Type)
sig.output;                // ReturnType::Default (()) or ReturnType::Type(_, ty)
sig.generics;
```

### 5.2 Running code before the body, without a closure

```rust
#[proc_macro_attribute]
pub fn trace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);
    let name = f.sig.ident.to_string();
    let body = &f.block;

    // Replace the body with a new block that contains the old one
    let new_block: syn::Block = syn::parse_quote!({
        ::std::eprintln!("entering {}", #name);
        #body
    });
    f.block = Box::new(new_block);

    quote!(#f).into()          // ItemFn is interpolated as is
}
```

Modifying the tree (`f.block = ...`) and then interpolating all of it automatically keeps the attributes, visibility and signature.

### 5.3 `async` functions

```rust
#[proc_macro_attribute]
pub fn timer_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);
    let name = f.sig.ident.to_string();
    let body = &f.block;

    let new_block: syn::Block = if f.sig.asyncness.is_some() {
        syn::parse_quote!({
            let __start = ::std::time::Instant::now();
            let __r = async move #body.await;          // an async block keeps .await and ?
            ::std::eprintln!("{}: {:?}", #name, __start.elapsed());
            __r
        })
    } else {
        syn::parse_quote!({
            let __start = ::std::time::Instant::now();
            let __r = (|| #body)();
            ::std::eprintln!("{}: {:?}", #name, __start.elapsed());
            __r
        })
    };
    f.block = Box::new(new_block);
    quote!(#f).into()
}
```

### 5.4 Reading the parameters

```rust
use syn::{FnArg, Pat};

let names: Vec<_> = f.sig.inputs.iter().filter_map(|a| match a {
    FnArg::Typed(pt) => match &*pt.pat {
        Pat::Ident(i) => Some(i.ident.clone()),
        _ => None,                                 // destructured pattern: no simple name
    },
    FnArg::Receiver(_) => None,                    // self
}).collect();

// quote! { ::std::eprintln!(concat!(#(stringify!(#names), " = {:?} "),*), #(#names),*); }
```

### 5.5 Rejecting what is not a function

```rust
let f = match syn::parse::<ItemFn>(item.clone()) {
    Ok(f) => f,
    Err(_) => {
        return syn::Error::new(proc_macro2::Span::call_site(), "#[timer] applies to a function")
            .to_compile_error()
            .into();
    }
};
```

---

## 6. Transforming a struct

### 6.1 Adding a field

```rust
use syn::{parse_macro_input, parse_quote, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn timestamped(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut s = parse_macro_input!(item as ItemStruct);

    if let Fields::Named(fields) = &mut s.fields {
        fields.named.push(parse_quote! { pub created_at: ::std::time::SystemTime });
        fields.named.push(parse_quote! { pub updated_at: ::std::time::SystemTime });
    } else {
        return syn::Error::new_spanned(&s.ident, "#[timestamped] requires named fields")
            .to_compile_error()
            .into();
    }

    let name = &s.ident;
    quote! {
        #s
        impl #name {
            pub fn touch(&mut self) { self.updated_at = ::std::time::SystemTime::now(); }
        }
    }
    .into()
}
```

```rust
#[timestamped]
struct Article { title: String }

// becomes:
// struct Article { title: String, pub created_at: SystemTime, pub updated_at: SystemTime }
// impl Article { pub fn touch(&mut self) { .. } }
```

A derive could not add a field: this is the case where the attribute is needed. The price: the user sees `struct Article { title }` but must build `Article { title, created_at, updated_at }`. A macro that adds fields must therefore provide a constructor and document it.

### 6.2 Filling an empty struct

```rust
#[proc_macro_attribute]
pub fn make_form(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let s = parse_macro_input!(item as ItemStruct);
    let vis = &s.vis;
    let name = &s.ident;
    let attrs = &s.attrs;

    if !matches!(s.fields, Fields::Unit) {
        return syn::Error::new_spanned(&s.fields, "write `struct Name;`: the fields are generated")
            .to_compile_error()
            .into();
    }

    quote! {
        #(#attrs)*
        #vis struct #name {
            pub data: ::std::collections::HashMap<String, String>,
            pub errors: Vec<String>,
        }
    }
    .into()
}
```

The user writes `#[make_form] pub struct Contact;` and gets a complete struct. This is the model of `#[form]` in Runique (section 10).

---

## 7. Compile-time checks and error positions

### 7.1 Having the compiler check a name

A macro only sees syntax: it does not know whether `email` is really a column. But it can **generate** code that only compiles if the name exists:

```rust
use quote::quote_spanned;

// For each column named by the user, a line that does nothing at runtime
let checks = columns.iter().map(|c| {
    quote_spanned! { c.span()=>
        let _ = #schema::Column::#c;          // ERROR if the column does not exist
    }
});

let code = quote! {
    const _: () = { #(#checks)* };            // constant block: nothing is executed
};
```

- `const _: () = { ... };` creates a block checked at compile time, with no runtime cost.
- `quote_spanned! { span=> ... }` gives the generated code the **position** of the name written by the user: if the column does not exist, the error underlines `emial` in their attribute, not an invisible line of the macro.

### 7.2 A compile-time assertion

```rust
let messages = columns.iter().map(|c| {
    let text = c.to_string();
    let msg = format!("`{text}` is the primary key: it cannot be a form field");
    quote_spanned! { c.span()=>
        assert!(!::my_lib::same_name(#schema::PRIMARY_KEY, #text), #msg);
    }
});
```

In a `const` block, a failing `assert!` is a **compile error** with this message. The function called must be a `const fn`.

### 7.3 `quote!` or `quote_spanned!`

| | `quote!` | `quote_spanned!` |
|---|---|---|
| Position of the generated code | the macro call | the one you choose |
| An error in this code underlines | the whole attribute | the chosen element (a name, a type) |
| For | code that cannot fail | checks on what the user wrote |

---

## 8. Attribute order

Attribute macros apply **from top to bottom**:

```rust
#[a]          // runs first, and receives the item WITH #[b] and #[derive] in its attributes
#[b]          // runs next, on what #[a] produced
#[derive(Debug)]
struct S;
```

`#[a]` receives `#[b] #[derive(Debug)] struct S;`. If it copies the attributes back (`#(#attrs)*`), `#[b]` and then the derive apply on the result. An attribute macro that **forgets** to copy them back makes everything placed below it disappear, including `#[derive(...)]`, `#[cfg]` and the documentation.

---

## 9. Attribute macros in the ecosystem

| Attribute | Crate | What it does |
|---|---|---|
| `#[tokio::main]` | tokio | turns `async fn main` into a `fn main` that starts a runtime |
| `#[tokio::test]` | tokio | same for a test |
| `#[async_trait]` | async-trait | rewrites the `async fn`s of a trait into functions that return a `Pin<Box<dyn Future>>` |
| `#[tracing::instrument]` | tracing | opens a *span* with the arguments on each call |
| `#[serde_as]` | serde_with | adjusts the serde attributes of fields |
| `#[test]`, `#[bench]` | std | mark a test function |

```rust
// What #[tokio::main] does, simplified
#[tokio::main]
async fn main() { /* ... */ }

// becomes
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { /* ... */ })
}
```

---

## 10. Example in Runique: `#[form]`

```rust
#[form(schema = crate::entities::contact, fields = [email, message])]
pub struct ContactForm;
```

### 10.1 The arguments: a custom `Parse`

```rust
// runique/derive_form/src/schema_form.rs (excerpt)
enum Columns {
    Only(Vec<Ident>),     // fields = [a, b]
    All,                  // fields = All
    Except(Vec<Ident>),   // exclude = [a, b]
}

struct FormAttrs {
    schema: Path,
    columns: Columns,
}

impl Parse for FormAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // key = value loop: schema, fields, exclude
        // fields and exclude together -> error; schema missing -> error;
        // neither fields nor exclude -> "incomplete declaration" error
        // ...
    }
}
```

This is the technique of section 4.4. The design choice: `fields` or `exclude` is **required**. A form that took all columns without saying so would also take any column added to the model later (an `is_admin` field, for example), a classic mass assignment flaw. `fields = All` is still possible, but you have to write it.

### 10.2 The struct transformation

The user writes an empty struct (`pub struct ContactForm;`); the macro generates a `pub form: Forms` field and the `FormAccess` and `ModelForm` implementations (section 6.2).

### 10.3 The compile-time checks

```rust
// runique/derive_form/src/schema_form.rs (excerpt)
// Each name a column of the entity, checked by the compiler: the error
// lands on the name, not on a panic the first time the form is used.
let checks = fields.iter().chain(exclude.iter()).flatten().map(|field| {
    let mut variant = crate::model::utils::ident_pascal(field);
    variant.set_span(field.span());
    quote_spanned! { field.span()=> let _ = #schema_path::Column::#variant; }
});

// The key is the database's to set: listed in `fields`, refused here.
let key_checks = fields.iter().flatten().map(|field| {
    let name = field.to_string();
    let message = format!(
        "#[form]: `{name}` is the primary key; the database sets it, it is never a form field"
    );
    quote_spanned! { field.span()=>
        assert!(!::runique::forms::__same_name(#schema_path::PRIMARY_KEY, #name), #message);
    }
});

let column_check = quote! { const _: () = { #(#checks)* #(#key_checks)* }; };
```

This is exactly section 7:

- each field name becomes `Column::PascalName`: a typo (`emial`) is a compile error underlined on `emial`;
- listing the primary key in `fields` is rejected at compile time by a `const` assertion, with a message that explains why;
- everything is in a `const _: () = { ... };`: nothing is executed at startup.

---

## 11. Attribute, derive or `macro_rules!`

Adding an entry log to functions.

```rust
// 1. By hand: explicit, but to repeat in every function
fn a() { eprintln!("entering a"); /* ... */ }

// 2. macro_rules!: you must wrap the whole function
macro_rules! traced {
    (fn $name:ident() $body:block) => {
        fn $name() { eprintln!("entering {}", stringify!($name)); $body }
    };
}
traced!(fn b() { /* ... */ });

// 3. Attribute: one line above an ordinary function
// #[trace]
// fn c() { /* ... */ }
```

| | By hand | `macro_rules!` | Attribute |
|---|---|---|---|
| Readability of the annotated code | perfect | average (everything is in the macro) | good |
| Handles generics, `async`, `pub`, attributes | yes | hardly (one arm per case) | yes, with `syn` |
| Cost | none | none | `proc-macro` crate |

---

## 12. Exercises

### Exercise 1 — `#[deprecate_to]`

Write `#[deprecate_to("new_function")]` that adds `#[deprecated(note = "use new_function")]` to the function, without changing anything else.

```rust
// Solution:
#[proc_macro_attribute]
pub fn deprecate_to(attr: TokenStream, item: TokenStream) -> TokenStream {
    let target = parse_macro_input!(attr as syn::LitStr);
    let f = parse_macro_input!(item as ItemFn);
    let note = format!("use {}", target.value());
    quote! {
        #[deprecated(note = #note)]
        #f
    }
    .into()
}
```

### Exercise 2 — `#[repeat(n = 3)]`

Write an attribute that runs the body of a function with no return value `n` times.

```rust
// Solution:
#[proc_macro_attribute]
pub fn repeat(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut n: u32 = 1;
    let reader = syn::meta::parser(|m| {
        if m.path.is_ident("n") {
            n = m.value()?.parse::<syn::LitInt>()?.base10_parse()?;
            Ok(())
        } else {
            Err(m.error("only the `n` option is accepted"))
        }
    });
    if let Err(e) = syn::parse::Parser::parse(reader, attr) {
        return e.to_compile_error().into();
    }
    let mut f = parse_macro_input!(item as ItemFn);
    if !matches!(f.sig.output, syn::ReturnType::Default) {
        return syn::Error::new_spanned(&f.sig.output, "#[repeat] requires a function with no return value")
            .to_compile_error()
            .into();
    }
    let body = &f.block;
    f.block = Box::new(syn::parse_quote!({ for _ in 0..#n #body }));
    quote!(#f).into()
}
```

### Exercise 3 — checked name

In a `#[route(handler = name)]` attribute, have the compiler check that `name` is an existing function, with the error underlined on `name`.

```rust
// Solution (excerpt):
let handler: syn::Ident = /* read from the arguments */ syn::parse_quote!(index);
let check = quote::quote_spanned! { handler.span()=>
    const _: () = { let _ = #handler; };           // does not exist: error underlined on the name
};
```

## 13. Cheat sheet

```rust
#[proc_macro_attribute]
pub fn name(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MyArgs);        // impl Parse for MyArgs
    let mut f = parse_macro_input!(item as ItemFn);       // or ItemStruct, Item...
    f.block = Box::new(parse_quote!({ /* before */ #body }));
    quote!(#f).into()
}

Punctuated::<Meta, Token![,]>::parse_terminated.parse(attr)
syn::meta::parser(|m| { m.path.is_ident("x"); m.value()?.parse::<LitStr>()?; Ok(()) })
bracketed!(content in input)   content.parse_terminated(Ident::parse, Token![,])
quote_spanned! { ident.span()=> let _ = #path::#ident; }
const _: () = { /* checks */ };
```

### Key points

- An attribute **replaces** the item: copy back attributes, visibility and signature.
- A procedural macro is not hygienic: prefix the variables you add (`__start`).
- Handle `async` separately (`async move` block), a closure does not accept `.await`.
- `meta::parser` for simple options, `impl Parse` for custom syntax.
- Adding fields is possible, but the user must know how to build them.
- `quote_spanned!` + `const _: () = { .. }`: have the compiler check names, with the error on the name the user wrote.
