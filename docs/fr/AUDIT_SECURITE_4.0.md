# Audit de sécurité — Runique 4.0.0 (arbre `main`)

> **Nature du document** : audit / constat uniquement.  
> **Aucune correction de code n’a été appliquée** dans le cadre de cet audit.  
> Date : 2026-03-29 · Périmètre : crate `runique` (+ flux demo-app liés à l’auth/admin) · Version workspace : `4.0.0` (non publiée crates.io).

---

## 1. Méthode

| Source | Usage |
|--------|--------|
| Code vivant (`runique/src/**`) | Lecture des flux sensibles : auth, reset, session, CSRF, uploads, admin, proxies, open-redirect, rate-limit |
| `SECURITY.md`, `CHANGELOG.md` [4.0.0] | Advisories déjà fermées / documentées |
| `ROADMAP.md` § « Identified fixes (next 4.0.x patch) » | File de patch sécu déjà priorisée produit |
| `diagramme/anomalies.md`, `diagramme/ETAT-AVANCEMENT.md`, UML middleware/auth | Anomalies historiques, faux positifs, dettes ouvertes |
| Working tree non commité | Signalé à part (ne fait **pas** partie de la baseline `HEAD`) |

**Hors scope volontaire** : revue ligne-à-ligne de tout `demo-app` métier, fuzzing runtime, pentest réseau, audit des dépendances au-delà de la politique CI `cargo audit` déjà en place.

**Sévérité** :

| Tag | Sens |
|-----|------|
| 🔴 Critique | Exploitation réaliste → prise de compte, RCE, fuite de secrets, contournement d’auth large |
| 🟠 Élevé | Contournement partiel, DoS ciblé, fuite d’info sensible, design crypto trompeur |
| 🟡 Moyen | Durcissement, multi-instance, dettes, fenêtre temporelle, UX sécu |
| 🟢 Faible / info | Déjà mitigé, by design documenté, ou risque résiduel acceptable |
| ✅ Fermé | Corrigé dans l’arbre 4.0 (référence) |

---

## 2. Synthèse exécutive

Runique 4.0 arrive avec une **base sécu solide** pour un framework web Django-like : CSRF fail-closed, auth timing-aware, tokens reset hashés + single-use + mutation par `user_id` serveur, uploads en staging après CSRF, open-redirect middleware, CSP/HSTS, redaction d’historique, boot prod qui refuse `secret_key` faible / reset sans mailer / sans `public_url`.

Les **correctifs 4.0 déjà dans l’arbre** (timing forgot-password, XSS erreurs globales, path traversal page debug, `csrf.js` same-origin, `password`/`is_active` hors forms `extend!`, message superuser générique) sont cohérents avec `SECURITY.md`.

La **file patch `4.0.x` du ROADMAP est juste** : le point n°1 (email chiffré dans le lien de reset) est le vrai sujet design le plus net de cet audit. Les autres items (uploads refusés, plafonds mémoire, sessions superuser, tests open-redirect, messages FK) sont confirmés comme dettes sécu/robustesse, pas comme « déjà réglés ».

**Verdict** : pas de 🔴 critique nouvellement prouvée sur `HEAD` dans le périmètre lu. Plusieurs 🟠/🟡 **ouverts**, dont une partie déjà listée produit. Priorité recommandée : **ne pas publier 4.0.0 sans au minimum traiter ou documenter explicitement le point reset-email-in-link** (soit fix patch, soit advisory « known limitation » si reporté après release).

---

## 3. Ce qui est sain (ne pas re-flaguer sans preuve nouvelle)

