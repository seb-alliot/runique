# ---------- Build stage ----------
FROM rust:1.85 as builder

WORKDIR /app

# 1. Copier les manifests pour cache des deps
COPY Cargo.toml Cargo.lock ./
COPY runique/Cargo.toml runique/
COPY runique/derive_form/Cargo.toml runique/derive_form/
COPY demo-app/Cargo.toml demo-app/
COPY demo-app/migration/Cargo.toml demo-app/migration/

# 2. Dummy build pour cache (très important pour perf)
RUN mkdir -p demo-app/src && echo "fn main() {}" > demo-app/src/main.rs
RUN cargo build --release -p demo-app

# 3. Copier le vrai code
COPY . .

# 4. Build final
RUN cargo build --release -p demo-app

# ---------- Runtime stage ----------
FROM debian:bookworm-slim

WORKDIR /app

# dépendances utiles (TLS, etc.)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copier le binaire
COPY --from=builder /app/target/release/demo-app /app/app

ENV PORT=8080
EXPOSE 8080

CMD ["./app"]