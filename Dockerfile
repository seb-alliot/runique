FROM rust:1.91-slim-bookworm AS builder
RUN rustup update stable

WORKDIR /usr/src/app

# On copie TOUT le repo (indispensable pour que le workspace fonctionne)
COPY . .

# On compile le package demo-app.
# Comme on est à la racine, Cargo trouve "runique" sans problème.
RUN cargo build --release -p demo-app

FROM debian:bookworm-slim
WORKDIR /app

# Dépendances système pour Rust/Postgres/OpenSSL
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# On récupère le binaire dans le dossier target racine
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app

ENV PORT=8080
EXPOSE 8080

CMD ["./demo-app"]