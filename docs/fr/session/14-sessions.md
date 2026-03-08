# Sessions

Runique utilise `CleaningMemoryStore` comme store de session par défaut — un wrapper autour d'un `HashMap` en mémoire qui ajoute purge automatique, protection par watermarks et protection des sessions à valeur.

Les données sont perdues au redémarrage du serveur. Pour la persistance, utilisez un store externe (Redis, base de données).

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Store & watermarks](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/store/store.md) | `CleaningMemoryStore`, low/high watermarks, estimation mémoire |
| [Protection](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/protection/protection.md) | Protection automatique (`user_id`), manuelle (`session_active`), cas d'usage panier |
| [Usage & configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/usage/usage.md) | Accès à la session dans les handlers, `.env`, builder |

---

## Prochaines étapes

← [**Authentification**](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md) | [**Variables d'environnement**](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md) →
