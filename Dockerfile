FROM rust:1.91-slim-bookworm AS builder
WORKDIR /usr/src/app

# 1. On installe les outils nécessaires

# Si 'runique' est un binaire de ton projet, on le compilera après

COPY . .

# 2. On compile TOUT le workspace (incluant le CLI runique et l'app)
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Installation forcée de la version RC correspondante
RUN cargo install sea-orm-cli --version 2.0.0-rc.32

# 3. On copie les binaires compilés depuis le builder
# (Adapte le chemin si ton binaire runique est dans un autre dossier target)
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app
COPY --from=builder /usr/src/app/target/release/runique /usr/local/bin/runique
COPY --from=builder /usr/local/cargo/bin/sea-orm-cli /usr/local/bin/sea-orm-cli

# 4. On copie le code source des entités et des migrations
# car 'runique makemigrations' en a besoin pour lire tes structures Rust
COPY --from=builder /usr/src/app/entity /app/entity
COPY --from=builder /usr/src/app/migration /app/migration

ENV PORT=3000
CMD ["./demo-app"]