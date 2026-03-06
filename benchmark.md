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
See [docs/en/14-sessions.md](docs/en/14-sessions.md) for full session management documentation.

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
Voir [docs/fr/14-sessions.md](docs/fr/14-sessions.md) pour la documentation complète de la gestion des sessions.

## Remarques

- Tous les tests ont été réalisés en local, compilation en mode release, avec la variable d'environnement `TOKIO_WORKER_THREADS=2`.
- Le serveur tient bien la charge, sans aucune erreur, même avec une forte concurrence.
- Les performances peuvent varier selon la route testée (plus de traitements ou d'accès base sur certaines routes).
- Pour des tests en production, il est recommandé de reproduire ces benchmarks sur un VPS ou une machine dédiée.
