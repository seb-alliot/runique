# Store & watermarks

## CleaningMemoryStore

### Pourquoi

Le `MemoryStore` de tower-sessions n'implémente pas de nettoyage des sessions expirées. Sans purge, chaque requête d'un bot sans cookies crée une session qui ne sera jamais supprimée — la mémoire croît indéfiniment.

`CleaningMemoryStore` résout ce problème avec trois mécanismes :

| Mécanisme | Déclencheur | Comportement |
|-----------|-------------|--------------|
| Timer périodique | Toutes les 60s (configurable) | Supprime toutes les sessions expirées |
| Low watermark | 128 Mo (configurable) | Purge asynchrone des sessions anonymes expirées |
| High watermark | 256 Mo (configurable) | Purge synchrone d'urgence + refus (503) si insuffisant |

### Estimation de la taille

Chaque record est estimé à : `24 octets (UUID + expiry) + taille JSON des données`.

Une alerte est loggée si un record dépasse 50 Ko (image ou fichier stocké en session par erreur).

---

## Système de watermarks

### Low watermark (128 Mo par défaut)

Lorsque la taille totale du store dépasse ce seuil, un cleanup non-bloquant est lancé en arrière-plan via `tokio::spawn`. Il supprime les sessions **anonymes expirées** sans interrompre la requête en cours.

### High watermark (256 Mo par défaut)

Lorsque la taille dépasse ce seuil au moment de créer une session :

1. **Passe 1** — supprime les sessions anonymes expirées
2. **Passe 2** — si toujours dépassé, supprime toutes les sessions expirées (y compris authentifiées)
3. **Refus** — si toujours dépassé, retourne `503 Service Unavailable`

Les sessions protégées ne sont jamais sacrifiées en passe 1.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Protection](/docs/fr/session/protection) | Protection des sessions |
| [Usage & configuration](/docs/fr/session/usage) | Accès et configuration |

## Retour au sommaire

- [Sessions](/docs/fr/session)
