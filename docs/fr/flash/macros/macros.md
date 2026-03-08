# Macros flash

## Macros de redirection

Ces macros stockent les messages en session via `request.notices`. Ils s'affichent **après la prochaine redirection** (pattern Post/Redirect/Get).

### success!

```rust
success!(request.notices => "Utilisateur créé avec succès !");
success!(request.notices => format!("Bienvenue {} !", username));

// Plusieurs messages en une fois
success!(request.notices => "Créé", "Email envoyé", "Bienvenue !");
```

### error!

```rust
error!(request.notices => "Une erreur s'est produite");
error!(request.notices => format!("Erreur : {}", e));
```

### info!

```rust
info!(request.notices => "Veuillez vérifier votre email");
```

### warning!

```rust
warning!(request.notices => "Cette action ne peut pas être annulée");
```

> Chaque macro appelle `.success()`, `.error()`, `.info()` ou `.warning()` sur `request.notices` (de type `Message`).

---

## Macro flash_now! — Messages immédiats

`flash_now!` crée un `Vec<FlashMessage>` pour affichage **immédiat** dans la requête courante. Idéal pour les cas où il n'y a pas de redirection (par exemple, ré-affichage du formulaire après une erreur de validation).

```rust
// Un seul message
let msgs = flash_now!(error => "Veuillez corriger les erreurs");

// Plusieurs messages
let msgs = flash_now!(warning => "Champ A incorrect", "Champ B manquant");
```

### Types disponibles

| Type | Classe CSS générée |
|------|-------------------|
| `success` | `message-success` |
| `error` | `message-error` |
| `info` | `message-info` |
| `warning` | `message-warning` |

### Injection dans le contexte

`flash_now!` retourne un vecteur à injecter manuellement dans le contexte :

```rust
context_update!(request => {
    "title" => "Erreur de validation",
    "form" => &form,
    "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
});
```

---

## Différence flash vs flash_now

| | `success!` / `error!` / etc. | `flash_now!` |
|---|---|---|
| **Stockage** | Session | Mémoire (Vec) |
| **Affichage** | Après redirect | Requête courante |
| **Durée de vie** | Jusqu'à la prochaine lecture | Requête unique |
| **Usage typique** | Post/Redirect/Get | Ré-affichage formulaire |
| **Injection contexte** | Automatique | Manuelle (`"messages" => flash_now!(...)`) |

---

## Quand utiliser quoi ?

### Utilisez les macros flash (session)

```rust
// Après une action réussie avec redirection
success!(request.notices => "Sauvegardé !");
return Ok(Redirect::to("/").into_response());
```

### Utilisez flash_now! (immédiat)

```rust
// Erreur de validation → ré-afficher la page sans redirect
context_update!(request => {
    "form" => &form,
    "messages" => flash_now!(error => "Formulaire invalide"),
});
return request.render("form.html");
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Handlers](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/handlers/handlers.md) | Utilisation dans les handlers |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/templates/templates.md) | Affichage dans les templates |

## Retour au sommaire

- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md)
