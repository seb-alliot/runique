# syntax=docker/dockerfile:1
# Built from the repository root (the workspace): `docker compose --profile demo up --build`.

# ── Build ────────────────────────────────────────────────────────────────────
FROM rust:1.94-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev cmake perl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# The CLI is built on its own: compiled with demo-app, it would pick up the
# features of the whole build instead of `postgres` alone.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p demo-app -p migration && \
    cargo build --release -p runique --bin runique --features postgres && \
    cp target/release/demo-app target/release/migration target/release/runique /tmp/

# ── Runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# psql: the demo seed loads seed.sql through it (COPY … FROM stdin).
RUN apt-get update && apt-get install -y ca-certificates postgresql-client \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /tmp/demo-app /tmp/migration /tmp/runique /usr/local/bin/

# The framework's own static files, at the path the build recorded for them.
COPY runique/static /app/runique/static
# The seeds read the docs, courses and README from ../docs at every start.
COPY docs /app/docs

WORKDIR /app/demo-app
COPY demo-app/templates ./templates
COPY demo-app/static ./static
COPY demo-app/media ./media
COPY demo-app/seed.sql demo-app/sitemap.xml ./

ENV IP_SERVER=0.0.0.0 \
    PORT=3000 \
    STATIC_RUNIQUE_PATH=/app/runique/static

EXPOSE 3000

CMD ["sh", "-c", "migration up && exec demo-app"]