| Zone | Constat | Preuves |
|------|---------|---------|
| Auth password | `authenticate_user` vérifie toujours un hash (dummy si user absent) → anti-énumération timing ; `can_sign_in` = actif + `activated_at` | `auth/user.rs` |
| Auth admin | Mot de passe d’abord, rang/staff ensuite | `authenticate_admin` |
| Reset IDOR | Cible = `user_id` de la ligne token en DB, pas un id client | `reset_token::consume` + `handle_password_reset` |
| Reset single-use | Delete + `rows_affected == 1` | `utils/reset_token/mod.rs` |
| Reset timing (forgot) | Lookup + token + mail en `tokio::spawn` ; même réponse pour toute adresse | `handle_forgot_password` |
| Token stocké | SHA-256 du raw token, raw seulement dans le mail | `hash_token` |
| CSRF | Source unique `csrf_required` ; ct_eq header ; mask BREACH ; `Prisme::data` `pub(crate)` ; message d’échec | middleware + forms |
| Uploads ordre | Staging `.staging-*` ; écriture fichier après CSRF dans le chemin gardé ; `is_staged_upload` canonicalisé | `parse_html.rs`, `file.rs` |
| Open redirect | Parser WHATWG ; Location + Refresh ; credentials / tabs | `open_redirect.rs` |
| History secrets | `REDACTED` + `is_sensitive_key` (password/token/…) | `auth/history.rs` |
| Boot prod | `secret_key` faible refusé ; reset sans mailer/public_url refusé | builder / SECURITY |
| Proxies ACME | `TrustedProxies::none()` si edge ACME ; pas de XFF sans `ConnectInfo` | `trusted_proxies.rs` |
| Advisories 4.0 listées | Voir §4 | `SECURITY.md` |

---

## 4. Advisories 4.0 déjà fermées (référence)

| ID doc | Sujet | Sévérité doc | Statut |
|--------|-------|--------------|--------|
| extend! password/is_active | Form admin pouvait fixer mdp / désactiver hors règles compte | High | ✅ 4.0 |
| csrf.js cross-origin | Token CSRF envoyé hors origin | Medium | ✅ 4.0 |
| XSS erreurs globales form | Rendu non échappé | Medium | ✅ 4.0 |
| Debug template path | Lecture hors `templates/` | Medium | ✅ 4.0 |
| Message superuser | Fuite de rang | Low | ✅ 4.0 |
| Timing forgot-password | Énumération par latence | Low | ✅ 4.0 |

Détail : `SECURITY.md` + `CHANGELOG.md` `[4.0.0]`.

---

## 5. Findings ouverts (audit 4.0)

### 5.1 File ROADMAP `4.0.x` — confirmée

Ces six points sont **reproduits / confirmés** par lecture de code. Ordre produit déjà défini dans `ROADMAP.md` : les traiter **avant** les mineures 4.x features.

---

#### F-01 — Email « chiffré » dans le lien de reset 🟠 Élevé (design)

| | |
|--|--|
| **ROADMAP** | Item 1 |
| **Où** | `utils/reset_token::{encrypt_email,decrypt_email}`, `auth/password.rs` (`send_reset_link`, `PasswordResetForm`, route `/{token}/{encrypted_email}`) |
| **Quoi** | Le ciphertext de l’email est authentifié (HMAC-SHA256 tronqué 16 o) et « chiffré » en CTR maison dont la clé est dérivée du **token**, placé **dans le même URL**. Quiconque possède le lien possède la clé → **aucune confidentialité** de l’email vis-à-vis du détenteur du lien. |
| **Impact** | 1) Fausse sensation de protection (crypto theater). 2) Surface crypto custom (CTR-SHA256 non standard) à maintenir sans bénéfice. 3) Email visible dans historique navigateur, logs proxy, Referer si mal configuré — le « chiffrement » ne change rien pour qui a le lien. |
| **Ce qui reste correct** | Le reset reste lié au `user_id` DB ; l’email URL n’est qu’un cross-check UX ; pas d’IDOR évident via le segment email seul. |
| **Correction appliquée** | Lien = token seul ; au POST, comparer l’email saisi au compte résolu par le token ; helpers `encrypt_email`/`decrypt_email` retirés. L’activation de demo-app utilise aussi l’ID lié au token. |
| **Tests** | Liens de reset token-only, email incorrect sans consommation du token, consommation simple et token à usage unique. |
| **Breaking ?** | Les anciens liens reçus avant le déploiement ne sont plus acceptés ; ils expirent après la durée de vie du token (1 h par défaut). |

