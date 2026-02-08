# Approche Admin — Architecture Runique

## Résumé

Système admin unique sans compilation ni runtime impact. Basé sur **convention stricte**, **registre centralisé** et **démon de surveillance en temps réel**.

---

## 1. Convention Stricte (Django-like)

Structure obligatoire :
```
src/
  models/
    users.rs       → struct Model { ... }
    blog.rs        → struct Model { ... }
    mod.rs

  forms/
    users.rs       → struct RegisterForm + impl RuniqueForm
    blog.rs        → struct Blog + impl RuniqueForm
    mod.rs

  admin.rs         → Déclaration des paires (registre)
```

**Un seul model et un seul form principal par fichier** — clarté maximale.

---

## 2. Registre Centralisé (`src/admin.rs`)

Fichier source unique où le dev déclare les paires model ↔ form :

```rust
// src/admin.rs

#[macro_export]
macro_rules! admin {
    ($model:path => $form:path) => {};  // no-op compile
}

admin!(crate::models::users::Model => crate::forms::users::RegisterForm);
admin!(crate::models::blog::Model => crate::forms::blog::Blog);
```

**Avantages** :
- Source unique, lisible.
- Paths explicites → zéro ambiguïté.
- Macro no-op → aucun impact compilation.
- Démon parse ce fichier pour extraire les paires.

---

## 3. Démon de Surveillance (Temps réel)

Process indépendant qui :
1. **Watcher** : surveille `src/**/*.rs` via `notify`.
2. **Parser** : extrait modèles, formulaires avec `syn`.
3. **Résolution** : match les chemins → fichiers → structs.
4. **Extraction champs** :
   - Model : `struct Model { field1, field2, ... }`
   - Form : littéraux dans `register_fields()` (ex. `TextField::...("username")`)
5. **Comparaison** : diff sets avec exclusions (`id`, `created_at`, `updated_at`).
6. **Diagnostics** : publie JSON (`.runique/diagnostics.json`) ou STDOUT.

**Temps réel** : déclenchement ~100-200ms après sauvegarde.

---

## 4. Intégration Builder Intelligent

Slot middleware dédié :
```rust
const SLOT_ADMIN_AUTH: u8 = 65;  // Entre CSRF(60) et Host(70)
```

Usage :
```rust
RuniqueApp::builder(config)
    .statics()
    .routes(app_routes)
    .with_admin(true)           // Activ/inactif
    .middleware(|m| { ... })
    .build().await?
```

Si `.with_admin(true)` :
- Routes admin activées (`/admin/*`)
- Middleware auth admin injecté (slot 65)
- Démon surveille `admin.rs`
- Registre des forms exposé au contexte Tera

---

## 5. Responsabilités AdminStaging

1. **Routing** : `/admin/login`, `/admin/dashboard`, `/admin/forms/{key}`, etc.
2. **Auth middleware** : vérifier `is_admin` + permissions par rôle.
3. **Découverte** : parser `admin.rs` une fois au démarrage + hot-reload démon.
4. **Registry** : construire map Form (clé/titre/permissions) → exposée au context Tera.

---

## 6. Templates Admin

**Dev les fournit** via include Tera :
```html
<!-- templates/admin/*.html -->
{% include "admin/login.html" %}
{% include "admin/dashboard.html" %}

{% for form_meta in admin_forms %}
  <div class="form-group">
    <h3>{{ form_meta.title }}</h3>
    <!-- form_meta.key, form_meta.permissions, etc. -->
  </div>
{% endfor %}
```

Framework = **zéro gestion template**, 100% flex Tera.

---

## 7. Comparaison Champs (Sans Compilation)

Statique et déporté du build :

- **Source de vérité** : `Model` struct.
- **Vérifications** :
  - Champs manquants dans Form (warning).
  - Champs en trop dans Form (warning).
  - Doublons de Form sur un Model (error).
  - Exclusions appliquées : `["id", "created_at", "updated_at"]`.
- **Diagnostics** : publiés JSON → affichés dans l'éditeur via problemMatcher ou extension légère.

---

## 8. Avantages Clés

✓ **Convention stricte** = prévisibilité + maintenabilité.
✓ **Zéro impact compilation** = rapidité build.
✓ **Diagnostics temps réel** = UX rapide (feedback ~100ms).
✓ **Registre centralisé** = source unique de vérité.
✓ **Extensible** : AdminRegistrable trait pour métadonnées futures.
✓ **Integration builder** : `.with_admin(true)` + slots intelligents.

---

## 9. Phase Suivante

- [ ] Implémenter AdminStaging + routing basic.
- [ ] Créer démon + parser `syn` complet.
- [ ] Intégrer au builder (flag + slot middleware).
- [ ] Tester sur demo-app (models/users + forms/users).
- [ ] Ajouter trait AdminRegistrable (optionnel, extensibilité).

---

## Notes

- **Proc-macros** : volontairement exclues (complexité, expansion nécessaire).
- **Normalisation champs** : snake_case, gestion `serde(rename)` si nécessaire.
- **Permissions** : gérées côté auth middleware (role-based).
- **Database** : imposer convention table `users` + ORM mappé.
