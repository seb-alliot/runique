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

# Installer les certificats et utilitaires nécessaires
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copier le binaire compilé
COPY --from=builder /app/target/release/demo-app /app/app

# Variables d'environnement pour Railway
ENV PORT=8080
# DATABASE_URL doit être défini par Railway dans les variables du projet
# Exemple local : postgres://user:password@host:port/dbname
# ENV DATABASE_URL=postgres://user:password@localhost:5432/demo

EXPOSE 8080

# Lancer demo-app
CMD ["./app"]