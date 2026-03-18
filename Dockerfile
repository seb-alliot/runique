# Stage 1: Build
FROM rust:1.85-bookworm AS builder

WORKDIR /usr/src/app

# 1. On copie TOUT le contenu du dépôt GitHub
COPY . .

# 2. DEBUG : On affiche la structure pour être sûr que tout est là
RUN ls -R

# 3. On build depuis la RACINE en utilisant le flag --package
# C'est LA méthode officielle pour les workspaces
RUN cargo build --release --package demo-app

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# 4. Le binaire est TOUJOURS dans target/release à la racine du WORKSPACE
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app

ENV PORT=8080
EXPOSE 8080

CMD ["./demo-app"]