# Audit bugs & sécurité — depuis les diagrammes

> Source : `diagramme/` (UML + Merise + flux). Croisé avec le code réel au 2026-07-25.
> Ce rapport ne liste que ce qui **reste à corriger** ou à **acter**, plus une vérification
> des écarts entre `anomalies.md`, `ETAT-AVANCEMENT.md` et le code.
> Le registre historique complet reste `diagramme/anomalies.md`.

## 0. Constat général

L'audit par diagrammes de la session du 2026-06-29 est **solide** : tous les critiques
(C1/C3 upload, C2 CSRF, D1 big-pk, CX2/STRICT_CSP, CFG1 secret_key) sont corrigés et
vérifiés dans le code. Aucun nouveau critique détecté. Ce qui reste est **sérieux/dette**
ou dépend d'une DB / du multi-instance.

---

## 1. Modifications en cours (working tree) — verdict

Le diff non commité (`csp.rs`, `error.rs`, `build.rs`, `config/security.rs`) corrige un
**footgun réel absent de `anomalies.md`** :

- **Avant** : `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`
  était **codé en dur dans 3 sites** (middleware CSP, pages d'erreur, fichiers statiques).
  Le `preload` en dur est quasi-irréversible (soumission à la liste des navigateurs) et
  imposé à tout utilisateur du framework sans opt-in.
- **Après** : source unique `SecurityConfig::hsts_header_value()`, `preload` **opt-in
  (défaut false)**, `max-age`/`includeSubDomains` configurables, warning boot si combo
  preload invalide (`hsts_preload_misconfigured()` → [build.rs:82](runique/src/app/builder/build.rs#L82)).

**Verdict : audit clean, à conserver.** Le sens des `unwrap_or` sur les env vars est sûr
(valeur malformée → défaut durci). Un seul point à surveiller lors du merge : le retrait de
HSTS des fichiers statiques ([build.rs:344](runique/src/app/builder/build.rs#L344)) repose sur
la portée *host-scoped* de HSTS — correct **tant qu'une page non-statique émet le header en
premier**. Sur un déploiement servant **uniquement** du statique sous un sous-domaine dédié
(`static.example.com` sans page HTML), HSTS ne serait jamais posé pour cet hôte. Cas de bord
à documenter, pas un bug de la config actuelle.

**À faire avant release :** exposer `HSTS_*` dans la doc `.env` (fr/en) + `.env.example` +
CHANGELOG (nouveaux champs `SecurityConfig`).

---

## 2. Écarts documentaires à corriger

| Item | anomalies.md | ETAT-AVANCEMENT.md | Code réel | Action |
|------|--------------|--------------------|-----------|--------|
| **CFG1** secret_key faible = échec boot | ✅ corrigé | ❌ listé « restant » (l.89) | ✅ **corrigé** — `cross_validate` refuse le boot en prod ([build.rs:293](runique/src/app/builder/build.rs#L293)) | Marquer CFG1 résolu dans ETAT-AVANCEMENT |
| **CX2** header_security en prod | ✅ corrigé | listé « nouvelles anomalies » | ✅ corrigé | Idem, purement doc |

`ETAT-AVANCEMENT.md` est en retard d'une session sur `anomalies.md`. Purement cosmétique.

---

## 3. Sérieux — à corriger

### 🟠 SEC2 — `TrustedProxies` par défaut fait confiance à tout le réseau privé
[trusted_proxies.rs:72](runique/src/middleware/security/trusted_proxies.rs#L72) — **confirmé dans le code** :
le défaut liste loopback + RFC 1918 + ULA IPv6. Derrière un reverse-proxy c'est le bon défaut.
Mais si l'app est exposée **directement** avec ce défaut, un client du même réseau privé peut
usurper `X-Forwarded-For` → fausse `ClientIp` → contourne rate-limit et lockout **par IP**.
**Correctif :** documenter explicitement `TrustedProxies::none()` sans proxy, et/ou logger un
warning boot si `enforce_https`/proxy non configuré mais défaut large actif.

### 🟠 A3 — `list_filter` dans `configure {}` builtin → 500
Bug connu (déjà dans le CLAUDE global). `filter_values[col] not found in context` sur le
chemin `configure`. **Correctif :** pousser `filter_values` dans le contexte du chemin
`configure`, comme sur le chemin ressource normal.

### 🟠 S1 — Sessions anonymes non persistées → CSRF des forms publics perdu au restart
[cleaning_store.rs:148](runique/src/middleware/session/cleaning_store.rs#L148). Un formulaire
public émis avant un restart voit son token CSRF invalidé (session mémoire perdue).
**Correctif :** persister un minimum anonyme, ou rendre le CSRF public indépendant de la
session mémoire. *(Nécessite une DB pour tester.)*

### 🟠 S2 — `save` relâche le lock avant l'écriture DB async
[cleaning_store.rs:378](runique/src/middleware/session/cleaning_store.rs#L378). Le backup DB
peut être périmé d'un cran sous concurrence. **Correctif :** sérialiser le persist par session
ou versionner le snapshot. *(DB requise.)*

### 🟠 D2 — `eihwaz_history.user_id` sans FK : choix implicite non documenté
[migrations_table.rs:283](runique/src/admin/table_admin/migrations_table.rs#L283). Absence de
FK **probablement voulue** (l'audit doit survivre à la suppression d'un user, un CASCADE
détruirait la trace). **Correctif :** documenter le choix en commentaire, sinon quelqu'un
« corrigera » en ajoutant un CASCADE destructeur.

### 🟠 D3 — Index manquants sur colonnes de requête/purge
- `eihwaz_sessions.user_id` (`invalidate_other_sessions` → scan)
- `eihwaz_history(resource_key, object_pk, batch_id)` (vues/filtres → scan)
- `eihwaz_reset_tokens.expires_at` (purge → scan)

Latent perf, s'aggrave avec le volume. **Correctif :** ajouter les index en migration, valider
sur Postgres/MariaDB/SQLite. *(DB requise.)*

---

## 4. Dette / mineurs — à acter

| ID | Anomalie | Localisation | Nature |
|----|----------|--------------|--------|
| **AU1/SEC1/AU2/AM4** | State process-local (lockout, rate-limit, cache permissions) → limites/lockout par instance, permissions périmées en multi-instance | guard.rs / rate_limit.rs / session.rs | Roadmap multi-instance (externaliser le state). OK en mono-process actuel |
| **ACR1 / A1** | `get_fn`/`list_fn` = `None` → page vide/« non trouvé » silencieux (create/update/delete ont des fallbacks sûrs, cf. anomalies A1 = non-issue) | handle_crud.rs | Durcir en 501 + log pour get/list |
| **AM2** | Double écriture `eihwaz_sessions` au login (perf, pas de divergence — vérifié) | session.rs:382 | Fusionner les 2 écritures |
| **D4/S4** | Double identifiant `cookie_id`/`session_id` à documenter | migrations_table.rs:209 | Doc |
| **E1** | `enable_debug_errors` : nom trompeur (défaut `true`, handler monté en prod) | middleware/config.rs:65 | Renommer / documenter |
| **E3** | Écart ordre d'écriture vs exécution des `.layer()` Axum | engine/core.rs:110 | Doc |
| **S3 docs** | 503 saturation + `session_store_saturated()` non documentés (fr/en) | — | Doc |

---

## 5. Faux positifs confirmés — NE PAS re-flaguer

- **M1/AM1** : `makemigrations` gère les `ALTER COLUMN` (`diff_schemas`/`modified_columns`).
- **AM2 divergence** : `on_conflict` disjoints, `session_id` déterministe (résidu = perf).
- **TR1** : `ErrorContext` gaté sur `config.debug` — pas exposé en prod.
- **A1/A2** : closures CRUD ont des fallbacks sûrs ; `own_field=None` = défaut sûr par design.
- **E2** : `attach_middlewares` = code mort (le staging applicator est le chemin vivant).

---

---

## 6. Audit code direct (lecture ligne à ligne, hors diagrammes)

Surfaces relues intégralement : CSRF (middleware + extractor + crypto token), hash password,
reset token, pipeline upload, RBAC admin, construction SQL, open redirect, `unsafe`, `unwrap`
runtime. **Verdict global : fondamentaux solides, pas cargo-cultés.** Détail des points positifs
et des findings neufs.

### Points forts vérifiés (pas juste supposés)

- **Comparaisons constant-time partout** où un secret est comparé : CSRF pipeline + middleware +
  admin ([mod.rs:1098](runique/src/admin/admin_main/mod.rs#L1098)), MAC du reset token
  ([reset_token/mod.rs:159](runique/src/utils/reset_token/mod.rs#L159)), password via libs vettées.
- **Anti-énumération réel** : `authenticate_user` fait toujours tourner `verify` contre un
  `dummy_hash` Argon2 même si l'utilisateur n'existe pas, et teste `is_active` **après** le verify
  ([user.rs:151](runique/src/auth/user.rs#L151)). Pas de fuite timing existe/actif.
- **CSRF fail-closed** : unmask KO → `false`, méthode inconnue (OPTIONS/TRACE) → token exigé,
  source unique `csrf_required` ([extractor.rs:106](runique/src/forms/extractor.rs#L106)).
- **Upload durci** : `sanitize_filename` **jette le nom user** et génère `{uuid}.{ext}` → path
  traversal neutralisé ; images validées par magic bytes ; SVG toujours rejeté ; staging non
  servi + commit après CSRF/validation ; sweep TTL des orphelins.
- **RBAC** : `ResourcePerms` = source unique UI + serveur, parse d'action par méthode
  (cross-méthode → 404), CSRF vérifié **avant** tout effet de bord
  ([mod.rs:520](runique/src/admin/admin_main/mod.rs#L520)), superuser explicite, défaut deny-all.
- **SQL** : 100% paramétré. Filtres/tri → colonnes whitelistées (`SORT_COLS`/`FILTER_COLS`) avant
  `Alias::new` ; recherche → `.like(format!("%{}%", val))` avec **valeur bindée** par SeaORM
  ([filter.rs:44](runique/src/macros/bdd/filter.rs#L44)) ; FK → `is_in` bindé, identifiants macro.
- **Open redirect** : normalise les backslashes (`\/evil.com`→`//evil.com`, bypass souvent
  oublié), fail-closed sur l'inparsable, détection loopback exhaustive (IPv4/IPv6/mapped)
  ([open_redirect.rs:44](runique/src/middleware/security/open_redirect.rs#L44)).
- **Zéro `unsafe` en production** (les 20 occurrences sont `env::set_var` de tests). Aucun
  `unwrap`/`expect` sur le chemin de traitement d'une requête (ceux restants = boot/CLI/tests).

### Findings neufs (invisibles depuis les diagrammes)

| ID | Sévérité | Finding | Localisation |
|----|----------|---------|--------------|
> **Recalibration honnête (après lecture du flux complet)** : NEW1 et NEW2 étaient
> **surévalués** dans ma première passe. Sévérités corrigées ci-dessous.

| **NEW1** | 🟡 Dette/simplification (pas vuln) | **Crypto maison saine mais custom.** `encrypt_email`/`decrypt_email` = CTR SHA256 + **Encrypt-then-MAC** (bonne composition), tag 128 bits **constant-time**, clé = token UUID4 **fraîche à chaque `generate()`** → pas de réutilisation de keystream dans l'usage réel. **Vérifié : l'intégrité de l'auth ne dépend PAS de ce module** — la garde serveur `consume(token)→user_id→find_by_id→compare email` ([password.rs:406](runique/src/auth/password.rs#L406)) couvre déjà le binding ; l'email chiffré n'est qu'un cross-check UX + confidentialité de l'email dans l'URL. Résiduel : (a) sûreté dépend d'un invariant non-forcé (token unique par chiffrement) → un AEAD à nonce (`chacha20poly1305`) la rendrait locale ; (b) charge de revue d'une crypto maison dans un framework publié ; (c) `.expect("REASON")` placeholder (l.121/175) ; (d) **le plus simple : supprimer le module** (la garde serveur suffit) | [reset_token/mod.rs:105](runique/src/utils/reset_token/mod.rs#L105) |
| **NEW2** | ⚪ Retiré (note de doc) | **Non-finding.** `FileField::any()` (tout sauf SVG) est un constructeur **permissif opt-in documenté**. Les presets sûrs existent : `image()` (whitelist + magic bytes), `document()`, `.allowed_extensions(vec![…])`. Le dev restreint trivialement ; la whitelist est appliquée sur la vraie extension du fichier stagé. Seule note : documenter « ne pas mettre `any()` derrière un endpoint public dont MEDIA_ROOT est servi » | [file.rs:135](runique/src/forms/fields/file.rs#L135) |
| **NEW3** | 🟡 Mineur | **(à discuter)** Extension user non restreinte : `{uuid}.{ext}`, `ext` du nom user sans filtre charset → sur NTFS un `ext` avec `:` peut créer un Alternate Data Stream ; unicode exotique. Borner `ext` à `[A-Za-z0-9]{1,10}` lowercased | [parse_html.rs:229](runique/src/utils/forms/parse_html.rs#L229) |
| **NEW4** | 🟡 Dette (pas d'exposition) | **CSRF, deux points.** (1) `generation_token` et `generation_user_token` sont byte-identiques → collapser en une fn (dette, zéro sécu). (2) `masked().unwrap_or_else(\|_\| clone())` renverrait le token non-masqué **si** le masquage échouait. **Vérifié par le flow : non-atteignable par l'attaquant** — `masked()` opère sur le token *serveur* (toujours hex valide), jamais sur l'input requête ; et le token renvoyé est celui de **la session du demandeur** (par-session), pas de la victime. Branche morte → perte de BREACH sur une réponse *si* la génération changeait un jour. Fix = newtype validé (mask infaillible), design pas urgence | [csrf.rs:52](runique/src/utils/middleware/csrf.rs#L52), [csrf.rs:170](runique/src/middleware/security/csrf.rs#L170) |
| **NEW5** | 🟡 Débattable | **Open redirect : `127.0.0.1`/`localhost` = safe.** Pour le **navigateur de la victime**, c'est la machine de la victime, pas le serveur → vecteur inhabituel (services locaux). Défendable. ✅ **Commentaire corrigé** ([open_redirect.rs:64](runique/src/middleware/security/open_redirect.rs#L64)) : le « unreachable by external attackers » imprécis remplacé par la note exacte (loopback victime, tradeoff DX assumé) | [open_redirect.rs:64](runique/src/middleware/security/open_redirect.rs#L64) |
| **NEW6** | 🟡 Mineur/dette | **Rate-limiter : fallback XFF spoofable, hors-pipeline seulement.** `rate_limit_middleware` préfère l'extension `ClientIp` (trusted) et ne lit `x-forwarded-for` brut que si elle est absente → **mort dans une app builder** (TrustedProxies au slot 2 la pose toujours). Reachable seulement si on câble le middleware à la main sans TrustedProxies. Mitigé : `LoginGuard` (par-username) backstoppe le login. Durcir le fallback en fail-safe (bucket « unknown » plutôt que XSS brut) | [rate_limit.rs:197](runique/src/middleware/security/rate_limit.rs#L197) |

---

## 7. Implémenté cette session (2026-07-26)

| Item | Décision | Détail |
|------|----------|--------|
| **SEC2** | ✅ **Corrigé** | Défaut `TrustedProxies` **edge-aware** : `TrustedProxies::default_for_edge(acme_enabled)` → `none()` en mode ACME (Runique = edge TLS, pas de proxy), plages privées sinon. Point d'assemblage [build.rs:116](runique/src/app/builder/build.rs#L116). Tests `default_for_edge_*`, `edge_mode_ignores_spoofed_xff`. Doc fr/en enrichie (défaut ACME + encadré « quand utiliser `.none()` » nommant le risque) |
| **SEC2b** | ❌ **Rejeté (documenté)** | Gater `X-Forwarded-Proto` sur le peer trusted **casserait les proxies à IP publique (Cloudflare) en boucle de redirection** (peer public non-trusted → XFP ignoré → redirect infini), pour un gain sécurité ≈ 0 (forger XFP ne downgrade que sa propre connexion, aucun impact tiers). Comportement d'origine conservé + commentaire expliquant pourquoi on ne gate pas ([csp.rs](runique/src/middleware/security/csp.rs)) |
| **Warning A** | ❌ **Abandonné** | Redondant en ACME (B auto-corrige) et bruyant sinon (cas non-ACME exposé-direct indétectable → warning pour tous les users derrière-proxy). Doc à la place |
| **NEW4** | ✅ **Corrigé** | (1) `generation_token`/`generation_user_token` délèguent à une privée `generation()` unique. (2) mask fail-safe : header CSRF **non posé** si le masquage échoue (plus de token nu). `.unwrap()` sur `duration_since` retiré au passage |
| **NEW6** | ✅ **Corrigé** | `extract_ip` (fallback rate-limit) lit désormais le **peer réel `ConnectInfo`**, plus jamais `X-Forwarded-For` spoofable |
| **NEW5** | ✅ **Corrigé** | Commentaire open_redirect rectifié |
| **NEW3** | ✅ **Corrigé** | `sanitize_filename` borne l'extension : ASCII alphanumérique minuscule, max 10 chars → tue l'ADS NTFS (`photo.php:zone` → `php`), l'unicode exotique, normalise la casse. 5 tests TDD ([parse_html.rs](runique/src/utils/forms/parse_html.rs)) |
| **NEW1 / NEW2** | ⏸️ **Séparés** | NEW1 = décision design (supprimer `encrypt_email` / AEAD) ; NEW2 = non-finding (doc) |

> Découverte pendant le trace : **boucle de redirection latente pré-existante** si `acme_enabled` **et** `enforce_https` sont tous deux activés (le port 80 redirige déjà, `https_redirect` sur le 443 reboucle). Non introduite ici — à traiter séparément si voulu.

⚠️ `cargo test`/`clippy` **pas encore lancés** (règle : sur demande explicite).

---

## Priorisation suggérée (reste)

1. **NEW1 crypto reset token** — trancher : supprimer `encrypt_email` si vestigial, sinon
   migrer vers un AEAD vetté. C'est mon inquiétude n°1 (pas exploitable en l'état, mais fragile).
2. **NEW2 `any()` + MEDIA servi** — décider la politique (attachment / denylist types actifs).
3. **Merge du fix HSTS** (working tree) + doc `HSTS_*` — footgun réel, prêt.
4. **SEC2** doc/warning `TrustedProxies` — surface concrète sans proxy.
5. **NEW3/NEW4/NEW5** — hardening rapide (ext charset, dédup CSRF, commentaire redirect).
6. **A3** `list_filter configure{}` ; **D2/D3/S1/S2** (session DB, prochaine session Docker).