---

#### F-02 — Fichiers staging d’un formulaire refusé 🟡 Moyen

| | |
|--|--|
| **ROADMAP** | Item 2 |
| **Où** | `utils/forms/parse_html.rs` (`STAGING_TTL_SECS = 3600`), `forms/fields/file.rs` (`cleanup_files` / `is_staged_upload`) |
| **Quoi** | Un upload peut rester en `.staging-*` jusqu’au sweep (~1 h) si le formulaire est invalidé pour une autre raison, selon les chemins de cleanup. |
| **Impact** | Fenêtre disque + éventuelle surface si un chemin staging était jamais servi (aujourd’hui : staging non servi, CSP media sans script — mitigations 3.0.2). Risque surtout **stockage / hygiene**, pas RCE direct. |
| **Reco** | Supprimer les staged files dès l’échec de validation du form (tous champs), pas seulement au TTL. Test : form invalide → plus de fichier staging. |

---

#### F-03 — `RateLimiter` / `LoginGuard` : maps non bornées 🟡 Moyen (DoS mémoire)

| | |
|--|--|
| **ROADMAP** | Item 3 |
| **Où** | `middleware/security/rate_limit.rs`, `auth/guard.rs` |
| **Quoi** | `HashMap` process-local sans plafond de clés entre deux sweeps. Un attaquant avec beaucoup d’IP (ou usernames) peut grossir la map. |
| **Impact** | DoS mémoire sur instance unique. Lockout par username reste aussi un levier de **lockout DoS de compte** (classique, by design anti-bruteforce). |
| **Reco** | Cap de clés + politique d’éviction (LRU / drop oldest) ; documenter le tradeoff lockout compte. Tests de borne. |

---

#### F-04 — Sessions d’un superuser non alignées sur la protection du compte 🟠 Élevé (admin)

| | |
|--|--|
| **ROADMAP** | Item 4 |
| **Où** | Admin sessions / `close_user_sessions` / CRUD sessions (à traiter avec les gates compte superuser déjà durcies en 3.0.3/4.0) |
| **Quoi** | Le compte superuser est protégé contre prise de contrôle staff (email/mdp) ; les **sessions** du superuser ne bénéficient pas du même niveau de garde (staff pouvant les clôturer). |
| **Impact** | DoS de session / harcèlement admin sur le superuser par un staff lesser — pas une prise de compte directe, mais rupture du modèle « superuser intouchable ». |
| **Reco** | Même règle que le compte : seul un superuser touche les sessions d’un superuser. Tests refus staff. |

---

#### F-05 — Couverture tests open-redirect incomplète 🟡 Moyen (assurance)

| | |
|--|--|
| **ROADMAP** | Item 5 |
| **Où** | `middleware/security/open_redirect.rs` + tests associés |
| **Quoi** | Implémentation déjà sérieuse (WHATWG, Refresh, non-ASCII, credentials). Manque une batterie alignée cheat sheet PortSwigger *URL validation bypass*. |
| **Impact** | Risque de régression ou bypass exotique non couvert — pas un bug prouvé aujourd’hui. |
| **Reco** | Importer les payloads de la cheat sheet en tests table-driven ; aucun changement prod tant que vert. |

---

#### F-06 — Erreur FK à la suppression admin 🟡 Moyen (info disclosure / UX sécu)

| | |
|--|--|
| **ROADMAP** | Item 6 |
| **Où** | Admin delete / bulk delete |
| **Quoi** | Échec FK → erreur DB brute possible au lieu d’un message métier borné. |
| **Impact** | Fuite de structure / confusion opérateur ; pas d’escalade directe. |
| **Reco** | Message stable (« N dépendances ») ; logs détaillés côté serveur seulement. |

