# Limites connues

Ce que Runique 4.0 ne fait pas encore, ou fait avec une restriction. Chaque ligne renvoie à la page qui explique la limite et ce qu'il faut faire en attendant. Les constats de sécurité ouverts sont tenus à part, dans `faille.md` à la racine du dépôt.

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
| Plusieurs formulaires sur une page : `request.method.is_safe()` ne dit pas lequel a été envoyé (le message global s'afficherait sous tous) | [Trait RuniqueForm](/docs/fr/formulaire/trait) |
| Un échec d'écriture en session perd le message flash sans erreur | [Macros flash](/docs/fr/flash/macros) |
| Un fichier publié puis une écriture en base qui échoue brutalement (coupure) peuvent laisser un fichier orphelin | [Champs](/docs/fr/formulaire/champs) |

## Sécurité, sessions, authentification

| Limite | Détail |
| --- | --- |
| `RateLimiter` et `LoginGuard` n'ont pas de plafond mémoire<br>`RateLimiter` et `LoginGuard` comptent par processus : avec plusieurs instances, le quota est multiplié | [Rate limiting](/docs/fr/middleware/rate-limit) |
| `LoginGuard` bloque par nom d'utilisateur : quelqu'un peut bloquer un compte dont il connaît le nom | [LoginGuard](/docs/fr/auth/login-guard) |
| Le store de sessions mémoire appartient à un processus | [Store de sessions](/docs/fr/session/store) |
| Un store personnalisé (`with_session_store`) ne peut pas être parcouru par utilisateur : un reset de mot de passe ne ferme pas ses sessions | [Session](/docs/fr/auth/session) |
| Mode `Delegated` : aucun fournisseur externe (OAuth, LDAP, SAML) n'est implémenté | [Mots de passe](/docs/fr/configuration/password) |
| ACME : un seul site par machine (port 80 exclusif) | [Réseau](/docs/fr/installation/network) |

## Admin

| Limite | Détail |
| --- | --- |
| `.links`, `.search_help`, `.empty_value` acceptés mais sans effet sur l'affichage | [Builder ModelAdmin](/docs/fr/admin/declaration-macro) |
| Pas de feuille de thème globale (contournement par un bloc)<br>`.with_htmx` est global, pas de fragment de liste par modèle<br>Un template de page ou une feuille de style absents ne sont signalés qu'à l'affichage | [Surcharge](/docs/fr/admin/template-surcharge) |
| Pas d'interface de gestion des sessions | — |
| Pas de many-to-many dans le DSL ni dans l'admin | [Évolutions](/docs/fr/admin/evolution) |

## Développement local

| Limite | Détail |
| --- | --- |
| Les cookies ignorent le port : deux projets sur `127.0.0.1` partagent (et s'écrasent) le cookie de session | [Dépannage](/docs/fr/installation/troubleshooting) |

---

## Retour au sommaire

- [Sommaire](https://github.com/seb-alliot/runique/blob/main/README.fr.md)
