# Ébauche — réinitialisation du mot de passe et second facteur

> Statut (2026-10-16) : **idée de conception, rien n'est implémenté, aucune version prévue.** Indépendante de la refonte de l'admin et des hooks : les deux peuvent venir avant ou après.

## Ce qui existe aujourd'hui (vérifié dans le code)

| Point | État |
| --- | --- |
| Réinitialisation | Met à jour le mot de passe en base, **ne connecte pas** |
| Sessions | Seule une réinitialisation **terminée** ferme toutes les sessions du compte (`tests/auth/test_reset_closes_sessions.rs`), pas la simple demande |
| `LoginGuard` | En mémoire, un seul palier (5 essais / 300 s), opt-in sur la connexion admin |
| Accès à l'admin | Le middleware vérifie `can_access_admin()` sur la session ; la connexion du site (`auth::session::login`) ouvre la **même** session |

## Principe : Runique possède le parcours, le dev fournit le canal

| Runique (non surchargeable) | Le dev |
| --- | --- |
| Ordre des étapes, jetons et codes (hash, expiration, essais, usage unique), réponses uniformes, moment où partent les notifications | Le transport SMS (`SmsBackend`, comme le mailer), la colonne du numéro, les textes (templates Tera) |

Si le dev peut réécrire la logique, il peut rouvrir sans le voir les failles relevées ci-dessous. Les réactions en plus (Slack, journal du projet) passeront par les futurs hooks : Runique émet « mot de passe changé », le projet s'abonne. Ajout non cassant, pas un prérequis.

## Le numéro de téléphone

- Une colonne ajoutée à `eihwaz_users` par `extend!` (ex. `user_profile::Column::Phone` dans demo-app), déclarée dans le builder de réinitialisation :

```rust
.phone(user_profile::Column::Phone)
```

- Même table que le compte : une seule requête, et la règle des rangs protège déjà ses écritures.
- Runique ne stocke ni ne modifie jamais le numéro. Son format est validé par le dev à l'écriture, stocké en `String`.
- **Jamais saisi dans le formulaire au moment de la réinitialisation** : l'attaquant taperait le sien.
- Numéro déclaré sans `SmsBackend` configuré : erreur au démarrage.

## Parcours

1. L'utilisateur demande la réinitialisation. Réponse toujours identique : « Si ce compte existe, un email a été envoyé ». Email envoyé en tâche de fond (même temps de réponse).
2. Il clique sur le lien et saisit le nouveau mot de passe, mis **en attente**, jamais enregistré à ce stade.
3. Un code est généré **à ce moment-là** :
   - compte sans numéro : code par email ;
   - compte avec numéro : code par SMS ;
   - SMS en échec : **repli sur l'email** (un utilisateur ne doit pas être pénalisé par une panne du fournisseur) ;
   - les deux en échec : « Une erreur est survenue, contactez le support ».
4. Il saisit le code : le mot de passe est enregistré, le jeton du lien devient inutilisable, toutes les sessions sont fermées.
5. Email de confirmation (« Votre mot de passe a été changé »). **Si le repli a eu lieu, le message le signale.**

Le code n'est **jamais affiché à l'écran** : celui qui est devant l'écran le recopierait, et un lien qui a fuité suffirait.

Le SMS ne part qu'après le clic sur le lien : déclencher un SMS demande déjà la boîte mail, donc pas de *SMS pumping* (épuisement du crédit) possible.

Tous les échecs d'envoi sont tracés, jamais avalés.

## Stockage des codes

Une table dédiée, sur le modèle du jeton de réinitialisation (ni colonne dans `eihwaz_users`, ni cookie) :

| Champ | Rôle |
| --- | --- |
| `user_id` | Le compte |
| hash du code | Jamais en clair : une fuite de la base ne donne aucun code valide |
| expiration | Courte (10 min) |
| essais | Quelques essais seulement, puis le code est détruit |
| envoyé le | Délai avant un nouveau code |

