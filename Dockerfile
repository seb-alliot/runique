FROM rust:1.85 as builder

WORKDIR /app

# ✅ copier TOUT le workspace
COPY . .

# build uniquement demo-app
RUN cargo build --release -p demo-app

# runtime
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/demo-app /app/app

ENV PORT=8080
EXPOSE 8080

CMD ["./app"]