---

### 5.2 Dettes encore ouvertes hors file courte ROADMAP

Issues déjà dans `diagramme/anomalies.md` / UML, **toujours pertinentes** :

| ID | Sévérité | Sujet | Note audit |
|----|----------|-------|------------|
| **S1** | 🟡 | Sessions anonymes non persistées → CSRF public perdu au restart | Dispo / UX sécu, pas vol de session auth |
| **S2** | 🟡 | Lock session relâché avant write DB async | Course possible snapshot périmé d’un cran |
| **D3** | 🟡 | Index manquants (`sessions.user_id`, history composite, `reset_tokens.expires_at`) | Perf + coût purge ; pas d’authz bypass |
| **D2** | 🟢 | `history.user_id` sans FK | **By design** (audit survit au delete user) — à documenter produit si pas déjà clair |
| **SEC1 / AU1** | 🟡 | Rate-limit + LoginGuard **process-local** | En multi-instance : quota × N ; lockout non partagé |
| **SEC2 résiduel** | 🟢/🟡 | Défaut TrustedProxies = plages privées si pas ACME | OK derrière reverse-proxy privé ; **footgun** si HTTP direct sans `.none()` — doc critique |
| **AM2** | 🟢 | Double écriture session au login | Perf only |

---

### 5.3 Observations complémentaires (cet audit)

#### F-07 — Crypto custom reset (au-delà du theater F-01) 🟡 Moyen

Même si on gardait un email dans l’URL (non recommandé), le primitive CTR-SHA256 + HMAC tronqué n’est pas une AEAD standard (`chacha20poly1305`, etc.). **Reco** : ne pas « améliorer » ce primitive — **supprimer** le besoin (F-01).

#### F-08 — `consume` = read puis `delete_by_id` 🟢 Acceptable

Course double-submit documentée ; gagnant = `rows_affected == 1`. Pas de finding bloquant. Amélioration possible plus tard : `DELETE … RETURNING` atomique selon moteur.

#### F-09 — Token = UUID v4 string 🟢 OK

Entropie suffisante pour bearer single-use TTL 1 h. Hash SHA-256 non salé acceptable car l’entrée a haute entropie (pas un mot de passe).

#### F-10 — LoginGuard par username 🟢 By design / 🟡 ops

Protège le compte ciblé indépendamment de l’IP (bon contre bot distribué). Permet lockout d’un compte connu. Documenter + alerter ops ; option future : seuil global + captcha.

#### F-11 — `success_redirect` du reset non lu 🟢 Info

Champ config documenté comme non consommé (re-render `reset_done`). Pas sécu ; éviter qu’un dev croie qu’un open-redirect y est branché.

#### F-12 — Working tree non commité (hors baseline) 🟠 Process

Au moment de l’audit, des modifications locales **non demandées comme correctifs** étaient présentes :

| Fichier | Observation |
|---------|-------------|
| `runique/src/utils/reset_token/mod.rs` | Retrait de `encrypt_email`/`decrypt_email` + ajout `resolve` — F-01 complété dans le framework et demo-app |
| `demo-app/seed.sql` | Exemple `ModelAdmin` **corrompu** (code invalide injecté dans le seed) |
| `docs/*/PROJECT_STATUS*.md` | Ajout d’une ligne « file patch sécu 4.0.x » |

→ **À traiter comme brouillon à abandonner ou finir volontairement**, pas comme livrable d’audit. Cet audit **n’a appliqué aucun correctif** et ne valide pas ce diff.

---

## 6. Matrice de risque (ouvert seulement)

