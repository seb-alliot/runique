#!/usr/bin/env bash
#
# Smoke-test multi-moteur de makemigrations (adapté aux machines lentes à compiler).
#
# Pour chaque moteur demandé (SQLite, Postgres, MariaDB) :
#   1. clean slate : suppression des migrations + snapshots + lib.rs de demo-app
#   2. régénération via `runique makemigrations` AVEC le DbKind du moteur
#      (CREATE TYPE/triggers pour PG, ENUM inline pour MariaDB, TEXT pour SQLite)
#   3. exécution réelle : `cargo run -p migration -- fresh` puis `reset`
#
# Les migrations de demo-app sont sauvegardées au début et RESTAURÉES à la fin.
#
# ── Optimisations compilation (laptop lent) ───────────────────────────────────
#   * La lib `runique` (le gros morceau) est compilée UNE SEULE FOIS au début ;
#     le binaire CLI est ensuite appelé directement (pas de `cargo run`).
#   * Seule la petite crate `migration` est recompilée entre les moteurs
#     (sa source est régénérée) ; les dépendances lourdes restent en cache.
#   * On peut ne tester qu'UN moteur pour une boucle rapide sans Docker :
#
# Usage :
#   bash scripts/smoke_migrations.sh              # les 3 moteurs
#   bash scripts/smoke_migrations.sh sqlite       # SQLite seul (rapide, sans Docker)
#   bash scripts/smoke_migrations.sh postgres mariadb
#
# Override des URLs : DATABASE_URL_PG / DATABASE_URL_MARIADB

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MIG_SRC="demo-app/migration/src"
PG_URL="${DATABASE_URL_PG:-postgres://runique:runique_test@localhost:5433/runique_test}"
MARIA_URL="${DATABASE_URL_MARIADB:-mysql://runique:runique_test@localhost:3307/runique_test}"
SQLITE_FILE="$(mktemp -u)_runique_smoke.db"
SQLITE_URL="sqlite://${SQLITE_FILE}?mode=rwc"
RUNIQUE_BIN="$ROOT/target/debug/runique"

# Moteurs demandés (args), sinon les 3.
ENGINES=("$@")
[ "${#ENGINES[@]}" -eq 0 ] && ENGINES=(sqlite postgres mariadb)

want() { for e in "${ENGINES[@]}"; do [ "$e" = "$1" ] && return 0; done; return 1; }

# ── Sauvegarde / restauration des migrations committées ───────────────────────
BACKUP="$(mktemp -d)"
cp -r "$MIG_SRC" "$BACKUP/src"
cleanup() {
    rm -rf "$MIG_SRC"
    cp -r "$BACKUP/src" "$MIG_SRC"
    rm -rf "$BACKUP" "$SQLITE_FILE"
    echo "--- migrations demo-app restaurées ---"
}
trap cleanup EXIT

clean_slate() {
    # Migration files are `m<digits>_*.rs` — must NOT match `main.rs` (the crate's bin).
    find "$MIG_SRC" -maxdepth 1 -name 'm[0-9]*.rs' -delete
    rm -rf "$MIG_SRC/snapshots" "$MIG_SRC/lib.rs"
}

run_engine() {
    local name="$1" url="$2" engine="$3"
    echo ""
    echo "================== $name =================="

    clean_slate

    echo "--- makemigrations ($name) ---"
    if ! ( cd demo-app && DB_URL="$url" DATABASE_URL="$url" DB_ENGINE="$engine" \
            "$RUNIQUE_BIN" makemigrations --entities src/entities --migrations migration/src ); then
        echo "ECHEC: makemigrations ($name)"
        return 1
    fi

    echo "--- migrate fresh ($name) ---"
    if ! DATABASE_URL="$url" cargo run -q -p migration -- fresh; then
        echo "ECHEC: fresh ($name)"
        return 1
    fi

    echo "--- migrate reset ($name) ---"
    if ! DATABASE_URL="$url" cargo run -q -p migration -- reset; then
        echo "ECHEC: reset ($name)"
        return 1
    fi

    echo "OK: $name"
    return 0
}

# ── Build unique du CLI runique (compile la lib lourde une seule fois) ─────────
echo "--- build CLI runique (une fois) ---"
if ! cargo build -q -p runique --bin runique; then
    echo "ECHEC: build du CLI runique"
    exit 1
fi

# ── Docker uniquement si PG/MariaDB sont demandés ─────────────────────────────
if (want postgres || want mariadb) && command -v docker >/dev/null 2>&1 && [ -f docker-compose.yml ]; then
    svc=()
    want postgres && svc+=(postgres)
    want mariadb && svc+=(mariadb)
    echo "--- docker compose up -d --wait ${svc[*]} ---"
    docker compose up -d --wait "${svc[@]}" || \
        echo "ATTENTION: docker compose a échoué — le(s) moteur(s) concerné(s) échoueront s'ils ne répondent pas"
fi

declare -A RESULT
want sqlite   && { run_engine "SQLite"   "$SQLITE_URL" "sqlite"   && RESULT[SQLite]=OK   || RESULT[SQLite]=ECHEC; }
want postgres && { run_engine "Postgres" "$PG_URL"     "postgres" && RESULT[Postgres]=OK || RESULT[Postgres]=ECHEC; }
want mariadb  && { run_engine "MariaDB"  "$MARIA_URL"  "mariadb"  && RESULT[MariaDB]=OK  || RESULT[MariaDB]=ECHEC; }

echo ""
echo "==================== RÉSUMÉ ===================="
fail=0
for engine in SQLite Postgres MariaDB; do
    [ -v "RESULT[$engine]" ] || continue
    printf "  %-10s : %s\n" "$engine" "${RESULT[$engine]}"
    [ "${RESULT[$engine]}" = "OK" ] || fail=1
done
echo "================================================"

exit "$fail"
