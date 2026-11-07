🌍 **Langues** : [English](SECURITY.md) | [Français](SECURITY.fr.md)

# Politique de sécurité

## Versions prises en charge

| Version | Prise en charge |
| ------- | --------------- |
| 4.0.x   | Oui (à venir, pas encore publiée) |
| 3.0.x   | Oui (utilisez la **3.0.3** ou une version ultérieure) |
| < 3.0   | Non |

Runique 4.0 conserve les correctifs de sécurité de la 3.0.x et en ajoute d'autres (listés ci-dessous). Runique 3.0.2 a corrigé des problèmes présents aussi en 2.2.x.

## Failles de sécurité connues

### Failles en cours (corrigées en 4.0)

#### `extend!{}` de la table des comptes met `password` et `is_active` dans son formulaire d'admin — Élevée, corrigée en 4.0

- **Ce que ça fait** : le formulaire généré pour un `extend!` de `eihwaz_users` (un profil utilisateur, par exemple) reprend ces deux colonnes du compte. Un admin peut alors choisir le mot de passe d'un utilisateur à la création ou à l'édition, et donc le connaître, et désactiver n'importe quel compte, le sien compris, sans passer par les règles des comptes.
- **Qui est concerné** : les projets Runique 3.0.x qui enregistrent un tel `extend!` dans `admin!{}`.
- **Comment se protéger dès maintenant** : gardez les comptes étendus hors de `admin!{}`, et gérez ces colonnes dans vos propres vues.
- **Correctif** : Runique 4.0 ne met jamais ces colonnes dans un formulaire, comme les rangs. Un nouveau compte reçoit un mot de passe aléatoire que personne ne connaît (son propriétaire choisit le sien via le lien reçu par e-mail), et une édition n'y touche pas, quoi que contienne la requête.

#### `csrf.js` envoie le jeton CSRF à d'autres origines — Moyenne, corrigée en 4.0

- **Ce que ça fait** : `csrf.js` enveloppe `fetch` pour ajouter l'en-tête `X-CSRF-Token` à chaque requête `POST`/`PUT`/`PATCH`/`DELETE`, sans regarder sa destination. Un `fetch` vers un autre domaine, lancé depuis une page qui charge le script (script tiers ou code du projet), transmet le jeton CSRF de la session à ce domaine, et la réponse de ce domaine peut remplacer le jeton de la page. Avec ce jeton, un autre site peut forger des requêtes au nom de l'utilisateur connecté.
- **Qui est concerné** : Runique 3.0.x, uniquement si le navigateur laisse la requête quitter la page. La CSP de Runique est active par défaut et tous ses préréglages définissent `connect-src 'self'` : le navigateur bloque alors tout `fetch` vers une autre origine avant son envoi, et le jeton ne sort pas. Sont exposés : un projet qui ajoute une autre origine à `connect-src` (le jeton atteint cette origine), ou qui désactive la CSP.
- **Comment se protéger dès maintenant** : gardez le `connect-src 'self'` par défaut. Si la page doit appeler une autre origine, faites-le depuis le serveur plutôt que depuis le navigateur, ou ne chargez pas `csrf.js` sur cette page.
- **Correctif** : Runique 4.0 n'ajoute l'en-tête, et n'accepte un jeton renouvelé, que pour les requêtes vers l'origine de la page.

#### Erreurs globales de formulaire affichées sans échappement (XSS) — Moyenne, corrigée en 4.0

