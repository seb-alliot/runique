# 📚 Cours Rusti Framework - Guide d'implémentation

Ce dossier contient des cours détaillés pour apprendre à implémenter toutes les fonctionnalités du framework Rusti.

## 📖 Structure des cours

### 1. [Routing et Reverse Routing](01-routing.md)
- Créer un système de routing nommé
- Implémenter `reverse()` et `reverse_with_parameters()`
- Créer la macro `urlpatterns!`

### 2. [Système de Formulaires](02-forms.md)
- Créer un système de validation de formulaires
- Implémenter les champs (CharField, IntegerField, EmailField)
- Gérer les erreurs et les données nettoyées

### 3. [Middleware de Sécurité](03-middleware-securite.md)
- CSRF Protection
- Content Security Policy (CSP)
- Validation des hosts autorisés
- Sanitization des entrées

### 4. [Utilitaires](04-utils.md)
- Génération de tokens sécurisés
- Hashing et cryptographie

### 5. [Configuration et Settings](05-settings.md)
- Système de configuration flexible
- Builder pattern
- Variables d'environnement

### 6. [Authentification](06-authentication.md)
- Middleware de login requis
- Gestion des sessions
- Redirections conditionnelles

## 🎯 Comment utiliser ces cours

1. **Lisez dans l'ordre** : Les cours sont organisés du plus simple au plus complexe
2. **Pratiquez** : Chaque cours contient des exercices
3. **Testez** : Utilisez les tests existants pour valider vos implémentations
4. **Adaptez** : Personnalisez selon vos besoins

## 📝 Prérequis

- Connaissance de base de Rust
- Compréhension des concepts async/await
- Familiarité avec Axum (recommandé)

## 🚀 Démarrage rapide

Commencez par le cours sur le Routing, c'est la base de tout framework web.
