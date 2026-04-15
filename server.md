# Super-server — Recap architecture utile

## Rôle
- Orchestrateur de runtimes applicatifs (boxes)
- Aucun code métier
- Aucun accès DB / engine

---

## Responsabilités

### 1. Registry
- Stockage des runtimes actifs
- Identification par :
  - host
  - api_name
  - version_id
  - build_hash

---

### 2. Routing
- host → runtime API
- sélection de la box active
- redirection interne uniquement

---

### 3. Lifecycle management
- start API box
- stop API box
- reload API box
- swap A → B (hot swap)
- rollback version

---

### 4. Concurrence / état
- état partagé via Arc
- protection via RwLock (ou équivalent)
- swap atomique logique (transition contrôlée)

---

### 5. Policies globales
- rate limit global
- fail2ban logique (ban temporaire host/IP)
- circuit breaker par API
- mode dégradé global (throttling système)

---

### 6. Isolation
- chaque API = boîte noire
- aucune introspection du contenu
- interaction uniquement via handle

---

### 7. Objectif fonctionnel
- multi-API runtime management
- swap sans interruption globale
- coexistence de versions A/B
- contrôle centralisé des flux