- **Ce que ça fait** : les erreurs globales d'un formulaire (une erreur levée par `clean()`, une erreur de base de données citant la valeur envoyée) arrivent dans la page sans échappement. Une valeur saisie par l'utilisateur et répétée par l'erreur peut contenir du HTML ou un script.
- **Qui est concerné** : les formulaires Runique 3.0.x dont les erreurs globales citent ce que l'utilisateur a envoyé.
- **Comment se protéger dès maintenant** : ne répétez pas les valeurs envoyées dans les erreurs globales de `clean()`, et gardez la CSP active (elle l'est par défaut), qui bloque un script inline injecté.
- **Correctif** : Runique 4.0 affiche les erreurs globales via un template, sous l'échappement automatique de Tera.

#### La page d'erreur de debug peut afficher un fichier hors de `templates/` — Moyenne, corrigée en 4.0

- **Ce que ça fait** : sur une erreur de rendu de template, la page de debug lit et affiche `templates/{nom}` sans contrôler le nom. Un template choisi à partir des données de la requête (un slug) permet d'afficher `../../.env`, avec les secrets du projet.
- **Qui est concerné** : les déploiements Runique 3.0.x accessibles avec `DEBUG=true`. Avec `DEBUG=false`, cette page n'existe pas.
- **Comment se protéger dès maintenant** : n'exposez jamais un déploiement avec `DEBUG=true`, et ne construisez pas de noms de template à partir des données de la requête.
- **Correctif** : Runique 4.0 ne lit qu'un chemin relatif ordinaire sous `templates/` ; `..` et les chemins absolus sont refusés.

#### Contenu des sessions lisible dans la base de données — Moyenne, corrigée en 4.0

- **Ce que ça fait** : `eihwaz_sessions.session_data` contient en clair le contenu des sessions connectées, y compris le jeton CSRF et tout ce que l'application y stocke. Lire la base (une sauvegarde, une injection SQL) donne accès à tout cela.
- **Qui est concerné** : les projets Runique 3.0.x qui utilisent le store de sessions par défaut.
- **Comment se protéger dès maintenant** : limitez l'accès à la base et à ses sauvegardes, et gardez les valeurs sensibles hors de la session.
- **Correctif** : Runique 4.0 chiffre le contenu (XChaCha20-Poly1305, clé dérivée de `SECRET_KEY`, liée à la ligne). Une ligne en clair, modifiée ou copiée est refusée. `user_id` et `expires_at` restent en clair, et quiconque peut aussi lire `SECRET_KEY` peut déchiffrer. La mise à jour vide la table : les utilisateurs se reconnectent une fois.

#### Un formulaire envoyé depuis un autre site atteint les handlers qui ne vérifient pas le jeton — Moyenne, corrigée en 4.0

- **Ce que ça fait** : un handler qui lit son corps avec un extracteur Axum (`Form<T>`, `Multipart`) au lieu de `request.form("nom")` ne vérifie pas le jeton CSRF, et le middleware laisse passer un formulaire sans en-tête `X-CSRF-Token`. Un autre site peut alors envoyer ce formulaire au nom d'un utilisateur connecté.
- **Qui est concerné** : les projets Runique 3.0.x qui ont un tel handler sur une route qui écrit. Les formulaires passant par `request.form` ne sont pas concernés : leur jeton est vérifié.
- **Comment se protéger dès maintenant** : vérifiez le jeton dans ces handlers, ou passez-les à `request.form`.
- **Correctif** : Runique 4.0 refuse (403), avant le handler, un formulaire que le navigateur annonce comme envoyé depuis un autre site ou un sous-domaine (`Sec-Fetch-Site`). Les navigateurs n'envoient cet en-tête qu'en HTTPS et sur localhost : un handler utilisant un extracteur doit donc toujours vérifier le jeton. Un chemin dans `csrf_exempt()` ou une requête avec `X-CSRF-Token` n'est pas concerné.

#### Le message de refus de l'admin révèle qu'un compte est superutilisateur — Faible, corrigée en 4.0

- **Ce que ça fait** : lorsqu'une action sur le compte d'un superutilisateur lui est refusée, un membre du staff lit « Seul un superutilisateur peut modifier le compte d'un superutilisateur », ce qui lui indique quels comptes sont superutilisateurs.
- **Qui est concerné** : les admins Runique 3.0.x avec des membres du staff.
- **Comment se protéger dès maintenant** : rien à faire ; le refus lui-même est correct, seule sa formulation en dit trop.
- **Correctif** : Runique 4.0 affiche le message générique « Action non autorisée » (9 langues).

#### Le temps de réponse de « Mot de passe oublié » révèle si un compte existe — Faible, corrigée en 4.0

- **Ce que ça fait** : la page « mot de passe oublié » répond le même message à toutes les adresses, mais elle n'écrit le jeton de réinitialisation, et n'attend cette écriture, que si l'adresse correspond à un compte. La page est plus lente de quelques millisecondes pour un compte existant, ce qui révèle à qui la chronomètre si une adresse a un compte.
- **Qui est concerné** : les projets Runique 3.0.x qui appellent `with_password_reset()`.
- **Comment se protéger dès maintenant** : gardez la limite de débit de la page (5 requêtes par 5 minutes et par IP par défaut, `max_requests` / `retry_after`) : elle réduit le nombre d'adresses qu'un client peut essayer.
- **Correctif** : Runique 4.0 exécute la recherche du compte, le jeton et l'e-mail en tâche de fond : la page répond dans le même temps pour toutes les adresses.

#### Dépendances

Aucune vulnérabilité connue. L'arbre des dépendances a été vérifié par rapport aux deux avis suivis auparavant ici (ci-dessous) : aucun ne s'applique. `cargo audit` signale aussi deux crates non maintenues, ce qui n'est pas une vulnérabilité : `paste` (RUSTSEC-2024-0436) et `rustls-pemfile` (RUSTSEC-2025-0134).

### Suivies auparavant (résolues)

#### RUSTSEC-2023-0071 : RSA Marvin Attack — ne s'applique plus
- **Concernait via** : `rsa` (dépendance transitive d'une ancienne version de `sqlx-mysql`)
- **État actuel** : `rsa` n'apparaît nulle part dans `Cargo.lock` ; `sqlx-mysql` (désormais `0.9.0`) ne l'utilise plus.

#### RUSTSEC-2025-0052 : async-std non maintenu — ne s'applique plus
- **Concernait via** : `async-std` (dépendance transitive de `sea-orm`/`sqlx`)
- **État actuel** : `async-std` n'apparaît nulle part dans `Cargo.lock` ; la pile SeaORM/sqlx utilisée par Runique est passée entièrement à Tokio.

Vérifié à nouveau sur un `Cargo.lock` local le 2026-11-07 ; ce fichier n'est pas commité, donc vos versions résolues peuvent différer. La CI lance `cargo audit` à chaque push, sans avis ignoré, et une vulnérabilité connue fait échouer le build. Si vous maintenez un fork avec d'autres versions de dépendances, vérifiez avec `cargo audit` avant de vous fier à cette section.

## Corrigé dans Runique

| Version | Problème | Gravité |
| ------- | -------- | ------- |
| 4.0 | `extend!{}` de la table des comptes mettant `password` et `is_active` dans son formulaire d'admin | Élevée |
| 4.0 | Jeton CSRF envoyé par `csrf.js` avec les requêtes `fetch` vers d'autres origines | Moyenne |
| 4.0 | Erreurs globales de formulaire affichées sans échappement (XSS) | Moyenne |
| 4.0 | Page d'erreur de debug pouvant afficher un fichier hors de `templates/` | Moyenne |
| 4.0 | Contenu des sessions stocké en clair dans la base | Moyenne |
| 4.0 | Réinitialisation du mot de passe laissant valides les autres sessions d'un store personnalisé | Moyenne |
| 4.0 | Formulaire envoyé depuis un autre site atteignant un handler qui ne vérifie pas le jeton | Moyenne |
| 4.0 | Fichiers validés avant d'être entièrement écrits : un fichier au-delà de la limite d'un champ pouvait passer | Moyenne |
| 4.0 | Une erreur de suppression en base pouvait laisser une session réutilisable ; révocation des sessions hors de la transaction du reset | Moyenne |
| 4.0 | Remplacement non atomique du jeton de reset : deux jetons valides en concurrence | Moyenne |
| 4.0 | Message de refus de l'admin révélant qu'un compte est superutilisateur | Faible |
| 4.0 | Temps de réponse de « Mot de passe oublié » révélant si un compte existe | Faible |
| 4.0 | Compte créé sans son activation dans un projet généré par `runique new` | Faible |
| 4.0 | Hash au préfixe non reconnu : vérification plus rapide, donc mesurable | Faible |
| 4.0 | Jeton CSRF en double dans un corps de formulaire (partie corrigée) | Faible |
| 4.0 | Alias d'enum chaîne ambigus interprétés comme une autre variante | Faible |
| 4.0 | Scripts `add_js` sans hash SRI ni nonce CSP | Durcissement |
| 3.0.3 | `bulk_create` : le seul droit de création permettait de modifier des lignes existantes (régénérez l'admin) | Élevée |
| 3.0.3 | Un membre du staff pouvait prendre le compte d'un superutilisateur (changement d'e-mail + réinitialisation) | Élevée |
| 3.0.3 | Open redirect via une tabulation (`/\t/evil.com`) ou un `Location` non ASCII non contrôlé | Moyenne |
| 3.0.3 | Open redirect via l'en-tête `Refresh`, jamais contrôlé | Moyenne |
| 3.0.2 | XSS stockée via un fichier en attente de son formulaire (chemin réaffiché, dossier de staging servi sans CSP) | Élevée |
| 3.0.2 | Fichiers envoyés écrits sur disque avant la vérification CSRF (multipart) | Moyenne |
| 3.0.2 | Open redirect via des identifiants dans une URL (`allowed.com:x@evil.com`) | Moyenne |
| 3.0.2 | Corps de requête urlencoded / JSON sans limite de taille | Moyenne |
| 3.0.2 | Réinitialisation de mot de passe de l'admin accessible depuis une ressource qui n'est pas un compte | Faible |
| 3.0.2 | Admin acceptant le jeton CSRF brut (masquage par réponse contourné) | Durcissement |
| 2.1.17 | Tokens de reset : mémoire → DB (hashés, single-use, durcis IDOR) | Durcissement |
| 2.1.15 | Énumération d'utilisateurs via timing attack au login | Moyenne |
| 2.1.15 | Contrôle d'accès manquant sur l'action admin reset-password | Moyenne |
| 2.1.9 | Injection SQL dans les filtres de liste admin | Élevée |
| 2.1.9 | Fixation de session au login (cycle_id manquant) | Moyenne |
| 2.1.9 | Granularité droits write admin (create/update/delete indistincts) | Moyenne |
| 2.1.9 | IDOR — can_update_own/can_delete_own non appliqués | Faible |

Les principaux correctifs de la 4.0 sont décrits plus haut, dans « Failles en cours (corrigées en 4.0) ». Détails dans le CHANGELOG : [3.0.3](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md#303---2026-10-10), [3.0.2](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md#302---2026-10-09).

## Signaler une vulnérabilité

Si vous découvrez une vulnérabilité dans Runique lui-même (pas dans ses dépendances) :

1. **N'ouvrez pas** d'issue publique
2. Écrivez à : alliotsebastien04@gmail.com
3. Indiquez :
   - la description de la vulnérabilité
   - les étapes pour la reproduire
   - l'impact possible
   - un correctif suggéré (le cas échéant)

Nous répondrons sous 48 heures et travaillerons à un correctif dès que possible.

## Bonnes pratiques de sécurité

Pour utiliser Runique en production :

1. **Utilisez toujours HTTPS** (`ENFORCE_HTTPS=true` derrière un proxy TLS, ou ACME) : cela active aussi HSTS
2. **Lancez avec `RUNIQUE_ENV=prod` et `DEBUG=false`** : les contrôles au démarrage s'appliquent et les pages d'erreur ne montrent aucun détail interne
3. **Définissez une `SECRET_KEY` forte** (32 caractères aléatoires ou plus) : elle chiffre aussi les sessions sauvegardées en base
4. **Activez la validation des hôtes** dans le builder : `.middleware(|m| m.with_allowed_hosts(|h| h.host("mysite.com")))`
5. **Utilisez le préréglage CSP strict** (`.with_csp(|c| c.policy(SecurityPolicy::strict()))`) : la CSP elle-même est toujours active par défaut
6. **Protégez la connexion à l'admin contre la force brute** : `.with_admin(|a| a.with_login_guard(LoginGuard::new()))`, à activer vous-même
7. **Dimensionnez explicitement les envois de fichiers** : `RUNIQUE_MAX_UPLOAD_MB` (sans valeur, une requête est limitée à 2 Mo)
8. **Dans un formulaire d'envoi écrit à la main, placez `{% csrf %}` avant les champs de fichier** : aucun fichier n'est écrit sur disque avant un jeton valide
9. **Gardez les dépendances à jour** : `cargo update`
10. **Lancez des audits de sécurité** : `cargo audit`

Le nettoyage du HTML en sortie (via `ammonia`) et l'échappement automatique de Tera sont actifs par défaut : il n'y a pas d'option `sanitize_inputs` à définir, ce comportement n'a pas à être activé.

## Calendrier de divulgation des vulnérabilités

- **Jour 0** : vulnérabilité signalée
- **Jours 1-2** : accusé de réception et première évaluation
- **Jours 3-7** : développement et test du correctif
- **Jours 7-14** : préparation de la version et de l'avis de sécurité
- **Jour 14 et après** : divulgation publique une fois le correctif disponible