Comparaison du code en temps constant (`ct_eq`). Jeton inconnu, expiré ou déjà utilisé : même traitement qu'un jeton valide jusqu'à la réponse (comparaison contre un hash factice), même message.

## Délais et blocage

- **Entre deux codes** : aucun nouveau code pendant la validité du précédent, **en silence** (même réponse, le code précédent reste valide). Un message « attendez » révélerait que le mot de passe ou le compte est bon.
- **Blocage progressif de la réinitialisation**, façon Square Enix : 3 essais → 5 min → 1 h → 24 h. Il vise la **réinitialisation, pas la connexion** : le dirigeant qui connaît son mot de passe se connecte toujours.
- Réponse identique pendant le blocage.
- Paliers stockés **en base** : `LoginGuard` en mémoire repart à zéro à chaque redémarrage et n'est pas partagé entre instances.
- Valeurs par défaut sûres, configurables, avec un minimum imposé au démarrage.

## Connexion à l'admin avec second facteur (idée liée)

Un enum de mode de connexion sur le builder de l'admin : classique (username + mot de passe), code par email, plus tard TOTP. Vaut pour superuser, admin et staff.

Parcours retenu :
1. Username seul → code envoyé. Réponse et temps identiques que le compte existe ou non.
2. Code + mot de passe vérifiés **ensemble**, erreur uniforme (« code ou mot de passe invalide »).
   - Tout échec compte un essai, quel que soit le facteur.
   - Le hash du mot de passe est toujours calculé, même si le code est faux.
   - **Valeurs factices** quand il n'y a rien à comparer, pour que chaque branche fasse le même travail : compte inconnu → mot de passe vérifié contre `dummy_hash()` (existe déjà, `authenticate_user`) ; pas de code en attente → code comparé à un hash factice, avec le même `ct_eq`. Jamais de retour anticipé avant les deux vérifications.

Conditions :
- Une marque « second facteur validé » en session, exigée par le middleware de l'admin. Sinon, la connexion du site contourne la 2FA.
- Mode email sans mailer configuré : erreur au démarrage.
- Runique garde `LoginGuard`, `can_sign_in()`, la rotation de session, le CSRF et le contrôle de `next` : le dev choisit le mode, il ne réécrit pas la connexion.

## Limites (ce que Runique ne protège pas)

- Celui qui contrôle la boîte mail **et** le téléphone.
- Celui qui contrôle la boîte mail au moment d'une panne SMS (risque du repli, accepté).
- Un compte sans numéro : un seul canal, l'email.
- Le maillon humain : le support qui change un numéro sur simple appel.

À documenter dans `SECURITY.md` : ce que Runique protège, et ce qu'il ne peut pas protéger.

## À trancher

| Question | Pistes |
| --- | --- |
| Garantir que `.phone()` vise une colonne de `eihwaz_users` | Marqueur généré par `extend!` (erreur de compilation) ou contrôle au démarrage |
| Déni de service à la connexion admin : un inconnu brûle les essais et bloque le vrai utilisateur jusqu'à la fin du délai | Lever le délai quand le code est détruit + plafond quotidien, ou risque accepté et tracé |
| TOTP comme variante de l'enum | Application d'authentification (RFC 6238), indépendante de l'email, secret rangé dans `SECRET` ; passkeys (WebAuthn) plus tard |

## Écarté (et pourquoi)

| Idée | Raison |
| --- | --- |
| Numéro saisi dans le formulaire | L'attaquant tape le sien |
| Code affiché à l'écran puis demandé | Celui qui est devant l'écran le recopie : ne prouve rien |
| Mot de passe d'abord, code ensuite (connexion) | « Code envoyé » confirme que le mot de passe est bon : oracle |
| Deux emails (alerte + lien) dans la même boîte | Même canal, même information : l'email du lien vaut alerte |
| Saisie manuelle du numéro « en secours » | L'attaquant choisirait toujours ce secours |
| Surcharger tout le formulaire de connexion | Le dev perdrait `LoginGuard`, la rotation de session, le contrôle de `next` sans le voir |
