# Sessions

Runique utilise `CleaningMemoryStore` comme store de session par défaut — un wrapper autour d'un `HashMap` en mémoire qui ajoute purge automatique, protection par watermarks et protection des sessions à valeur.

Les données sont perdues au redémarrage du serveur. Pour la persistance, utilisez un store externe (Redis, base de données).

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Store & watermarks](/docs/fr/session/store) | `CleaningMemoryStore`, low/high watermarks, estimation mémoire |
| [Protection](/docs/fr/session/protection) | Protection automatique (`user_id`), manuelle (`session_active`), cas d'usage panier |
| [Usage & configuration](/docs/fr/session/usage) | Accès à la session dans les handlers, `.env`, builder |

---

## Prochaines étapes

← [**Authentification**](/docs/fr/auth) | [**Variables d'environnement**](/docs/fr/env) →
