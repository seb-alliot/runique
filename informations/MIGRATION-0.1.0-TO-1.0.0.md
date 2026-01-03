# Guide de Migration : 1.0.0

## 🎯 Vue d'ensemble

Ce guide vous aidera à migrer votre application Runique de la version 0.1.0 vers 1.0.0.

## ⚠️ Breaking Changes

### 1. Correction de sécurité dans `allowed_hosts`

**Changement :** La validation des wildcards sous-domaines a été corrigée pour éviter les vulnérabilités.

**Avant (v0.1.0) :**
```rust
// Bug : "malicious-example.com" était autorisé avec ".example.com"
let validator = AllowedHostsValidator::new(
    vec![".example.com".to_string()],
    false,
);
// ❌ "malicious-example.com" était incorrectement autorisé
```

**Après (v1.0.0) :**
```rust
// ✅ "malicious-example.com" est maintenant correctement rejeté
let validator = AllowedHostsValidator::new(
    vec![".example.com".to_string()],
    false,
);
// ✅ Seuls les vrais sous-domaines sont autorisés
```

**Action requise :** Aucune, la correction est automatique et plus sécurisée.

### 2. Utilisation de `header::HOST` au lieu de `"host"`

**Changement :** Le middleware `allowed_hosts` utilise maintenant la constante `header::HOST`.

**Avant (v0.1.0) :**
```rust
headers.get("host")  // Fonctionne mais moins explicite
```

**Après (v1.0.0) :**
```rust
headers.get(header::HOST)  // Plus explicite et type-safe
```

**Action requise :** Aucune pour les utilisateurs, changement interne.

## ✨ Nouvelles fonctionnalités

### 1. Tests d'intégration complets

Des tests d'intégration ont été ajoutés pour toutes les fonctionnalités :
- `allowed_hosts` : 9 tests
- `csrf` : 5 tests
- `csp` : 6 tests
- `routing` : 7 tests
- `forms` : 17 tests
- `sanitization` : 5 tests
- `utils` : 5 tests
- `login` : 4 tests
- `settings` : 9 tests

**Action requise :** Aucune, mais vous pouvez utiliser ces tests comme référence.

### 2. Cours d'implémentation

Des cours détaillés ont été ajoutés dans `informations/cours/` pour apprendre à implémenter chaque fonctionnalité.

**Action requise :** Consultez les cours si vous voulez comprendre l'implémentation.

## 🔧 Changements de code

### Aucun changement d'API public

L'API publique reste compatible. Aucun changement de code n'est nécessaire dans vos applications.

## 📝 Checklist de migration

- [ ] Mettre à jour `Cargo.toml` : `runique = "1.0.0"`
- [ ] Vérifier que tous vos tests passent
- [ ] Vérifier la configuration `allowed_hosts` (si vous utilisez des wildcards)
- [ ] Consulter les nouveaux tests pour voir des exemples d'utilisation
- [ ] (Optionnel) Lire les cours d'implémentation

## 🐛 Problèmes connus résolus

- ✅ Bug de sécurité dans la validation des wildcards sous-domaines
- ✅ Amélioration de l'utilisation des constantes HTTP

## 📚 Ressources

- [Documentation completed in english](informations/documentation_english/)
- [Documentation complète en francais](informations/documentation_french/)
- [Cours d'implémentation](cours/)
- [Changelog](documentation%20english/CHANGELOG.md)

## 💬 Support

Si vous rencontrez des problèmes lors de la migration, ouvrez une issue sur GitHub.

---

**Migration simple et sans risque ! 🦀**
