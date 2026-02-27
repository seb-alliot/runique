# Résumé des modifications à apporter

## 1. Configuration du mot de passe via settings

**Status :** 🔴 À faire

### 1.a. Ajouter un enum pour `main.rs`
Contenant les options :
- `False`
- `Auto` (Argon2 | Bcrypt | autre)
- Choix manuel

### 1.b. Logique selon l'option choisie

| Option | Comportement |
|--------|-------------|
| `False` | Aucun hashage de mot de passe. Le développeur peut utiliser une API tierce |
| **Choix manuel** | La fonction de hash récupère l'option de configuration et match dessus |
| **Auto hashage** | Si option choisie (ex: `auto::auto::argon2`), hash automatique du champ `password` |
| **Vérification** | Fonction de vérification du mot de passe basée sur l'option de hash :&lt;br&gt;- Si Argon2 → vérif basée sur Argon2&lt;br&gt;- Si Bcrypt → vérif basée sur Bcrypt&lt;br&gt;- Si mode auto → choix de l'algorithme |

---

## 2. I18n et Tracing

**Status :** 🔴 À faire

### 2.a. I18n (Internationalisation)
- Configuration dans `main.rs` via un choix de langue
- Exemple : `let lang = config.language(enum possible)`
- Charge un fichier JSON correspondant

### 2.b. Tracing d'erreur
- Configurer le tracing sur le mode debug

| Debug | Tracing |
|-------|---------|
| `false` | Off |
| `true` | On → rendu console et page de debug |

---

## 3. Migration et Vue Admin

**Status :** 🔴 À faire (3.a à corriger)

### 3.a. Système de migration
- Finaliser le système de migration
- Une fois terminé : API stable pour schema/migration
- **Équivalent Django :** `models/admin`

### 3.b. Vue Admin

#### b.1. Refonte du rendu
- Ne plus baser le rendu admin sur les formulaires bruts
- Basculer sur les **models** qui gèrent leur propre rendu
- Les formulaires se basent sur le model (et non l'inverse) si macro attribut connecté

#### b.2. Formulaires personnalisés
- Permettre l'ajout de formulaires pour récupérer la logique métier de l'API sur les models

---

## 4. Middleware CSP et Stabilité

**Status :** 🔴 À faire

### 4.a. Middleware CSP
- Peaufiner la configuration pour la rendre plus simple et lisible

### 4.b. Stabilité

| Tâche | Description |
|-------|-------------|
| **b.1** | Vérifier toutes les features, tests exhaustifs sur toutes les features |
| **b.2** | Mettre à mal le framework, le pousser à bout pour la fiabilité |
| **b.3** | Trouver des failles pour les corriger |
| **b.4** | Ne pas considérer le framework comme terminé, il y a toujours à ajouter |

---

## 5. Moteurs de formulaire

**Status :** 🟢 Fait

### 5.a. Peaufiner/corriger les moteurs de formulaire
- **Problème identifié :** Double appel de `is_valid()`
  - 1ère fois dans `build_with_data`
  - 2ème fois dans `is_valid` du handler du développeur



  Restructure la gestion des mots de passe


# flow ashe du mot de passe


### 6 Admin

- **Personalisation** des templates
    - 1) Personnalisation visuel
    - 2) Documentation => clef a renseigner fournis dans le templates

enum => delagate , auto , manuel

logique enum =>

delagate => api => exemple -> google authenticator

auto => ashage du mot de passe automatique dans TexeField==SpecialFormat::password en fonction de la config , appel automatique a ashe_password.

manuel => choix de l'algo , appel manuel a ash_password dans la logique metier


donc =>

    dalagate => code perso du dev dans son utisl par exemple
    auto => enum appel ashe dans password
    manuel => desactive hash dans passord et force l'appel dans la logique metier


exemple

mains.rs

let ashe = config.password(Delegate | auto::algo | manuel::algo )

mise en pratique

        form.field(
            &TextField::password("password")
                .label("Entrez votre mot de passe")
                .required()
                .min_length(10, "Le titre doit contenir au moins 10 caractères"),
        );

fn clean_field (
    let pw1 = get...
    let pw2 = get..
    if pw1 == pw2
    pw1 = ash.pw1
)

et dans TexteField::SpecialFormat::password

    password = config.password


field s'auto valide => clean_field se verifie  => clean regroupe tout => finalyze transforme => validate => persistence

a quel moment injecter config.password ?

=> field password => aucune verification, ne renvois qu'un champs apssword
=> clean_field => contrainte metier
=> clean => regroupement logique metier global
=> finalyse => transformation
=> validate => regroupe tout avant persistance
=> save => persistence
