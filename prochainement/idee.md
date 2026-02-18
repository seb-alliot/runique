---

# Admin Rendering Architecture — Design Notes (Runique)

##  Objectif

Permettre la création d’une **vue admin Django-like** sans dupliquer :

* les routes,
* les handlers,
* la logique métier,
* les formulaires,
* ni les règles de validation.

L’admin devient une **projection de l’application existante**, obtenue par un changement de **layout de rendu**, et non par la génération de code spécifique.

---

##  Principe fondamental

> **Même logique, même routes, mêmes données — seule la présentation change.**

La différenciation *public / admin* se fait au niveau du **rendu (layout)**, pas au niveau :

* du routeur,
* des handlers,
* ni de l’accès à la base de données.

---

## 🏗️ Architecture retenue

### Pipeline de rendu

```
Request
  ↓
Handler (logique métier + context)
  ↓
Middleware Admin (optionnel)
  ↓
Renderer (Tera)
  ↓
HTML
```

* Les **handlers** produisent des données (via `context_update!`)
* Le **middleware admin** modifie uniquement le **layout**
* Le **renderer final** assemble le tout

---

## 🧱 Convention de templates (Template Contract)

Pour permettre le **swap de layout**, tous les layouts doivent respecter un **contrat minimal de blocks**.

### Blocks obligatoires (contrat v1)

* `style` (ou `head`)
* `content` (**obligatoire**)
* `scripts` (optionnel)
* `footer` (optionnel)

### Règles

* Les **pages** n’écrivent que dans ces blocks
* Les **layouts** (public et admin) exposent tous ces blocks
* L’admin peut ajouter sidebar, header, breadcrumbs **autour** de `content`

👉 Les pages restent totalement agnostiques du mode admin.

---

## 🧠 Layout dynamique via Tera

Les pages utilisent un layout dynamique :

```tera
{% extends base_template %}
```

### Valeurs possibles

* Public (par défaut)

  ```text
  base_template = "index.html"
  ```

* Admin

  ```text
  base_template = "admin/index.html"
  ```

Cette clé est injectée :

* par défaut au niveau du framework,
* ou surchargée par un **middleware admin**.

---

## 🔐 Middleware Admin

Le middleware admin est responsable de :

* détecter le contexte admin (ex: `/admin/*`)
* vérifier les permissions (RBAC, auth)
* surcharger le layout
* injecter les données admin globales

### Exemple conceptuel

```rust
if request.path().starts_with("/admin") {
    context_update!(request => {
        "base_template" => "admin/index.html",
        "admin" => {
            "nav": ...,
            "breadcrumbs": ...,
        }
    });
}
```

---

## 🧠 Avantages de cette approche

✅ Aucune duplication de logique
✅ Aucune génération de routes admin
✅ Réutilisation complète des formulaires
✅ Admin extensible (thèmes, backoffice, mobile)
✅ Simplicité mentale : *layout swap*
✅ Compatible avec l’existant

---

## 🤖 Rôle du daemon (réévalué)

Le daemon **n’est plus nécessaire pour le rendu admin**.

Il peut néanmoins rester utile comme **outil DX** pour :

* générer une navigation admin
* analyser models/forms
* proposer des CRUD par défaut
* accélérer l’onboarding

👉 Le daemon devient **optionnel**, pas structurel.

---

## 🧭 Vision long terme

Cette architecture permet :

* Admin
* Backoffice
* Thèmes
* White-label
* Multi-frontend

… **sans changer les handlers**.

L’admin n’est plus un module :
➡️ **c’est un mode de rendu**.

---

## 📝 Conclusion

Ce design transforme une idée de *“panel admin basique”* en une **capacité centrale du framework**.

Il s’appuie sur :

* des conventions simples,
* un pipeline clair,
* et une séparation stricte entre **logique métier** et **présentation**.

---
