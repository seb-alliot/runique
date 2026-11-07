# Limites connues

Ce que Runique 4.0 ne fait pas encore, ou fait avec une restriction. Chaque ligne renvoie à la page qui explique la limite et ce qu'il faut faire en attendant. La politique de sécurité et les failles corrigées sont décrites dans [`SECURITY.fr.md`](https://github.com/seb-alliot/runique/blob/main/SECURITY.fr.md).

---

## Base de données et migrations

| Limite | Détail |
| --- | --- |
| Un moteur par compilation (`sqlite`, `postgres`, `mysql`)<br>SQLite : un seul écrivain à la fois, `SQLITE_BUSY` sous forte charge<br>Changer de moteur demande une base neuve : les données ne sont pas transférées | [Base de données](/docs/fr/installation/base-de-donnees) |
| Le changement de type d'une colonne n'est pas migré (`--force` écrit une migration vide, à compléter à la main)<br>Des snapshots copiés sans leurs migrations font croire que les tables existent<br>Identifiants SQL limités à 64 caractères sous MySQL / MariaDB<br>`migration up` : une migration n'est atomique que sous Postgres (SQLite, MySQL / MariaDB : une migration qui échoue peut rester à moitié appliquée)<br>Un arrêt brutal pendant `makemigrations` peut laisser des fichiers à moitié écrits (`git` pour restaurer) | [Migrations](/docs/fr/installation/migrations), [Makemigrations](/docs/fr/architecture/makemigrations) |
| `search!` : `in (vec)` avec un vec vide, pas de filtre conditionnel inline | [Requêtes](/docs/fr/orm/requetes) |

## Formulaires et messages

| Limite | Détail |
| --- | --- |
| Un échec d'écriture en session perd le message flash sans erreur | [Macros flash](/docs/fr/flash/macros) |
| Un fichier publié puis une écriture en base qui échoue brutalement (coupure) peuvent laisser un fichier orphelin | [Champs](/docs/fr/formulaire/champs) |

## Sécurité, sessions, authentification

| Limite | Détail |
| --- | --- |
| `RateLimiter` et `LoginGuard` n'ont pas de plafond mémoire<br>`RateLimiter` et `LoginGuard` comptent par processus : avec plusieurs instances, le quota est multiplié | [Rate limiting](/docs/fr/middleware/rate-limit) |
| `LoginGuard` bloque par nom d'utilisateur : quelqu'un peut bloquer un compte dont il connaît le nom | [LoginGuard](/docs/fr/auth/login-guard) |
| Le store de sessions mémoire appartient à un processus | [Store de sessions](/docs/fr/session/store) |
| Un store personnalisé (`with_session_store`) ne peut pas être parcouru par utilisateur : un reset de mot de passe ne ferme pas ses sessions | [Session](/docs/fr/auth/session) |
| Durée de session : une seule pour tous les comptes connectés ; « se souvenir de moi » (durée par session) n'est pas encore possible | [Sessions](/docs/fr/middleware/sessions) |
| Au plus 5 middlewares personnalisés (le 6ᵉ fait paniquer le démarrage) ; ils tournent avant la session, sans accès au compte connecté | [Stack middleware](/docs/fr/architecture/middleware) |
| Mode `Delegated` : aucun fournisseur externe (OAuth, LDAP, SAML) n'est implémenté | [Mots de passe](/docs/fr/configuration/password) |
| ACME : un seul site par machine (port 80 exclusif) | [Réseau](/docs/fr/installation/network) |

## Admin

| Limite | Détail |
| --- | --- |
| `.links`, `.search_help`, `.empty_value` acceptés mais sans effet sur l'affichage | [Builder ModelAdmin](/docs/fr/admin/declaration-macro) |
| Pas de feuille de thème globale (contournement par un bloc)<br>`.with_htmx` est global, pas de fragment de liste par modèle<br>En production, un template de page absent n'est signalé qu'à l'affichage (refus du démarrage en dev ou avec `DEBUG=true`) ; une feuille de style absente ne l'est jamais | [Surcharge](/docs/fr/admin/template-surcharge) |
| Pas d'interface de gestion des sessions | — |
| Many-to-many : déclaré dans le DSL par une table pivot, sans gestion dans l'admin | [Évolutions](/docs/fr/admin/evolution) |

## Développement local

| Limite | Détail |
| --- | --- |
| Les cookies ignorent le port : deux projets sur `127.0.0.1` partagent (et s'écrasent) le cookie de session | [Dépannage](/docs/fr/installation/troubleshooting) |

---

## Solutions en attendant

| Limite | Solution |
| --- | --- |
| `RateLimiter` et `LoginGuard` sans plafond mémoire, compteurs par processus | Limiter aussi au reverse proxy (exemple ci-dessous) |
| `LoginGuard` bloque par nom d'utilisateur | L'associer à une limite par IP, au reverse proxy ou avec `RateLimiter` |
| Store de sessions mémoire propre à un processus | Une seule instance, ou un reverse proxy qui renvoie chaque visiteur vers la même (`ip_hash` avec nginx) |
| ACME : un seul site par machine | Confier le TLS à un reverse proxy dès qu'il y a plusieurs sites |
| SQLite : un seul écrivain à la fois | PostgreSQL pour une charge en écriture soutenue |
| Migration atomique seulement sous PostgreSQL | Sauvegarder la base avant `migration up` sur SQLite et MySQL / MariaDB |
| Changement de type de colonne non migré | Compléter à la main la migration écrite par `--force` |
| Cookies communs à deux projets sur `127.0.0.1` | Ouvrir l'un sur `127.0.0.1`, l'autre sur `localhost` |

Limite par IP avec nginx, à adapter. La mémoire est plafonnée par la zone, et une requête refusée reçoit un code 503 par défaut :

```nginx
limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;

location /login {
    limit_req zone=login burst=5 nodelay;
    proxy_pass http://127.0.0.1:3000;
}
```

---

## Retour au sommaire

- [Sommaire](https://github.com/seb-alliot/runique/blob/main/README.fr.md)
