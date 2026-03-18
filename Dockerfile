# Stage 1: Build
FROM rust:1.85-bookworm AS builder

WORKDIR /usr/src/app

# On copie TOUT le repo (indispensable pour le workspace)
COPY . .

# ON RESTE À LA RACINE. On utilise -p (package) pour cibler demo-app.
# Cargo trouvera automatiquement le dossier runique car il est à la racine.
RUN cargo build --release -p demo-app

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Installation des libs pour le réseau et la DB
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Le binaire se trouve TOUJOURS dans target/release à la racine du workspace
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app

ENV PORT=8080
EXPOSE 8080

CMD ["./demo-app"]