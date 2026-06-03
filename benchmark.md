🌍 **Languages**: [English]

# Runique Benchmarks

This file presents the results of benchmarks performed locally on the development machine, with different routes and load parameters.

| Test                        | Threads | Requests | Concurrency | Total time (ms) | Avg (ms) | RPS    | 90% < (ms) | 99% < (ms) | Success (%) | Notes                   |
|-----------------------------|---------|----------|-------------|-----------------|----------|--------|------------|------------|-------------|-------------------------|
| Local PC (oha #1)           | 2       | 10,000   | 100         | 2258            | 22.38    | 4429   | 29.92      | 47.53      | 100         | Manual TOKIO limit      |
| Local PC (oha #2)           | 2       | 20,000   | 200         | 3985            | 39.54    | 5019   | 53.46      | 68.67      | 100         | Manual TOKIO limit      |
| Local PC (oha /blog/liste)  | 2       | 10,000   | 100         | 2531            | 25.10    | 3951   | 32.05      | 56.18      | 100         | Route /blog/liste       |
| Local PC (release /blog/liste) | 2    | 10,000   | 100         | 1898            | 18.80    | 5270   | 21.11      | 38.27      | 100         | target/release, fastest |
| Local PC (release #root)       | 2    | 1,000    | 50          | 248             | 11.77    | 4035   | 15.28      | 83.30      | 100         | target/release, /       |
| Local PC (release /blog/liste) | 2    | 1,000    | 50          | 269             | 12.76    | 3715   | 14.90      | 30.44      | 100         | target/release, /blog/liste |
| Local PC (30s /blog/liste)     | 2    | 144,863  | 200         | 30,012          | 41.38    | 4833   | 56.17      | 93.11      | 100         | 172 deadline, 3.35 MiB/s |
| Local PC (30s /blog/liste)     | 2    | 134,518  | 500         | 30,028          | 111.3    | 4496   | 120.8      | 153.4      | 100         | 488 deadline, 3.11 MiB/s |
| Local PC (2m /blog/liste)      | 2    | 467,339  | 200         | 120,010         | 51.33    | 3896   | 68.14      | 80.47      | 100         | 184 deadline, 2.70 MiB/s |
| Local PC (30s /)               | 2    | 146,667  | 500         | 30,020          | 102.1    | 4901   | 113.9      | 138.6      | 100         | 470 deadline, 3.57 MiB/s |

## Connection pool impact — Laptop (CSP + CSRF + security headers, Postgres, TOKIO_WORKER_THREADS=2)

Tests performed on laptop (weaker CPU than Ryzen 7 5800X), release build, route `/`, PostgreSQL, with full security middleware stack (CSP, CSRF, security headers, compression).

| Pool config                | Concurrency | RPS   | Avg (ms) | P50 (ms) | P90 (ms) | P95 (ms) | P99 (ms) | Success (%) |
|----------------------------|-------------|-------|----------|----------|----------|----------|----------|-------------|
| min=20 / max=100 (default) | 100         | 376   | 264.8    | 15.0     | 837.0    | 860.4    | 911.1    | 100         |
| min=100 / max=200          | 100         | 1,765 | 55.9     | 15.3     | 21.6     | 723.4    | 813.4    | 100         |

**Key finding**: with default pool (min=20), 80% of connections at 100 concurrency must wait for a slot → bimodal latency (P50=15ms vs P90=837ms). With min=100 pre-opened connections, P90 drops from 837ms to **21ms** (+40x). P50 unchanged (15ms) — the handler itself is fast, the bottleneck is pool acquisition.

Pool is fully configurable via `.env`: `DB_MIN_CONNECTIONS`, `DB_MAX_CONNECTIONS`, `DB_ACQUIRE_TIMEOUT`. Default values (20/100) are safe for most deployments. Increase `DB_MIN_CONNECTIONS` on high-traffic servers.

## Session memory management — before/after fix

`MemoryStore` (tower-sessions default) never deletes expired sessions — memory grows unboundedly under load.
Runique replaces it with `CleaningMemoryStore`: periodic cleanup (60s timer) + two-tier watermark protection.

**Test conditions**: release build, 500 concurrent, 5 min, route `/`, `TOKIO_WORKER_THREADS` default (Ryzen 7 5800X)
Watermarks reduced for stress-test visibility: low = 5 MB, high = 10 MB, cleanup interval = 5s.

| Phase                        | WorkingSet | Private |
|------------------------------|------------|---------|
| Before benchmark             | 16.5 MB    | 5.7 MB  |
| After 5 min / 500 concurrent | 79.2 MB    | 68.7 MB |
| Idle +8 min (post-benchmark) | 39.0 MB    | 27.8 MB |

**Before fix** (raw `MemoryStore`, same load profile): **~1 369 MB** after 5 minutes — unbounded growth.
**After fix**: peak at **79 MB**, returns to **39 MB** at idle. No unbounded growth.

The 1 683 HTTP 500 responses are expected — the high watermark (10 MB) was intentionally set low to trigger
the emergency refusal mechanism under stress. With production defaults (128 MB / 256 MB) no refusals occur
at this load level.

Post-benchmark idle memory does not return to baseline — this is normal allocator behavior (reserved pages),
not a leak. The allocator keeps pages warm after a load spike.
See [docs/en/session/14-sessions.md](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md) for full session management documentation.

## VPS Production — Hostinger KVM (2 vCPU, 8 GB RAM, Debian trixie, PostgreSQL 18, release build)

Route `/docs/fr` — page with multiple DB queries (doc_section + doc_page + doc_block joins). Full middleware stack active (CSP, CSRF, compression, session, trusted proxies). Cloudflare bypassed (direct `127.0.0.1:3000`).

### Baseline — pool min=2 / max=10

| Tool | Concurrency | RPS    | Avg (ms) | P90 (ms) | P95 (ms) | P99 (ms) | Max (ms) | Success (%) |
|------|-------------|--------|----------|----------|----------|----------|----------|-------------|
| wrk (30s) | 100 | 1,250 | 79.8 | — | — | — | 129.4 | 100 |
| oha (1000 req) | 50 | 1,081 | 44.7 | 50.2 | 52.8 | 58.2 | 77.5 | 100 |

### Stress test — pool min=5 / max=30, 2 minutes, 200 concurrent

| Concurrency | RPS | Avg (ms) | P50 (ms) | P90 (ms) | P95 (ms) | P99 (ms) | Max (ms) | Success (%) |
|-------------|-----|----------|----------|----------|----------|----------|----------|-------------|
| 200 | 463 | 428 | 230 | 326 | 2,262 | 2,526 | 2,721 | 100 |

**Observation**: bimodal distribution — 90% of requests under 326ms, P95 jumps to 2.26s (pool exhaustion at 30 connections for 200 concurrent). CPU peaked at 113% (multi-core saturation), RSS stable at 85 MB.

**Pool impact**: 10 connections → bottleneck at 100 concurrent. 30 connections → bottleneck at ~150 concurrent. Increase `DB_MAX_CONNECTIONS` proportionally to expected peak load.

## Cloudflare Edge Cache — Windows PC → runique.io (MRS edge, cache HIT)

Route `/docs/fr` with Cloudflare cache active (Transform Rule strips `Set-Cookie`, Cache Rule TTL 3600s). Requests served from Cloudflare edge — VPS not hit. DNS+dialup overhead included (new TCP connections from oha).

| Tool | Concurrency | RPS | Avg (ms) | P50 (ms) | P90 (ms) | P95 (ms) | P99 (ms) | Success (%) |
|------|-------------|-----|----------|----------|----------|----------|----------|-------------|
| oha (1000 req) | 50 | 808 | 60.1 | 20.9 | 39.5 | 394.8 | 1,095 | 100 |
| oha (2 min) | 50 | 1,745 | 28.7 | 21.7 | 51.4 | 67.9 | 106.6 | 100 |

**Key finding**: P50 drops from 44ms (VPS direct) to **22ms** (Cloudflare edge). VPS CPU stays at ~8% during the 2-minute test — edge absorbs 100% of traffic on cached routes. P95 spike on the 1000-req test is TCP setup overhead (new connections), not representative of real browser traffic (keep-alive). The 2-minute test with connection reuse shows the real P95 at **68ms**.

## Notes

- All tests were performed locally, compiled in release mode, with the environment variable `TOKIO_WORKER_THREADS=2`.
- The server handles the load well, with no errors, even under high concurrency.
- Performance may vary depending on the route tested (more processing or database access on some routes).
- For production tests, it is recommended to reproduce these benchmarks on a VPS or dedicated machine.

```

```

🌍 **Languages**: [Francais]

# Benchmarks Runique

Ce fichier présente les résultats des benchmarks réalisés en local sur la machine de développement, avec différentes routes et paramètres de charge.

| Test                        | Threads | Requêtes | Concurrence | Temps total (ms) | Moyenne (ms) | RPS    | 90% < (ms) | 99% < (ms) | Succès (%) | Remarques                |
|-----------------------------|---------|----------|-------------|------------------|--------------|--------|------------|------------|------------|--------------------------|
| Local PC (oha #1)           | 2       | 10 000   | 100         | 2258             | 22.38        | 4429   | 29.92      | 47.53      | 100        | Limite TOKIO manuelle    |
| Local PC (oha #2)           | 2       | 20 000   | 200         | 3985             | 39.54        | 5019   | 53.46      | 68.67      | 100        | Limite TOKIO manuelle    |
| Local PC (oha /blog/liste)  | 2       | 10 000   | 100         | 2531             | 25.10        | 3951   | 32.05      | 56.18      | 100        | Route /blog/liste        |
| Local PC (release /blog/liste) | 2    | 10 000   | 100         | 1898             | 18.80        | 5270   | 21.11      | 38.27      | 100        | target/release, plus rapide |
| Local PC (release #racine)     | 2    | 1 000    | 50          | 248              | 11.77        | 4035   | 15.28      | 83.30      | 100        | target/release, /        |
| Local PC (release /blog/liste) | 2    | 1 000    | 50          | 269              | 12.76        | 3715   | 14.90      | 30.44      | 100        | target/release, /blog/liste |
| Local PC (30s /blog/liste)     | 2    | 144 863  | 200         | 30 012           | 41.38        | 4833   | 56.17      | 93.11      | 100        | 172 deadline, 3.35 MiB/s |
| Local PC (30s /blog/liste)     | 2    | 134 518  | 500         | 30 028           | 111.3       | 4496   | 120.8      | 153.4      | 100        | 488 deadline, 3.11 MiB/s |
| Local PC (2m /blog/liste)      | 2    | 467 339  | 200         | 120 010          | 51.33        | 3896   | 68.14      | 80.47      | 100        | 184 deadline, 2.70 MiB/s |
| Local PC (30s /)               | 2    | 146 667  | 500         | 30 020           | 102.1        | 4901   | 113.9      | 138.6      | 100        | 470 deadline, 3.57 MiB/s |

## Impact du pool de connexions — Laptop (CSP + CSRF + headers sécurité, Postgres, TOKIO_WORKER_THREADS=2)

Tests réalisés sur laptop (CPU moins puissant qu'un Ryzen 7 5800X), build release, route `/`, PostgreSQL, pile de middlewares sécurité complète (CSP, CSRF, headers sécurité, compression).

| Config pool                | Concurrence | RPS   | Moy. (ms) | P50 (ms) | P90 (ms) | P95 (ms) | P99 (ms) | Succès (%) |
|----------------------------|-------------|-------|-----------|----------|----------|----------|----------|------------|
| min=20 / max=100 (défaut)  | 100         | 376   | 264.8     | 15.0     | 837.0    | 860.4    | 911.1    | 100        |
| min=100 / max=200          | 100         | 1 765 | 55.9      | 15.3     | 21.6     | 723.4    | 813.4    | 100        |

**Enseignement clé** : avec le pool par défaut (min=20), 80% des connexions à 100 concurrents doivent attendre un slot → latence bimodale (P50=15ms vs P90=837ms). Avec 100 connexions pré-ouvertes, le P90 chute de 837ms à **21ms** (÷40). Le P50 reste identique (15ms) — le handler lui-même est rapide, le goulot est l'acquisition du pool.

Le pool est entièrement configurable via `.env` : `DB_MIN_CONNECTIONS`, `DB_MAX_CONNECTIONS`, `DB_ACQUIRE_TIMEOUT`. Les valeurs par défaut (20/100) sont sûres pour la plupart des déploiements. Augmenter `DB_MIN_CONNECTIONS` sur les serveurs à fort trafic.

## Gestion mémoire des sessions — avant/après correctif

`MemoryStore` (défaut tower-sessions) ne supprime jamais les sessions expirées — la mémoire croît sans limite sous charge.
Runique le remplace par `CleaningMemoryStore` : cleanup périodique (timer 60s) + protection par watermarks à deux niveaux.

**Conditions du test** : build release, 500 connexions simultanées, 5 min, route `/`, `TOKIO_WORKER_THREADS` par défaut (Ryzen 7 5800X).
Watermarks réduits pour rendre les mécanismes visibles : low = 5 Mo, high = 10 Mo, intervalle cleanup = 5s.

| Phase                              | WorkingSet | Private |
|------------------------------------|------------|---------|
| Avant benchmark                    | 16.5 Mo    | 5.7 Mo  |
| Après 5 min / 500 connexions       | 79.2 Mo    | 68.7 Mo |
| Idle +8 min (après benchmark)      | 39.0 Mo    | 27.8 Mo |

**Avant correctif** (`MemoryStore` brut, même profil de charge) : **~1 369 Mo** après 5 minutes — croissance sans limite.
**Après correctif** : pic à **79 Mo**, retour à **39 Mo** au repos. Aucune croissance unbounded.

Les 1 683 erreurs HTTP 500 sont attendues — le high watermark (10 Mo) était volontairement bas pour déclencher
le mécanisme de refus d'urgence sous stress. Avec les valeurs de production (128 Mo / 256 Mo) aucun refus n'intervient
à ce niveau de charge.

La mémoire idle après benchmark ne revient pas au niveau initial — comportement normal de l'allocateur (pages réservées),
pas une fuite. L'allocateur conserve les pages chaudes après un pic de charge.
Voir [docs/fr/session/14-sessions.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md) pour la documentation complète de la gestion des sessions.

## VPS Production — Hostinger KVM (2 vCPU, 8 Go RAM, Debian trixie, PostgreSQL 18, build release)

Route `/docs/fr` — page avec plusieurs requêtes DB (jointures doc_section + doc_page + doc_block). Stack middleware complète active (CSP, CSRF, compression, session, trusted proxies). Cloudflare bypassé (direct `127.0.0.1:3000`).

### Baseline — pool min=2 / max=10

| Outil | Concurrence | RPS    | Moy. (ms) | P90 (ms) | P95 (ms) | P99 (ms) | Max (ms) | Succès (%) |
|-------|-------------|--------|-----------|----------|----------|----------|----------|------------|
| wrk (30s) | 100 | 1 250 | 79.8 | — | — | — | 129.4 | 100 |
| oha (1000 req) | 50 | 1 081 | 44.7 | 50.2 | 52.8 | 58.2 | 77.5 | 100 |

### Stress test — pool min=5 / max=30, 2 minutes, 200 connexions simultanées

| Concurrence | RPS | Moy. (ms) | P50 (ms) | P90 (ms) | P95 (ms) | P99 (ms) | Max (ms) | Succès (%) |
|-------------|-----|-----------|----------|----------|----------|----------|----------|------------|
| 200 | 463 | 428 | 230 | 326 | 2 262 | 2 526 | 2 721 | 100 |

**Observation** : distribution bimodale — 90% des requêtes sous 326ms, P95 saute à 2,26s (épuisement du pool à 30 connexions pour 200 concurrent). CPU pic à 113% (saturation multi-cœur), RSS stable à 85 Mo.

**Impact du pool** : 10 connexions → goulot à 100 concurrent. 30 connexions → goulot vers ~150 concurrent. Augmenter `DB_MAX_CONNECTIONS` proportionnellement au pic de charge attendu.

## Remarques

- Tous les tests ont été réalisés en local, compilation en mode release, avec la variable d'environnement `TOKIO_WORKER_THREADS=2`.
- Le serveur tient bien la charge, sans aucune erreur, même avec une forte concurrence.
- Les performances peuvent varier selon la route testée (plus de traitements ou d'accès base sur certaines routes).
- Pour des tests en production, il est recommandé de reproduire ces benchmarks sur un VPS ou une machine dédiée.