| ID | Titre | Sévérité | Exploitabilité | Priorité patch |
|----|-------|----------|----------------|----------------|
| F-01 | Email chiffré dans lien reset | 🟠 | Facile à comprendre / abuser conceptuellement | **P0** (ROADMAP #1) |
| F-04 | Sessions superuser | 🟠 | Staff authentifié | **P1** (#4) |
| F-03 | Maps non bornées | 🟡 | Réseau / volume | **P1** (#3) |
| F-02 | Staging 1 h | 🟡 | Upload + form invalide | **P2** (#2) |
| F-05 | Tests open-redirect | 🟡 | Assurance | **P2** (#5) |
| F-06 | Message FK | 🟡 | Staff | **P2** (#6) |
| S1/S2 | Session store | 🟡 | Restart / course | Backlog robustesse |
| SEC1 | Multi-instance counters | 🟡 | Déploiement scale-out | Avant HA |
| D3 | Index | 🟡 | Charge DB | Session DB |

---

## 7. Surfaces revues — checklist

| Surface | Statut audit |
|---------|--------------|
| Login public / admin timing | ✅ OK |
| Session fixation (`cycle_id` au login, doc status) | ✅ (revu doc + design) |
| CSRF pipeline + forms | ✅ OK |
| Password reset forgot/reset/admin send link | 🟠 F-01 ; reste IDOR/timing OK |
| Uploads multipart / staging / media | 🟡 F-02 ; ordre CSRF OK |
| Open redirect | ✅ code ; 🟡 tests F-05 |
| Trusted proxies / ClientIp | ✅ mitigations ; footgun doc SEC2 |
| Rate limit / LoginGuard | 🟡 F-03 + multi-instance |
| Admin gates / superuser | ✅ messages/compte ; 🟠 sessions F-04 |
| History redaction | ✅ OK |
| Headers CSP/HSTS | ✅ (historique CX2/CX3 fermés) |
| Secrets dans logs (mailer Debug) | ✅ TR2 fermé |
| `cargo audit` CI | ✅ politique SECURITY |

---

## 8. Recommandations de pilotage (toujours sans coder)

1. **Geler toute « correction opportuniste » hors ticket** — l’audit livre des findings, pas des PR partielles (cf. F-12).
2. **Traiter la file ROADMAP 4.0.x dans l’ordre**, un item = tests de non-régression + `cargo mutants --in-diff` (définition « finished » du ROADMAP).
3. **Décision release 4.0.0** :
   - soit **F-01 dans 4.0.0** avant crates.io,
   - soit 4.0.0 + advisory « known » + **4.0.1 immédiat** dédié F-01.
4. **Ne pas ouvrir les mineures features 4.x** tant que la file P0/P1 n’est pas vidée (déjà écrit ROADMAP).
5. **Documenter** SEC2 (`.none()` si exposition directe) et D2 (history sans FK) côté ops/produit si pas assez visibles.
6. **HA / multi-instance** : externaliser LoginGuard + RateLimiter (Redis ou équivalent) avant scale horizontal — sinon documenter « single instance security counters ».

---

## 9. Non-objectifs de cet audit

- Aucun patch, aucun rename API, aucun commit.
- Pas de modification de `seed.sql`, reset_token, ou PROJECT_STATUS dans le cadre de l’audit.
- Pas de re-run complet de la suite `cargo test` (constat statique + recoupement docs).
- Pas de nouveau diagramme UML (les existants restent la carte).

---

## 10. Conclusion

Runique 4.0 n’est **pas** dans un état « passoire » : le travail sécu 3.0.2 → 4.0 se voit dans le code.  
Le principal écart **design** restant est **F-01** (email dans le lien de reset).  
Le reste de la file 4.0.x est de la **hygiène offensive / admin / DoS** à planifier, pas des 🔴 non documentés.

**Livrable** : ce fichier uniquement.  
**Prochaine étape humaine** : prioriser F-01…F-06 sur le board, décider release, **puis** ouvrir des correctifs ticket par ticket — pas un mega-diff « audit+fix ».

---

*Rédigé pour relecture humaine. Toute implémentation ultérieure doit partir de ce constat, avec tests, sans élargir le scope.*
