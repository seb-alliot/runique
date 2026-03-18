# Stage 1: Build
FROM rust:1.85-bookworm AS builder

WORKDIR /usr/src/app

# Copie de TOUT le dépôt (nécessaire pour les dépendances du workspace)
COPY . .

# On compile uniquement le binaire de demo-app
# Le flag --bin garantit qu'on génère le bon exécutable
RUN cd demo-app && cargo build --release --bin demo-app

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Dépendances système nécessaires
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# On récupère le binaire compilé dans le dossier target du workspace
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app

# Railway utilise souvent le port 8080 par défaut
ENV PORT=8080
EXPOSE 8080

# Commande de lancement
CMD ["./demo-app"]