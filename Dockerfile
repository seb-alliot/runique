# --- Étape 1 : BUILDER ---
# On utilise Rust 1.91 pour être aligné avec ta machine locale
FROM rust:1.91-slim-bookworm AS builder

# Installation des dépendances système nécessaires à la compilation
# pkg-config et libssl-dev sont CRUCIAUX pour compiler sea-orm-cli et les crates réseau
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Installation de la version précise de sea-orm-cli (RC 32)
# On le fait AVANT de copier le code pour mettre cette couche en cache
RUN cargo install sea-orm-cli --version 2.0.0-rc.32

# Copie de tout le workspace
COPY . .

# Compilation de l'application demo-app et de ton outil runique
# Le flag --release optimise les performances (indispensable pour la prod)
RUN cargo build --release

# --- Étape 2 : RUNTIME (Image finale légère) ---
FROM debian:bookworm-slim
WORKDIR /app

# Installation des bibliothèques de partage nécessaires au binaire Rust pour s'exécuter
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# On récupère uniquement les fichiers nécessaires depuis le builder
# 1. Le binaire principal
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app
COPY --from=builder /usr/src/app/target/release/runique /usr/local/bin/runique
COPY --from=builder /usr/local/cargo/bin/sea-orm-cli /usr/local/bin/sea-orm-cli

# --- CORRECTION DES CHEMINS ICI ---
# On crée la structure attendue par ton outil
COPY --from=builder /usr/src/app/demo-app/src/entities /app/src/entities
COPY --from=builder /usr/src/app/demo-app/migration /app/migration
COPY --from=builder /usr/src/app/demo-app/static /app/static
COPY --from=builder /usr/src/app/demo-app/media /app/media
COPY --from=builder /usr/src/app/demo-app/templates /app/templates

# Configuration du port
ENV PORT=3000
EXPOSE 3000

# Lancement de l'application
CMD ["./demo-app"]