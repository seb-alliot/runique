# Lifecycle d'une requête

## Cycle de vie

```text
1. La requête HTTP arrive
2. Les middlewares sont traversés dans l'ordre des slots (voir Stack middleware) :
   Engine/Tera/Config injectés, IP cliente, hôte vérifié, en-têtes de sécurité…
3. La session est chargée, puis le compte connecté relu en base (request.user)
4. CSRF : un jeton envoyé en en-tête (X-CSRF-Token) est vérifié ici ; un jeton
   de formulaire (champ csrf_token) est vérifié par Prisme à l'étape suivante
5. Le handler reçoit Request : le corps est lu (Prisme), request.form() construit
   le formulaire, ValidationForm::try_form le valide
6. Le handler retourne AppResult<Response>
7. Les middlewares sont traversés en sens inverse (la session est enregistrée)
8. La réponse HTTP est envoyée
```

---

## Bonnes pratiques

### 1. Cloner les Arc

```rust
let db = request.engine.db.clone();
```

### 2. Formulaires = copies par requête

```rust
let mut form: MyForm = request.form();
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
| [Concepts clés](/docs/fr/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |

## Retour au sommaire

- [Architecture](/docs/fr/architecture)
