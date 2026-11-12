# Compatibilité

Les tableaux **Versions** et **Moteurs testés** de la page sont générés depuis `Cargo.toml` et
`docker-compose.yml`. Ce fichier ne contient que la partie écrite à la main.

## Réglages par défaut

Ce que Runique fait sans réglage de ta part. Chaque valeur se change, sur la page indiquée.

| Élément | Par défaut | Page |
|---|---|---|
| Échappement des templates (Tera) | actif sur `.html` et `.xml` ; `{{ var }}` toujours échappé, sauf avec le filtre `safe` | [Syntaxe des templates](/docs/fr/template/syntaxe) |
| CSRF | exigé pour toute méthode sauf `GET` et `HEAD` | [CSRF](/docs/fr/middleware/csrf) |
| CSP | `default-src 'none'`, `script-src 'self'`, `style-src 'self' 'unsafe-inline'` | [CSP](/docs/fr/middleware/csp) |
| Session anonyme | 5 minutes d'inactivité, en mémoire seulement | [Durées de session](/docs/fr/middleware/sessions) |
| Session connectée | 24 heures d'inactivité, aussi enregistrée en base | [Sessions](/docs/fr/session) |
| `LoginGuard` | 5 essais, puis 5 minutes de blocage | [LoginGuard](/docs/fr/auth/login-guard) |
| `RateLimiter` | 60 requêtes par minute, sur les routes où il est attaché | [Rate limit](/docs/fr/middleware/rate-limit) |
| Mots de passe | hachés avec Argon2 | [Mots de passe](/docs/fr/configuration/password) |
| Taille des requêtes | formulaire et JSON : 2 Mo lus en mémoire ; envoi de fichiers : 100 Mo (`RUNIQUE_MAX_BODY_MB`) | [Assets et médias](/docs/fr/env/assets) |

## Bonnes pratiques

- Utiliser **exactement** la version de SeaORM épinglée ci-dessus dans ton projet et dans son crate `migration/` : `runique new` l'écrit pour toi.
- Générer les migrations en développement (`runique makemigrations`), les commiter, et n'appliquer en production que `migration up`.
- Tester sur le moteur de production, pas seulement sur SQLite : certains comportements (fuseaux horaires, casse des textes, index des clés étrangères) diffèrent d'un moteur à l'autre.
- Sous MySQL et MariaDB, sauvegarder la base avant chaque `migration up` : un `CREATE` ou un `ALTER` n'y est jamais annulé.

## À éviter

- Mélanger une autre version de SeaORM ou de sea-query dans le même projet : les types ne correspondraient plus.
- Lancer `runique makemigrations` en production.
- Écrire du SQL brut là où `search!`, `search_cond!` ou les méthodes de SeaORM suffisent : le placeholder `?` casse Postgres.
- Mettre `| safe` sur une valeur qui vient d'un visiteur.
