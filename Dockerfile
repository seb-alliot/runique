# ---------- Build ----------
FROM rust:1.85 as builder

WORKDIR /app

# Copier tout le workspace
COPY . .

# Build uniquement demo-app
RUN cargo build --release -p demo-app

# ---------- Runtime ----------
FROM debian:bookworm-slim

WORKDIR /app

# Installer certificats et bibliothèques nécessaires pour Postgres
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copier le binaire compilé
COPY --from=builder /app/target/release/demo-app /app/demo-app

# Variables d'environnement pour Railway
ENV PORT=8080
# DATABASE_URL est injecté automatiquement par Railway

EXPOSE 8080

# Lancer demo-app
CMD ["./demo-app"]