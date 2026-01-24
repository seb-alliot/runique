# 📖 README Structure Guide

Guide complet de la structure des README créée pour Runique.

## 🏗️ Hiérarchie des README

```
Runique Project/
│
├── README.md (MAIN)                    ← Commencer ici !
│   └─ Index principal
│   └─ Liens vers toutes les sections
│   └─ Navigation générale
│
├── docs/
│   ├── README.md (Docs Index)
│   │   └─ Guide de navigation docs
│   │
│   ├── en/ (English)
│   │   ├── README.md (Index EN)
│   │   ├── 01-installation.md
│   │   ├── 02-architecture.md
│   │   ├── 03-configuration.md
│   │   ├── 04-routing.md
│   │   ├── 05-forms.md
│   │   ├── 06-templates.md
│   │   ├── 07-orm.md
│   │   ├── 08-middleware.md
│   │   ├── 09-flash-messages.md
│   │   └── 10-examples.md
│   │
│   └── fr/ (Français)
│       ├── README.md (Index FR)
│       ├── 01-installation.md
│       ├── 02-architecture.md
│       ├── 03-configuration.md
│       ├── 04-routing.md
│       ├── 05-forms.md
│       ├── 06-templates.md
│       ├── 07-orm.md
│       ├── 08-middleware.md
│       ├── 09-flash-messages.md
│       └── 10-examples.md
│
├── runique/
│   └── README.md (Framework Guide)
│       └─ Structure du framework
│       └─ Modules principaux
│       └─ Commandes de test
│
└── demo-app/
    └── README.md (App Guide)
        └─ Structure de l'app
        └─ Démarrage rapide
        └─ Fonctionnalités
```

## 🎯 Parcours de navigation

### Pour un nouvel utilisateur

1. **Lire** `README.md` (root)
   - Comprendre l'overview
   - Voir les features

2. **Choisir langue**
   - [English](docs/en/README.md)
   - [Français](docs/fr/README.md)

3. **Suivre l'ordre**
   - Installation
   - Architecture
   - Configuration
   - etc.

4. **Consulter exemples**
   - [Examples](docs/en/10-examples.md)

### Pour un développeur

1. **Aller à** `runique/README.md`
   - Voir la structure
   - Comprendre les modules

2. **Consulter** `demo-app/README.md`
   - Voir l'application exemple
   - Comprendre le fonctionnement

3. **Vérifier** les docs spécifiques
   - Formulaires, Routage, ORM, etc.

## 📋 Contenu par fichier

### `README.md` (Root)
- 🎯 Overview du framework
- 📚 Table des matières
- 🚀 Démarrage rapide
- 🧪 État des tests
- 🔗 Liens principaux

### `docs/README.md`
- 📖 Guide de navigation docs
- 🌍 Choix de langue
- 🎯 Navigation par sujet
- ❓ FAQ

### `docs/en/README.md` & `docs/fr/README.md`
- 📖 Index des 10 sections
- 🎯 Navigation rapide
- 🚀 Où commencer
- 💡 Conseils

### `docs/XX/01-installation.md` (tous les 01-10)
- 📝 Explications détaillées
- 💻 Exemples de code
- 🎯 Bonnes pratiques
- ⚠️ Pièges à éviter

### `runique/README.md`
- 📁 Structure du projet
- 🧪 Commandes de test
- 📦 Modules principaux
- 📚 Documentation links

### `demo-app/README.md`
- 📁 Structure de l'app
- 🚀 Démarrage
- 🎯 Fonctionnalités
- 💡 Développement

## 🔍 Recherche par sujet

### Formulaires ?
- Voir : `docs/en/05-forms.md` ou `docs/fr/05-forms.md`

### Routage ?
- Voir : `docs/en/04-routing.md` ou `docs/fr/04-routing.md`

### Base de données ?
- Voir : `docs/en/07-orm.md` ou `docs/fr/07-orm.md`

### Sécurité ?
- Voir : `docs/en/08-middleware.md` ou `docs/fr/08-middleware.md`

### Exemples ?
- Voir : `docs/en/10-examples.md` ou `docs/fr/10-examples.md`

## 🌍 Bilingue

- 🇬🇧 Tous les docs en **English**
- 🇫🇷 Tous les docs en **Français**
- 📚 Même contenu, deux langues

Choix de la langue dans les README principaux.

## 🔗 Liens internes

- Tous les README contiennent des liens
- Navigation facile entre les sections
- Accès rapide aux docs relacionadas

## 💡 Conseils de navigation

1. **Utilisez Ctrl+F** pour chercher
2. **Suivez les liens** proposés
3. **Consultez les exemples** pour du code
4. **Revisitez** régulièrement

## 📊 Vue d'ensemble

| Niveau | Fichier | Contenu |
|--------|---------|---------|
| Root | README.md | Overview principal |
| Docs | README.md | Guide docs |
| Langue | en/README.md | Index langue |
| Section | 01-10.md | Contenu détaillé |
| Framework | runique/README.md | Guide framework |
| App | demo-app/README.md | Guide app |

## ✅ Couverture

- ✅ 10 sections documentées
- ✅ Bilingue (EN & FR)
- ✅ Hiérarchie claire
- ✅ Navigation facile
- ✅ Exemples inclus

## 🚀 Démarrage

1. Ouvrir `README.md`
2. Suivre les liens
3. Consulter les docs pertinentes
4. Vérifier les exemples

---

**Prêt ?** → Ouvrir [README.md](README.md) ! 📖
