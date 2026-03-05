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

## Remarques

- Tous les tests ont été réalisés en local, compilation en mode release, avec la variable d'environnement `TOKIO_WORKER_THREADS=2`.
- Le serveur tient bien la charge, sans aucune erreur, même avec une forte concurrence.
- Les performances peuvent varier selon la route testée (plus de traitements ou d'accès base sur certaines routes).
- Pour des tests en production, il est recommandé de reproduire ces benchmarks sur un VPS ou une machine dédiée.
