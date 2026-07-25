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

## Priorisation suggérée

1. **Merge du fix HSTS** (working tree) + doc `HSTS_*` — footgun réel, prêt.
2. **SEC2** doc/warning `TrustedProxies` — surface d'attaque concrète sans proxy.
3. **A3** `list_filter configure{}` — bug fonctionnel connu, régulièrement rencontré.
4. **D2/D3** (session DB) — regrouper dans la prochaine session avec Docker (S1/S2/D3/index).
5. Reste = dette/doc, non bloquant.
