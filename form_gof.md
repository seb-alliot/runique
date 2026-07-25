# Moteur de formulaire Runique — Patterns GoF

Le moteur de formulaire compose **trois patterns GoF** à trois niveaux d'abstraction : **Strategy** (le champ), **Composite** (le formulaire agrège les champs), **Template Method** (le flux validation/save). Chacun est porté par un idiome Rust (trait object, méthodes par défaut) plutôt qu'une hiérarchie de classes.

Fichiers de référence :
- `runique/src/forms/base.rs` — `trait FormField` (interface des champs)
- `runique/src/forms/fields/*` — champs concrets (`TextField`, `ChoiceField`, `FileField`…)
- `runique/src/forms/form.rs` — `struct Forms` (conteneur) + validation/rendu
- `runique/src/forms/validator.rs` — `FormValidator` (itération de validation)
- `runique/src/forms/renderer.rs` — `FormRenderer` (rendu HTML)
- `runique/src/forms/field.rs` — `trait RuniqueForm` (formulaire métier + save)

Class diagrams complets :
- [diagramme/uml/forms/formulaires.md](diagramme/uml/forms/formulaires.md)
- [diagramme/uml/forms/fields-complets.md](diagramme/uml/forms/fields-complets.md)

---

## 1. Strategy — les champs (`FormField`)

**Intention GoF** : encapsuler une famille d'algorithmes interchangeables derrière une interface commune, sélectionnés à l'exécution, pour que le contexte varie sans connaître le type concret.

**Problème résolu** : un formulaire manipule des champs de natures très différentes (texte, choix, fichier, caché, date…) qui doivent tous se **valider** et se **rendre en HTML**, chacun selon ses propres règles.

### Participants

| Rôle GoF | Type Runique |
|----------|--------------|
| `Strategy` (interface) | `trait FormField` |
| `ConcreteStrategy` | `TextField`, `ChoiceField`, `FileField`, `HiddenField`, `NumericField`, `BooleanField`, `DateField`… |
| `Context` (détient les stratégies) | `Forms.fields : IndexMap<String, Box<dyn FormField>>` |

### Les deux opérations abstraites

