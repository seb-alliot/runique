# ---------- Build ----------
FROM rust:1.85-bookworm as builder

WORKDIR /usr/src/app

# Copier TOUT le contenu du dépôt (nécessaire pour un workspace)
COPY . .

# Build spécifique du package demo-app
# On utilise --locked pour s'assurer que le Cargo.lock est respecté
RUN cargo build --release -p demo-app

# ---------- Runtime ----------
FROM debian:bookworm-slim

WORKDIR /app

# Installation des libs essentielles (openssl est souvent oublié et cause des crashs au runtime)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Copie du binaire depuis le builder
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app

# Railway injecte PORT, mais on définit une valeur par défaut
ENV PORT=8080
EXPOSE ${PORT}

CMD ["./demo-app"]