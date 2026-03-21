# Lifecycle d'une requête

## Cycle de vie

```text
1. Requête HTTP arrive
2. Middlewares traversés (order des slots)
3. Extensions injectées (Engine, Tera, Config)
4. Session chargée, CSRF vérifié
5. Handler appelé avec extracteurs (Request, Prisme<T>)
6. Handler retourne AppResult<Response>
7. Middlewares traversés en sens inverse
8. Réponse HTTP envoyée
```

---

## Bonnes pratiques

### 1. Cloner les Arc

```rust
let db = request.engine.db.clone();
```

### 2. Formulaires = copies par requête

```rust
Prisme(mut form): Prisme<MyForm>
// Chaque requête = formulaire isolé, zéro concurrence
```

### 3. `context_update!` pour le contexte

```rust
context_update!(request => {
    "title" => "Ma page",
    "data" => &my_data,
});
```

### 4. Flash messages pour les redirections

```rust
success!(request.notices => "Action réussie !");
return Ok(Redirect::to("/").into_response());
```

### 5. `flash_now!` pour les rendus directs

```rust
context_update!(request => {
    "messages" => flash_now!(error => "Erreur de validation"),
});
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Concepts clés](/docs/fr/architecture/concepts) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |

## Retour au sommaire

- [Architecture](/docs/fr/architecture)