Le trait `FormField` fournit les getters/setters par défaut (via l'état partagé `FieldConfig`), et déclare **deux opérations abstraites** que chaque stratégie concrète implémente :

```rust
fn validate(&mut self) -> bool;                     // règles propres au champ
fn render(&self, tera: &ATera) -> Result<String, String>;  // HTML du champ
fn finalize(&mut self) -> Result<(), String> { Ok(()) }    // hook (défaut no-op)
```

### Ce que fait `validate()` selon la stratégie

- **`TextField`** : présence si `required`, longueur min/max, format (`SpecialFormat` : Email/Url/Password…). Pose son erreur via `set_error()`, renvoie `false` si invalide.
- **`NumericField`** : parse en Integer/Float/Decimal, borne min/max, pas (`step`).
- **`ChoiceField` / `RadioField` / `CheckboxField`** : la valeur soumise appartient-elle aux `Vec<ChoiceOption>` autorisées.
- **`FileField`** : type (`FileFieldType` Image/Document/Any), extensions (`AllowedExtensions`), taille (`FileSize` vs `model_max_size`).
- **`HiddenField`** : cas CSRF — comparaison constante-temps (`ct_eq`) si `expected_value` est posé (sinon no-op, le CSRF étant validé en amont par Prisme).

### Ce que fait `render()` selon la stratégie

Chaque champ porte un `template_name` (ex. `base_text.html`, `base_file.html`), construit un contexte Tera à partir de son `FieldConfig` (name, label, value, erreur, attributs HTML, readonly/disabled) et rend **son** template. Le résultat est un fragment HTML.

### `finalize()`

Défaut : `Ok(())`. Surchargé par les champs qui transforment leur valeur après validation — typiquement le hash d'un mot de passe.

**Idiome Rust** : `Box<dyn FormField>` = **trait object** (dispatch dynamique), pas d'héritage de classes. Le super-trait **`DynClone`** est requis parce que Rust ne sait pas cloner un `Box<dyn Trait>` nativement — c'est ce qui permet `#[derive(Clone)]` sur `Forms`.

---

## 2. Composite — le formulaire agrège les champs (`Forms`)

**Intention GoF** : composer des objets et traiter uniformément l'ensemble et l'unité pour une opération, le conteneur déléguant l'opération à ses parties puis agrégeant.

**Problème résolu** : « valider le formulaire » = valider tous ses champs et agréger ; « rendre le formulaire » = rendre tous ses champs et concaténer.

### Participants

| Rôle GoF | Type Runique |
|----------|--------------|
| `Component` | l'opération `validate` / `render` de `FormField` |
| `Leaf` | les champs concrets |
| `Composite` | `Forms` (+ `FormValidator`, `FormRenderer`) |

### Étapes exactes de la validation (composite)

1. `Forms::is_valid()` :
   - court-circuit si `force_invalid` (honeypot anti-bot déclenché) → `Ok(false)` ;
   - marque `validated = true` ;
   - appelle `Forms::validate(&mut fields, &errors)`.
2. `Forms::validate()` entoure l'appel d'un **garde de profondeur** (`VALIDATION_DEPTH`, `MAX_VALIDATION_DEPTH`) → renvoie `Err(ValidationError::StackOverflow)` si une validation récursive s'emballe, puis délègue à `FormValidator::validate_fields`.
3. `FormValidator::validate_fields()` **itère chaque champ**, appelle `field.validate()`, collecte les erreurs, et renvoie `true` seulement si **tous** sont valides.

### Étapes exactes du rendu (composite)

1. `Forms::render()` exige un `renderer` configuré (`FormRenderer`, sinon erreur `tera_not_configured`) et lui délègue.
2. `FormRenderer::render(fields, errors)` **itère `fields.values()`**, appelle `field.render(&tera)` pour chaque, et **concatène** les fragments HTML (plus le bloc `<script>` pré-rendu et les erreurs de formulaire).

**Écart au manuel** : c'est un Composite **plat** — `Forms` → champs, sans formulaire imbriqué récursif. L'intention (agrégation + délégation uniforme) est présente ; la récursivité n'est pas implémentée car le besoin ne l'exige pas.

---

## 3. Template Method — le flux validation/save (`RuniqueForm`)

**Intention GoF** : définir le squelette invariant d'un algorithme dans la classe de base, en laissant les sous-classes remplir des étapes via des hooks (« primitive operations »), sans altérer la structure.

**Problème résolu** : l'ordre de validation (champs → nettoyage métier → finalisation) et le save transactionnel doivent être **verrouillés** ; seul le contenu métier de certaines étapes varie d'un formulaire à l'autre.

### Participants (`runique/src/forms/field.rs`)

| Rôle GoF | Type Runique |
|----------|--------------|
| `AbstractClass` | `trait RuniqueForm` (méthodes par défaut) |
| `templateMethod()` | `is_valid()`, `save()` — ordre figé |
| `primitiveOperation()` (hooks) | `clean()`, `clean_field()`, `on_save()`, `before_save()`, `after_save()` — défaut no-op/Ok, surchargeables |
| `ConcreteClass` | le formulaire généré par `#[form]` ou écrit à la main |

### Étapes exactes de `is_valid()` (le template)

1. **Garde de soumission** : si `!get_form().is_submitted()` → `false` immédiatement, **sans** poser d'erreur (évite d'afficher des erreurs au premier GET, et laisse les formulaires de recherche GET tomber dans leur `else`).
2. **Validation des champs** : appelle `Forms::is_valid()` (le composite de la section 2). En cas de `StackOverflow`, pousse une erreur de formulaire et renvoie `false`.
3. **`clean_field()` par champ** : boucle sur les noms de champs ; chaque `clean_field(name)` (hook, défaut = champ présent) peut invalider.
4. **`clean()`** (hook métier global) :
   - `Ok(_)` → appelle `finalize()` (qui peut échouer, ex. hash) ; si ok → `true` ;
   - `Err(business_errors)` → distribue chaque message : sur le champ nommé (`field.set_error`) s'il existe, sinon en erreur de formulaire ; renvoie `false`.

### Étapes exactes de `save()` (template transactionnel)

1. Ouvre une **transaction** SeaORM.
2. `before_save(ctx, txn)` (hook, défaut no-op).
3. `on_save(txn)` (hook, défaut no-op) — l'écriture métier.
4. `after_save(txn)` (hook, défaut no-op).
5. Sur `Err` d'un hook → **rollback** (tracé) ; sinon → **commit**.

**Idiome Rust** : les **méthodes de trait par défaut** *sont* les primitive operations. Le trait fournit le squelette (`is_valid`/`save`) en méthode par défaut ; le formulaire concret ne surcharge que les hooks. Équivalent conceptuel du `clean()` de Django Forms.

---

## Synthèse

| Pattern | Portée | Ce qui varie |
|---------|--------|--------------|
| **Strategy** | un champ | l'algorithme `validate`/`render` (par type de champ) |
| **Composite** | le formulaire | rien — délègue + agrège uniformément |
| **Template Method** | le flux | les hooks métier (`clean`, `on_save`…), pas l'ordre |

**Tradeoff** : la validation est résolue au **runtime** (pas à la compilation) — un champ mal configuré échoue à l'exécution, pas au build. En contrepartie : ajouter un type de champ = une stratégie de plus, sans impact sur le conteneur ni le flux ; et le flux métier (ordre de validation, save transactionnel) est verrouillé par le trait, donc impossible à court-circuiter par erreur.
