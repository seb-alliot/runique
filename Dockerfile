# --- Étape 1 : BUILDER ---
FROM rust:1.91-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Installation de sea-orm-cli pour les migrations au build si besoin
RUN cargo install sea-orm-cli --version 2.0.0-rc.32

# On copie tout le projet (demo-app + runique)
COPY . .

# Compilation en mode release
RUN cargo build --release

# --- Étape 2 : RUNTIME (L'image qui tourne sur Railway) ---
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# 1. Récupération des binaires
COPY --from=builder /usr/src/app/target/release/demo-app /app/demo-app
COPY --from=builder /usr/src/app/target/release/runique /usr/local/bin/runique
COPY --from=builder /usr/local/cargo/bin/sea-orm-cli /usr/local/bin/sea-orm-cli

# 2. Création de TOUTE la structure de dossiers
# On crée "src/entities" parce que Sea-ORM le cherche au démarrage
RUN mkdir -p /app/runique/static /app/runique/templates /app/runique/media \
            /app/static /app/media /app/templates \
            /app/src/entities /app/migration


# 3. Copies des fichiers statiques et templates
# On respecte les deux dossiers pour éviter les conflits
COPY --from=builder /usr/src/app/runique/static/ /app/runique/static/
COPY --from=builder /usr/src/app/runique/templates/ /app/runique/templates/

COPY --from=builder /usr/src/app/demo-app/static/ /app/static/
COPY --from=builder /usr/src/app/demo-app/templates/ /app/templates/
COPY --from=builder /usr/src/app/demo-app/media/ /app/media/
COPY --from=builder /usr/src/app/demo-app/sitemap.xml /app/sitemap.xml

# 4. Copie des fichiers sources nécessaires au runtime (Entities & Migrations)
COPY --from=builder /usr/src/app/demo-app/migration/ /app/migration/
COPY --from=builder /usr/src/app/demo-app/src/entities/ /app/src/entities/

# 5. Copie de la documentation (lue par doc_seed.rs au démarrage)
COPY --from=builder /usr/src/app/docs/ /app/docs/

# Droits d'écriture pour les uploads
RUN chmod -R 777 /app/media

# Configuration réseau
ENV PORT=3000
EXPOSE 3000

# On lance l'app
CMD ["./demo-app"]